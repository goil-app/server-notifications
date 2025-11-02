use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt;
use std::collections::HashSet;

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

    async fn find_users_notifications(&self, users: &[SimplifiedUser], business_ids: &[String]) -> Result<Vec<String>, NotificationRepoError> {
        if users.is_empty() {
            return Ok(Vec::new());
        }

        // Convertir businessIds de String a ObjectId
        let business_oids: Result<Vec<ObjectId>, _> = business_ids
            .iter()
            .map(|bid| ObjectId::parse_str(bid))
            .collect();
        let business_oids = business_oids.map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        
        let mut user_ids = Vec::new();
        let mut account_types_set = HashSet::new();
        let mut phones_set = HashSet::new();
        let mut oldest_creation_date: Option<mongodb::bson::DateTime> = None;

        for user in users {
            user_ids.push(user.id.clone());
            account_types_set.insert(user.account_type.clone());
            phones_set.insert(user.phone.clone());
            
            let user_creation_bson = mongodb::bson::DateTime::from_millis(user.creation_date.timestamp_millis());
            oldest_creation_date = match oldest_creation_date {
                Some(current) if current < user_creation_bson => Some(current),
                None => Some(user_creation_bson),
                Some(current) => Some(current),
            };
        }

        // Eliminar duplicados de user_ids
        let user_ids: Vec<String> = user_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
        let account_types: Vec<String> = account_types_set.into_iter().collect();
        let phones: Vec<String> = phones_set.into_iter().collect();

        // Usar la fecha de creación más antigua para filtrar (ISODate de MongoDB)
        let account_creation_date = oldest_creation_date.unwrap();
        
        // Crear topics "all_{businessId}" para cada businessId del array
        let topic_all_business_ids: Vec<String> = business_ids.iter()
            .map(|bid| format!("all_{}", bid))
            .collect();

        let external_hidden_type = 17;

        // Construir el filtro: notificaciones que coincidan con cualquiera de los usuarios
        // businessId acepta array de businessIds, topic también acepta array
        // creationDate se compara como ISODate de MongoDB
        let filter = doc! {
            "businessId": { "$in": business_oids },
            "creationDate": { "$gt": account_creation_date },
            "deleted": false,
            "type": { "$ne": external_hidden_type },
            "$or": [
                { "topic": { "$in": account_types.clone() } },
                { "topic": { "$in": topic_all_business_ids } },
                { "userTargets": { "$in": user_ids.clone() } },
                { "accountTypeTargets": { "$in": account_types } },
                { "userTargetsChannel": { "$in": user_ids } },
                { "phones": { "$in": phones } }
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
            if !id.is_empty() {
                notification_ids.push(id);
            }
        }

        // Eliminar duplicados usando HashSet
        let mut unique_ids: HashSet<String> = HashSet::new();
        unique_ids.extend(notification_ids);
        Ok(unique_ids.into_iter().collect())
    }
}
