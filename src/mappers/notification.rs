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

    // Extraer campos adicionales o usar valores dummy
    let url = doc.get_str("url")
        .map(String::from)
        .unwrap_or_else(|_| "".to_string()); // Dummy: string vacío
    
    let notification_type = doc.get_i32("type")
        .unwrap_or(0); // Dummy: 0
    
    let payload_type = doc.get_i32("payloadType")
        .unwrap_or(0); // Dummy: 0
    
    let is_read = doc.get_bool("isRead")
        .unwrap_or(false); // Dummy: false

    Ok(Notification {
        id,
        title,
        body,
        image_paths,
        url,
        r#type: notification_type,
        payload_type,
        is_read,
    })
}

// Dominio -> Response DTO
#[allow(non_snake_case)] // Los nombres están en camelCase para la API externa
#[derive(Serialize)]
pub struct NotificationResponse {
    pub notification: NotificationDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub businessName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub businessId: Option<String>,
}

#[allow(non_snake_case)] // Los nombres están en camelCase para la API externa
#[derive(Serialize)]
pub struct NotificationDto {
    pub id: String,
    pub title: String,
    pub body: String,
    pub imageUrls: Vec<String>,
    pub imagePath: Vec<String>,
    pub url: String,
    pub r#type: i32, // r#type porque "type" es una palabra reservada en Rust
    pub payloadType: i32,
    pub isRead: bool,
}

pub async fn domain_to_response(
    n: Notification, 
    s3_signer: &crate::infrastructure::s3::S3UrlSigner,
    business_id: Option<String>,
    business_name: Option<String>,
    unread_count: i32,
) -> NotificationResponse {
    // Firmar URLs de S3 para imageUrls
    // Duración por defecto: 600 segundos (10 minutos)
    let expires_in = std::env::var("S3_URL_EXPIRES_IN")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(600);
    
    let image_urls = if n.image_paths.is_empty() {
        Vec::new()
    } else {
        s3_signer.sign_urls(&n.image_paths, expires_in).await
            .unwrap_or_else(|e| {
                eprintln!("[domain_to_response] Error signing S3 URLs: {}", e);
                // Si falla la firma, retornar las rutas originales
                n.image_paths.clone()
            })
    };
    
    let dto = NotificationDto {
        id: n.id,
        title: n.title,
        body: n.body,
        imageUrls: image_urls,
        imagePath: n.image_paths,
        url: n.url,
        r#type: n.r#type,
        payloadType: n.payload_type,
        isRead: n.is_read,
    };
    
    NotificationResponse { 
        notification: dto,
        badge: Some(unread_count),
        businessName: business_name,
        businessId: business_id,
    }
}
