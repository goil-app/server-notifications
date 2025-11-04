use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt;
use std::collections::HashSet;

use crate::domain::{Notification, NotificationRepository, NotificationRepoError, SimplifiedUser};
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
            "businessId": bid,
            "deleted": false 
        };
        let coll = self.db.collection::<Document>("Notification");
        let doc = match coll
            .find_one(filter)
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
        
        let mut user_ids_set = HashSet::new();
        let mut account_types_set = HashSet::new();
        let mut phones_set = HashSet::new();
        let mut oldest_creation_date: Option<mongodb::bson::DateTime> = None;

        for user in users {
            user_ids_set.insert(user.id.clone());
            account_types_set.insert(user.account_type.clone());
            phones_set.insert(user.phone.clone());
            
            let user_creation_bson = mongodb::bson::DateTime::from_millis(user.creation_date.timestamp_millis());
            oldest_creation_date = match oldest_creation_date {
                Some(current) if current < user_creation_bson => Some(current),
                None => Some(user_creation_bson),
                Some(current) => Some(current),
            };
        }

        // Convertir sets a vectores (ya no hay duplicados que eliminar)
        let user_ids: Vec<String> = user_ids_set.into_iter().collect();
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
        // OPTIMIZACIÓN: Simplificar el filtro si las listas están vacías
        let mut or_conditions = vec![];
        
        if !account_types.is_empty() {
            or_conditions.push(doc! { "topic": { "$in": account_types.clone() } });
            or_conditions.push(doc! { "accountTypeTargets": { "$in": account_types.clone() } });
        }
        
        if !topic_all_business_ids.is_empty() {
            or_conditions.push(doc! { "topic": { "$in": topic_all_business_ids } });
        }
        
        if !user_ids.is_empty() {
            or_conditions.push(doc! { "userTargets": { "$in": user_ids.clone() } });
            or_conditions.push(doc! { "userTargetsChannel": { "$in": user_ids.clone() } });
        }
        
        if !phones.is_empty() {
            or_conditions.push(doc! { "phones": { "$in": phones } });
        }
        
        // Si no hay condiciones OR, retornar vacío
        if or_conditions.is_empty() {
            return Ok(Vec::new());
        }
        
        let filter = doc! {
            "businessId": { "$in": business_oids },
            "creationDate": { "$gt": account_creation_date },
            "deleted": false,
            "type": { "$ne": external_hidden_type },
            "$or": or_conditions
        };

        // Optimización: limitar resultados y usar batch size óptimo
        // IMPORTANTE: Para máximo rendimiento, crear índices en MongoDB:
        // db.Notification.createIndex({ "businessId": 1, "creationDate": -1, "deleted": 1, "type": 1 })
        // db.Notification.createIndex({ "topic": 1 })
        // db.Notification.createIndex({ "userTargets": 1 })
        // db.Notification.createIndex({ "phones": 1 })
        let options = FindOptions::builder()
            .projection(doc! { "_id": 1 })
            .limit(1000) // Límite razonable para obtener unread count preciso
            .batch_size(100) // Batch size óptimo para transferencia
            .build();

        let coll = self.db.collection::<Document>("Notification");
        
        // Optimización: usar collect en lugar de iterar cursor para mejor rendimiento
        let cursor = coll
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;
        
        let docs: Vec<Document> = cursor
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| NotificationRepoError::Unexpected(e.to_string()))?;

        // Extraer IDs y eliminar duplicados en una sola pasada
        let mut unique_ids: HashSet<String> = HashSet::new();
        for doc in docs {
            if let Ok(oid) = doc.get_object_id("_id") {
                unique_ids.insert(oid.to_hex());
            }
        }
        
        Ok(unique_ids.into_iter().collect())
    }
}
