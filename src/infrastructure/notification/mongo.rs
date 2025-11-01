use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt;

use crate::domain::{Notification, NotificationRepository, NotificationRepoError, SimplifiedUser};
use crate::mappers::{notification::doc_to_domain, common::object_id_to_string_or_empty};

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
            "businessId": bid,
            "deleted": false 
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

    async fn find_user_notifications(&self, simplified_user: &SimplifiedUser) -> Result<Vec<String>, NotificationRepoError> {
        let user_oid = ObjectId::parse_str(&simplified_user.id)
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        let business_oid = ObjectId::parse_str(&simplified_user.business_id)
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        
        // Convertir DateTime<Utc> a mongodb::bson::DateTime
        let account_creation_bson = mongodb::bson::DateTime::from_millis(
            simplified_user.creation_date.timestamp_millis()
        );

        let account_type_id = &simplified_user.account_type;
        let phone = &simplified_user.phone;
        let topic_all_business_id = format!("all_{}", simplified_user.business_id);

        let external_hidden_type = 17;

        let filter = doc! {
            "businessId": business_oid,
            "creationDate": { "$gt": account_creation_bson },
            "deleted": false,
            "type": { "$ne": external_hidden_type },
            "$or": [
                { "topic": { "$regex": account_type_id } },
                { "topic": { "$eq": &topic_all_business_id } },
                { "userTargets": { "$in": [&user_oid] } },
                { "accountTypeTargets": { "$in": [account_type_id] } },
                { "userTargetsChannel": { "$in": [&user_oid] } },
                { "phones": { "$in": [phone] } }
            ]
        };

        let options = FindOptions::builder()
            .projection(doc! { "_id": 1 })
            .build();

        let coll = self.db.collection::<Document>("Notification");
        let mut cursor = coll
        .find(filter, options)
        .await
        .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;

        let mut notification_ids = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| NotificationRepoError::Unexpected(e.to_string()))? {
            let id = object_id_to_string_or_empty(result.get_object_id("_id").ok());
            println!("{:?}", id);
            if !id.is_empty() {
                notification_ids.push(id);
            }
        }

        Ok(notification_ids)
    }
}
