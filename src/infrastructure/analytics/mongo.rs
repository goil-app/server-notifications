use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use futures::stream::TryStreamExt; // Necesario para try_collect()

use crate::domain::analytics::{NotificationReadRepository, NotificationReadRepoError};
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
    async fn find_by_phone_and_business_ids(&self, phone: &str, business_ids: &[String]) -> Result<Vec<String>, NotificationReadRepoError> {
        // Convertir businessIds de String a ObjectId
        let business_oids: Result<Vec<ObjectId>, _> = business_ids
            .iter()
            .map(|bid| ObjectId::parse_str(bid))
            .collect();
        let business_oids = business_oids.map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        
        // Buscar notification reads por phone y filtrar por businessId (ObjectId) que esté en el array
        let filter = doc! {
            "phone": phone,
            "businessId": { "$in": business_oids }
        };

        // IMPORTANTE: Para máximo rendimiento, crear índice en MongoDB:
        // db.NotificationRead.createIndex({ "phone": 1, "businessId": 1 })
        let options = FindOptions::builder()
            .projection(doc! { "notificationId": 1 })
            .limit(1000) // Límite razonable para obtener reads precisos
            .batch_size(100) // Batch size óptimo
            .build();

        let coll = self.db.collection::<Document>("NotificationRead");
        
        // Optimización: usar collect en lugar de iterar cursor para mejor rendimiento
        let cursor = coll
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        
        let docs: Vec<Document> = cursor
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| NotificationReadRepoError::Unexpected(e.to_string()))?;
        
        let mut notification_ids = Vec::new();
        for result in docs {
                let id = object_id_to_string_or_empty(result.get_object_id("notificationId").ok());
                if !id.is_empty() {
                    notification_ids.push(id);
                }
        }
        Ok(notification_ids)
    }
}

