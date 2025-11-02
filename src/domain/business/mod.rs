use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct Business {
    pub name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum BusinessRepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait BusinessRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Business, BusinessRepoError>;
}

