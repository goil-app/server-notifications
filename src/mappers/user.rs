use mongodb::bson::Document;
use chrono::{DateTime, Utc};

use crate::domain::{SimplifiedUser, UserRepoError};

// Infra -> Dominio
pub fn doc_to_simplified(doc: Document) -> Result<SimplifiedUser, UserRepoError> {
    let id = doc.get_object_id("_id")
        .map(|oid| oid.to_hex())
        .unwrap_or_default();
    
    let phone = doc.get_str("phone")
        .unwrap_or("")
        .to_string();
    
    let creation_date = if let Ok(dt) = doc.get_datetime("creationDate") {
        DateTime::<Utc>::from_timestamp_millis(dt.timestamp_millis())
            .unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };
    
    let account_type = doc.get_object_id("accountType")
        .map(|oid| oid.to_hex())
        .unwrap_or_default();
    
    let business_id = doc.get_object_id("businessId")
        .map(|oid| oid.to_hex())
        .unwrap_or_default();
    
    Ok(SimplifiedUser {
        id,
        phone,
        creation_date,
        account_type,
        business_id,
    })
}

