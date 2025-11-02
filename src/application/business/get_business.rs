use crate::domain::{Business, BusinessRepository, BusinessRepoError};

#[derive(Clone)]
pub struct GetBusinessUseCase<R: BusinessRepository> {
    repo: R,
}

impl<R: BusinessRepository> GetBusinessUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &str) -> Result<Business, BusinessRepoError> {
        self.repo.find_by_id(id).await
    }
}

