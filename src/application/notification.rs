use crate::domain::{Notification, NotificationRepository, RepoError};

#[derive(Clone)]
pub struct GetNotificationUseCase<R: NotificationRepository> {
    repo: R,
}

impl<R: NotificationRepository> GetNotificationUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, id: &str) -> Result<Notification, RepoError> {
        self.repo.find_by_id(id).await
    }
}

