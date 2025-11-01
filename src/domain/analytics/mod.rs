use async_trait::async_trait;
use crate::domain::SimplifiedUser;

#[derive(Clone, Debug)]
#[allow(dead_code)] // Estructura de dominio, puede usarse en el futuro
pub struct NotificationRead {
    pub notification_id: String,
    pub account_id: Option<String>,
    pub phone: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum NotificationReadRepoError {
    #[error("not found")]
    #[allow(dead_code)] // Variante de error, puede usarse en el futuro
    NotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait NotificationReadRepository: Send + Sync {
    async fn find_by_user_id(&self, simplified_user: &SimplifiedUser) -> Result<Vec<String>, NotificationReadRepoError>;
}

