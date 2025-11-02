use mongodb::bson::Document;
use chrono::{DateTime, Utc};

use crate::domain::{SimplifiedUser, UserRepoError};
use crate::mappers::common::object_id_to_string_or_empty;

// Infra -> Dominio
pub fn doc_to_simplified(doc: Document) -> Result<SimplifiedUser, UserRepoError> {
    let id = object_id_to_string_or_empty(doc.get_object_id("_id").ok());
    
    let phone = doc.get_str("phone")
        .unwrap_or("")
        .to_string();
    
    let creation_date = if let Ok(dt) = doc.get_datetime("creationDate") {
        DateTime::<Utc>::from_timestamp_millis(dt.timestamp_millis())
            .unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };
    
    let account_type = object_id_to_string_or_empty(doc.get_object_id("accountType").ok());
    
    Ok(SimplifiedUser {
        id,
        phone,
        creation_date,
        account_type,
    })
}

