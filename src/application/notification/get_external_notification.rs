use crate::domain::Notification;
use crate::domain::getstream::{GetStreamRepository, GetStreamRepoError};

#[derive(Clone)]
pub struct GetGetStreamMessageUseCase<R: GetStreamRepository> {
    repo: R,
}

impl<R: GetStreamRepository> GetGetStreamMessageUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, id: &str, user_id: &str, language: &str, business_id: &str) -> Result<Notification, GetStreamRepoError> {
        self.repo.find_message_by_uuid(id, user_id, language, business_id).await
    }
}


