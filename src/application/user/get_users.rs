use crate::domain::{SimplifiedUser, UserRepository, UserRepoError};

#[derive(Clone)]
pub struct GetUsersUseCase<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> GetUsersUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, phone: &str, business_ids: &[String]) -> Result<Vec<SimplifiedUser>, UserRepoError> {
        self.repo.find_by_phone_and_business_ids(phone, business_ids).await
    }
}

