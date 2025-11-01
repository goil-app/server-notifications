use crate::domain::analytics::{NotificationReadRepository, NotificationReadRepoError};
use crate::domain::SimplifiedUser;

#[derive(Clone)]
pub struct GetNotificationReadsUseCase<R: NotificationReadRepository> {
    repo: R,
}

impl<R: NotificationReadRepository> GetNotificationReadsUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, simplified_user: &SimplifiedUser) -> Result<Vec<String>, NotificationReadRepoError> {
        self.repo.find_by_user_id(simplified_user).await
    }
}

