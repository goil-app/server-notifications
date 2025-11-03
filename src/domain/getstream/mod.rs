use async_trait::async_trait;

use crate::domain::Notification;

#[derive(thiserror::Error, Debug)]
pub enum GetStreamRepoError {
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait GetStreamRepository: Send + Sync {
    /// Obtiene un mensaje enviado desde GetStream por su UUID
    async fn find_message_by_uuid(&self, id: &str, user_id: &str, language: &str, business_id: &str) -> Result<Notification, GetStreamRepoError>;
    /// Obtiene el número de notificaciones no leídas del usuario en GetStream
    async fn get_unread_count(&self, user_id: &str) -> Result<i32, GetStreamRepoError>;
}


