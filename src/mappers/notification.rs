use mongodb::bson::{Document, Bson};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::{Notification, Linked, NotificationRepoError};
use crate::mappers::common::object_id_to_string_or_empty;

// Infra -> Dominio
// language: idioma a usar para i18n, por defecto "es"
pub fn doc_to_domain(doc: Document, language: &str) -> Result<Notification, NotificationRepoError> {
    let id = object_id_to_string_or_empty(doc.get_object_id("_id").ok());
    
    // Priorizar i18nTitle sobre title, filtrando por lang del parámetro language
    let title = if let Ok(i18n_array) = doc.get_array("i18nTitle") {
        i18n_array
            .iter()
            .filter_map(|v| v.as_document())
            .find(|d| d.get_str("lang").unwrap_or("") == language)
            .and_then(|d| d.get_str("text").ok())
            .map(String::from)
            .unwrap_or_else(|| doc.get_str("title").unwrap_or("").to_string())
    } else {
        doc.get_str("title").unwrap_or("").to_string()
    };
    
    // Priorizar i18nBody sobre body, filtrando por lang del parámetro language
    let body = if let Ok(i18n_array) = doc.get_array("i18nBody") {
        i18n_array
            .iter()
            .filter_map(|v| v.as_document())
            .find(|d| d.get_str("lang").unwrap_or("") == language)
            .and_then(|d| d.get_str("text").ok())
            .map(String::from)
            .unwrap_or_else(|| doc.get_str("body").unwrap_or("").to_string())
    } else {
        doc.get_str("body").unwrap_or("").to_string()
    };

    let image_paths: Vec<String> = doc
        .get_array("imagePath")
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let url = doc.get_str("url").unwrap_or("").to_string();

    let user_targets: Vec<String> = doc
        .get_array("userTargets")
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let topic = doc.get_str("topic").ok().map(String::from);
    let notification_type = doc.get_i32("type").unwrap_or(1);

    let creation_date = if let Ok(dt) = doc.get_datetime("creationDate") {
        DateTime::<Utc>::from_timestamp_millis(dt.timestamp_millis()).unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };

    let payload_type = doc.get_i32("payloadType").unwrap_or(1);

    // Leer accountTypeTargets del documento (array de ObjectIds)
    let account_type_targets: Vec<String> = doc
        .get_array("accountTypeTargets")
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // Leer linked del documento - leer directamente lo que venga
    let linked = if let Some(Bson::Document(linked_doc)) = doc.get("linked") {
        let linked_type = linked_doc.get_i32("type").unwrap_or(0);
        let object_id = linked_doc
            .get_object_id("objectId")
            .ok()
            .map(|oid| oid.to_hex())
            .filter(|s| !s.is_empty());
        let object = linked_doc.get("object")
            .and_then(|b| {
                match b {
                    Bson::Null => None,
                    _ => Some(b.clone().into_relaxed_extjson()),
                }
            });
        Linked { linked_type, object_id, object }
    } else {
        Linked { linked_type: 0, object_id: None, object: None }
    };
    // Browser (por defecto 2 si no existe)
    let browser = doc.get_i32("browser").unwrap_or(2);

    // business_id viene del contexto de autenticación (JWT), no del documento
    Ok(Notification {
        id,
        title,
        body,
        image_paths,
        url,
        user_targets,
        topic,
        notification_type,
        creation_date,
        payload_type,
        business_id: None, // Viene del contexto de autenticación (JWT), no del documento
        linked,
        browser,
        account_type_targets,
    })
}

// Dominio -> Response DTO
#[derive(Serialize)]
pub struct NotificationResponse {
    pub notification: NotificationDto,
}

#[allow(non_snake_case)] // Los nombres están en camelCase para la API externa
#[derive(Serialize)]
pub struct NotificationDto {
    pub id: String,
    pub title: String,
    pub body: String,
    pub imageUrls: Vec<String>,
    pub imagePath: Vec<String>,
}

pub fn domain_to_response(n: Notification) -> NotificationResponse {
    // Mapear imageUrls desde image_paths (mismo contenido)
    let image_urls = n.image_paths.clone();
    
    let dto = NotificationDto {
        id: n.id,
        title: n.title,
        body: n.body,
        imageUrls: image_urls,
        imagePath: n.image_paths,
    };
    
    NotificationResponse { notification: dto }
}
