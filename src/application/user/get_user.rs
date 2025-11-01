use crate::domain::{SimplifiedUser, UserRepository, UserRepoError};

#[derive(Clone)]
pub struct GetUserUseCase<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, id: &str, business_id: &str) -> Result<SimplifiedUser, UserRepoError> {
        self.repo.find_simplified_by_id(id, business_id).await
    }
}

