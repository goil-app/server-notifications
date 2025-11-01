use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct SimplifiedUser {
    pub id: String,
    pub phone: String,
    pub creation_date: DateTime<Utc>,
    pub account_type: String,
    pub business_id: String,
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
}