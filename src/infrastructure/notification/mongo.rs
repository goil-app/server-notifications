use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt;
use std::collections::HashSet;
use serde_json;

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
        let business_oid = ObjectId::parse_str(&simplified_user.business_id)
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        
        // Convertir DateTime<Utc> a ISODate de MongoDB
        let account_creation_date = mongodb::bson::DateTime::from_millis(
            simplified_user.creation_date.timestamp_millis()
        );

        let account_type_id = &simplified_user.account_type;
        let phone = &simplified_user.phone;
        let topic_all_business_id = format!("all_{}", simplified_user.business_id);
        let user_id = &simplified_user.id;

        let external_hidden_type = 17;

        let filter = doc! {
            "businessId": business_oid,
            "creationDate": { "$gt": account_creation_date },
            "deleted": false,
            "type": { "$ne": external_hidden_type },
            "$or": [
                { "topic": { "$regex": account_type_id } },
                { "topic": { "$eq": &topic_all_business_id } },
                { "userTargets": { "$in": [user_id] } },
                { "accountTypeTargets": { "$in": [account_type_id] } },
                { "userTargetsChannel": { "$in": [user_id] } },
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
            if !id.is_empty() {
                notification_ids.push(id);
            }
        }

        Ok(notification_ids)
    }

    async fn find_users_notifications(&self, users: &[SimplifiedUser]) -> Result<Vec<String>, NotificationRepoError> {
        if users.is_empty() {
            return Ok(Vec::new());
        }

        // Extraer todos los user_ids (como strings), business_id (único), account_types, phones y fechas de creación
        // Asumimos que todos los usuarios tienen el mismo businessId
        let first_user = &users[0];
        let business_oid = ObjectId::parse_str(&first_user.business_id)
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        
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
        
        // Crear topic "all_{businessId}" único (todos tienen el mismo businessId)
        let topic_all_business_id = format!("all_{}", first_user.business_id);

        let external_hidden_type = 17;

        // Construir el filtro: notificaciones que coincidan con cualquiera de los usuarios
        // businessId es un ObjectId único (no array), topic también es único (no array)
        // creationDate se compara como ISODate de MongoDB
        let filter = doc! {
            "businessId": business_oid,
            "creationDate": { "$gt": account_creation_date },
            "deleted": false,
            "type": { "$ne": external_hidden_type },
            "$or": [
                { "topic": { "$in": account_types.clone() } },
                { "topic": &topic_all_business_id },
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
