use async_trait::async_trait;

#[derive(thiserror::Error, Debug)]
pub enum NotificationReadRepoError {
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait NotificationReadRepository: Send + Sync {
    async fn find_by_phone_and_business_ids(&self, phone: &str, business_ids: &[String]) -> Result<Vec<String>, NotificationReadRepoError>;
}

