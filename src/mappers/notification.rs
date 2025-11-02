use mongodb::bson::Document;
use serde::Serialize;

use crate::domain::{Notification, NotificationRepoError};
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

    Ok(Notification {
        id,
        title,
        body,
        image_paths,
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
