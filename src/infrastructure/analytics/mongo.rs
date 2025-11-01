use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt;

use crate::domain::analytics::{NotificationReadRepository, NotificationReadRepoError};
use crate::domain::SimplifiedUser;
use crate::mappers::common::object_id_to_string_or_empty;

#[derive(Clone)]
pub struct MongoNotificationReadRepository {
    db: Database,
}

impl MongoNotificationReadRepository {
    pub fn new(db: Database) -> Self { Self { db } }
}

#[async_trait]
impl NotificationReadRepository for MongoNotificationReadRepository {
    async fn find_by_user_id(&self, simplified_user: &SimplifiedUser) -> Result<Vec<String>, NotificationReadRepoError> {
        let user_oid = ObjectId::parse_str(&simplified_user.id)
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        let business_oid = ObjectId::parse_str(&simplified_user.business_id)
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        
        // Buscar notification reads por accountId o phone, y filtrar por businessId
        let filter = doc! {
            "accountId": user_oid,
            "businessId": business_oid
        };

        let options = FindOptions::builder()
            .projection(doc! { "notificationId": 1 })
            .build();

        let coll = self.db.collection::<Document>("NotificationRead");
        let mut cursor = coll
            .find(filter, options)
            .await
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        
        let mut notification_ids = Vec::new();
        while let Some(result) = cursor
            .try_next()
            .await
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))? {
                let id = object_id_to_string_or_empty(result.get_object_id("notificationId").ok());
                if !id.is_empty() {
                    notification_ids.push(id);
                }
        }

        println!("notification_ids: {:?}", notification_ids.len());

        Ok(notification_ids)
    }
}

