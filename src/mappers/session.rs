use mongodb::bson::Document;

use crate::domain::{Session, SessionRepoError};
use crate::mappers::common::object_id_to_string_or_empty;

// Infra -> Dominio
pub fn doc_to_domain(doc: Document) -> Result<Session, SessionRepoError> {
    let id = object_id_to_string_or_empty(doc.get_object_id("_id").ok());
    let language = doc.get_str("language").unwrap_or("es").to_string();
    Ok(Session { id, language })
}