use crate::domain::analytics::{NotificationReadRepository, NotificationReadRepoError};

#[derive(Clone)]
pub struct GetNotificationReadsUseCase<R: NotificationReadRepository> {
    repo: R,
}

impl<R: NotificationReadRepository> GetNotificationReadsUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, phone: &str, business_ids: &[String]) -> Result<Vec<String>, NotificationReadRepoError> {
        self.repo.find_by_phone_and_business_ids(phone, business_ids).await
    }
}

