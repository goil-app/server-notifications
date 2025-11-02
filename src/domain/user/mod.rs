use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct SimplifiedUser {
    pub id: String,
    pub phone: String,
    pub creation_date: DateTime<Utc>,
    pub account_type: String,
}

#[derive(thiserror::Error, Debug)]
pub enum UserRepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_simplified_by_id(&self, id: &str, business_id: &str) -> Result<SimplifiedUser, UserRepoError>;
    async fn find_by_id_and_business_ids(&self, id: &str, business_ids: &[String]) -> Result<SimplifiedUser, UserRepoError>;
    async fn find_by_phone_and_business_ids(&self, phone: &str, business_ids: &[String]) -> Result<Vec<SimplifiedUser>, UserRepoError>;
}

