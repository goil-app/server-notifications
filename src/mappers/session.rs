use mongodb::bson::Document;

use crate::domain::{Session, SessionRepoError};

// Infra -> Dominio
pub fn doc_to_domain(doc: Document) -> Result<Session, SessionRepoError> {
    let id = doc.get_object_id("_id").map(|oid| oid.to_hex()).unwrap_or_default();
    let language = doc.get_str("language").unwrap_or("es").to_string();
    Ok(Session { id, language })
}