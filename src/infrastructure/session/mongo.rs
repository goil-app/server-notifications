use async_trait::async_trait;
use mongodb::Database;
use mongodb::bson::{doc, oid::ObjectId, Document};

use crate::domain::{Session, SessionRepository, SessionRepoError};
use crate::mappers::session::doc_to_domain;

#[derive(Clone)]
pub struct MongoSessionRepository {
    db: Database,
}

impl MongoSessionRepository {
    pub fn new(db: Database) -> Self { Self { db } }
}

#[async_trait]
impl SessionRepository for MongoSessionRepository {
    async fn find_by_id(&self, id: &str, business_id: &str) -> Result<Session, SessionRepoError> {
        let bid = ObjectId::parse_str(business_id).map_err(|e| SessionRepoError::Unexpected(e.to_string()))?;
        let coll = self.db.collection::<Document>("AccountSessionInfo");
        let filter = doc! { 
            "sessionId": id,
            "businessId": bid 
        };
        let projection = doc! { "language": 1 };
        let find_options = mongodb::options::FindOneOptions::builder()
            .projection(projection)
            .build();
        let doc = match coll.find_one(filter)
            .with_options(find_options)
            .await.map_err(|e| SessionRepoError::Unexpected(e.to_string()))? {
            Some(d) => d,
            None => return Err(SessionRepoError::NotFound),
        };
        doc_to_domain(doc)
    }
}