use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use futures::stream::TryStreamExt;
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
            .find_one(doc! { "_id": oid, "businessId": bid }, options)
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
            .find_one(doc! { "_id": oid, "businessId": { "$in": business_oids } }, options)
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(UserRepoError::NotFound)
        };
        doc_to_simplified(doc)
    }

    async fn find_by_phone_and_business_ids(&self, phone: &str, business_ids: &[String]) -> Result<Vec<SimplifiedUser>, UserRepoError> {
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

        let options = FindOptions::builder()
            .projection(doc! { "_id": 1, "phone": 1, "creationDate": 1, "accountType": 1 })
            .build();

        let coll = self.db.collection::<Document>("Account");
        let mut cursor = coll
            .find(filter, options)
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))?;
        
        let mut users = Vec::new();
        while let Some(result) = cursor
            .try_next()
            .await
            .map_err(|e| UserRepoError::Unexpected(e.to_string()))? {
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