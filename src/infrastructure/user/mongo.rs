use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use futures::stream::TryStreamExt; // Necesario para try_collect()
use crate::domain::{SimplifiedUser, UserRepository, UserRepoError};
use crate::mappers::user::doc_to_simplified;

#[derive(Clone)]
pub struct MongoUserRepository {
    db: Database,
}

impl MongoUserRepository {
    pub fn new(db: Database) -> Self { Self { db } }
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn find_simplified_by_id(&self, id: &str, business_id: &str) -> Result<SimplifiedUser, UserRepoError> {
        let oid = ObjectId::parse_str(id).map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        let bid = ObjectId::parse_str(business_id).map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        let options = FindOneOptions::builder()
            .projection(doc! { "_id": 1, "phone": 1, "creationDate": 1, "accountType": 1 })
            .build();
        let coll = self.db.collection::<Document>("Account");
        let doc = match coll
            .find_one(doc! { "_id": oid, "businessId": bid })
            .with_options(options)
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(UserRepoError::NotFound)
        };
        doc_to_simplified(doc)
    }

    async fn find_by_id_and_business_ids(&self, id: &str, business_ids: &[String]) -> Result<SimplifiedUser, UserRepoError> {
        let oid = ObjectId::parse_str(id).map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        // Convertir businessIds de String a ObjectId
        let business_oids: Result<Vec<ObjectId>, _> = business_ids
            .iter()
            .map(|bid| ObjectId::parse_str(bid))
            .collect();
        let business_oids = business_oids.map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        let options = FindOneOptions::builder()
            .projection(doc! { "_id": 1, "phone": 1, "creationDate": 1, "accountType": 1 })
            .build();
        
        let coll = self.db.collection::<Document>("Account");
        let doc = match coll
            .find_one(doc! { "_id": oid, "businessId": { "$in": business_oids } })
            .with_options(options)
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(UserRepoError::NotFound)
        };
        doc_to_simplified(doc)
    }

    async fn find_by_phone_and_business_ids(&self, phone: &str, business_ids: &[String]) -> Result<Vec<SimplifiedUser>, UserRepoError> {
        // Si no hay business_ids, retornar vacío (evitar query innecesaria)
        if business_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Convertir businessIds de String a ObjectId
        let business_oids: Result<Vec<ObjectId>, _> = business_ids
            .iter()
            .map(|bid| ObjectId::parse_str(bid))
            .collect();
        let business_oids = business_oids.map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        // Buscar usuarios por phone y filtrar por businessId que esté en el array
        let filter = doc! {
            "phone": phone,
            "businessId": { "$in": business_oids }
        };

        // IMPORTANTE: Para máximo rendimiento, crear índice en MongoDB:
        // db.Account.createIndex({ "phone": 1, "businessId": 1 }, { name: "phone_businessId_idx" })
        // 
        // Optimizaciones aplicadas:
        // - batch_size mayor que limit para evitar round-trips extra
        // - limit razonable para evitar cargar demasiados documentos
        // - pre-allocación de capacidad del vector
        let options = FindOptions::builder()
            .projection(doc! { "_id": 1, "phone": 1, "creationDate": 1, "accountType": 1 })
            .limit(20) // Límite razonable (normalmente hay pocos usuarios con el mismo teléfono)
            .batch_size(50) // Mayor que limit para evitar round-trips adicionales
            .hint(mongodb::options::Hint::Keys(doc! { "phone": 1, "businessId": 1 })) // Forzar uso del índice compuesto
            .build();

        let coll = self.db.collection::<Document>("Account");
        
        // Optimización: usar collect en lugar de iterar cursor para mejor rendimiento
        let cursor = coll
            .find(filter)
            .with_options(options)
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        let docs: Vec<Document> = cursor
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        let mut users = Vec::with_capacity(docs.len()); // Pre-allocar capacidad
        for result in docs {
            match doc_to_simplified(result) {
                Ok(user) => users.push(user),
                Err(e) => {
                    eprintln!("[MongoUserRepository::find_by_phone_and_business_ids] Error mapping document: {:?}", e);
                    // Continuamos con el siguiente documento en lugar de fallar
                }
            }
        }

        Ok(users)
    }
}