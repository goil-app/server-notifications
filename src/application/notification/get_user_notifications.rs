use crate::domain::{NotificationRepository, NotificationRepoError, SimplifiedUser};

#[derive(Clone)]
pub struct GetUserNotificationsUseCase<R: NotificationRepository> {
    repo: R,
}

impl<R: NotificationRepository> GetUserNotificationsUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, simplified_user: &SimplifiedUser) -> Result<Vec<String>, NotificationRepoError> {
        self.repo.find_user_notifications(simplified_user).await
    }
}

