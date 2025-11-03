use async_trait::async_trait;

use crate::domain::Notification;

#[derive(thiserror::Error, Debug)]
pub enum GetStreamRepoError {
    #[error("not found")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait GetStreamRepository: Send + Sync {
    /// Obtiene un mensaje enviado desde GetStream por su UUID
    async fn find_message_by_uuid(&self, id: &str, language: &str, business_id: &str) -> Result<Notification, GetStreamRepoError>;
}


