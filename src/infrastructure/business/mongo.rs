use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOneOptions;

use crate::domain::{Business, BusinessRepository, BusinessRepoError};
use crate::mappers::business::doc_to_domain;

#[derive(Clone)]
pub struct MongoBusinessRepository {
    db: Database,
}

impl MongoBusinessRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BusinessRepository for MongoBusinessRepository {
    async fn find_by_id(&self, id: &str) -> Result<Business, BusinessRepoError> {
        let oid = ObjectId::parse_str(id).map_err(|e| BusinessRepoError::Unexpected(e.to_string()))?;
        
        let options = FindOneOptions::builder()
            .projection(doc! { "name": 1 })
            .build();
        
        let coll = self.db.collection::<Document>("Business");
        let doc = match coll
            .find_one(doc! { "_id": oid })
            .with_options(options)
            .await
            .map_err(|e| BusinessRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(BusinessRepoError::NotFound),
        };
        
        doc_to_domain(doc)
    }
}

