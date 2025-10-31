use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Linked {
    pub linked_type: i32,
    pub object_id: Option<String>,
    pub object: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub image_paths: Vec<String>,
    pub url: String,
    pub user_targets: Vec<String>,
    pub topic: Option<String>,
    pub notification_type: i32,
    pub creation_date: DateTime<Utc>,
    pub payload_type: i32,
    pub business_id: Option<String>,
    pub linked: Linked,
    pub browser: i32,
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

