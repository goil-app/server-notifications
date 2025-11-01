use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub language: String,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionRepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Session, SessionRepoError>;
}