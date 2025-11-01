use crate::domain::{Session, SessionRepository, SessionRepoError};

#[derive(Clone)]
pub struct GetSessionUseCase<R: SessionRepository> {
    repo: R,
}

impl<R: SessionRepository> GetSessionUseCase<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn execute(&self, session_id: &str) -> Result<Session, SessionRepoError> {
        self.repo.find_by_id(session_id).await
    }
}