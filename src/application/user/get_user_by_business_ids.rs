use crate::domain::{SimplifiedUser, UserRepository, UserRepoError};

#[derive(Clone)]
pub struct GetUserByBusinessIdsUseCase<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> GetUserByBusinessIdsUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, id: &str, business_ids: &[String]) -> Result<SimplifiedUser, UserRepoError> {
        self.repo.find_by_id_and_business_ids(id, business_ids).await
    }
}

