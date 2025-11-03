use crate::domain::getstream::{GetStreamRepository, GetStreamRepoError};

#[derive(Clone)]
pub struct GetGetStreamUnreadCountUseCase<R: GetStreamRepository> {
    repo: R,
}

impl<R: GetStreamRepository> GetGetStreamUnreadCountUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, user_id: &str) -> Result<i32, GetStreamRepoError> {
        self.repo.get_unread_count(user_id).await
    }
}


