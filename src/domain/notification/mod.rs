use async_trait::async_trait;
use crate::domain::SimplifiedUser;

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub image_paths: Vec<String>,
    pub url: String,
    pub r#type: i32, // r#type porque "type" es una palabra reservada en Rust
    pub payload_type: i32,
    pub is_read: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum NotificationRepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn find_by_id(&self, id: &str, language: &str, business_id: &str) -> Result<Notification, NotificationRepoError>;
    async fn find_users_notifications(&self, users: &[SimplifiedUser], business_ids: &[String]) -> Result<Vec<String>, NotificationRepoError>;
}
