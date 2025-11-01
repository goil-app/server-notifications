use crate::domain::{Notification, NotificationRepository, NotificationRepoError};

#[derive(Clone)]
pub struct GetNotificationUseCase<R: NotificationRepository> {
    repo: R,
}

impl<R: NotificationRepository> GetNotificationUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, id: &str, language: &str, business_id: &str) -> Result<Notification, NotificationRepoError> {
        self.repo.find_by_id(id, language, business_id).await
    }
}

