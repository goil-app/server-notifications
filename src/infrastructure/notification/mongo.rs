use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};

use crate::domain::{Notification, NotificationRepository, NotificationRepoError};
use crate::mappers::notification::doc_to_domain;

#[derive(Clone)]
pub struct MongoNotificationRepository {
    db: Database,
}

impl MongoNotificationRepository {
    pub fn new(db: Database) -> Self { Self { db } }
}

#[async_trait]
impl NotificationRepository for MongoNotificationRepository {
    async fn find_by_id(&self, id: &str, language: &str, business_id: &str) -> Result<Notification, NotificationRepoError> {
        let oid = ObjectId::parse_str(id).map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        let bid = ObjectId::parse_str(business_id).map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        let filter = doc! { 
            "_id": oid, 
            "businessId": bid 
        };
        let coll = self.db.collection::<Document>("Notification");
        let doc = match coll
            .find_one(filter, None)
            .await
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(NotificationRepoError::NotFound),
        };
        doc_to_domain(doc, language)
    }
}


