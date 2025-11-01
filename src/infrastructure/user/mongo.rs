use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOneOptions;
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
            .projection(doc! { "_id": 1, "phone": 1, "creationDate": 1, "accountType": 1, "businessId": 1 })
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
}