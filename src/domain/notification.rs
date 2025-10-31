use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Notification, RepoError>;
}

