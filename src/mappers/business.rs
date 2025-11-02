use mongodb::bson::Document;
use crate::domain::{Business, BusinessRepoError};

// Infra -> Dominio
pub fn doc_to_domain(doc: Document) -> Result<Business, BusinessRepoError> {
    let name = doc.get_str("name")
        .map_err(|e| BusinessRepoError::Unexpected(format!("Error reading name field: {}", e)))?
        .to_string();
    
    Ok(Business { name })
}

