use crate::domain::{NotificationRepository, NotificationRepoError, SimplifiedUser};

#[derive(Clone)]
pub struct GetUsersNotificationsUseCase<R: NotificationRepository> {
    repo: R,
}

impl<R: NotificationRepository> GetUsersNotificationsUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, users: &[SimplifiedUser], business_ids: &[String]) -> Result<Vec<String>, NotificationRepoError> {
        self.repo.find_users_notifications(users, business_ids).await
    }
}

