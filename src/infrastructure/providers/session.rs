use crate::application::GetSessionUseCase;
use crate::infrastructure::session::mongo::MongoSessionRepository;
use crate::infrastructure::db::Databases;

#[derive(Clone)]
pub struct SessionServiceProvider {
    pub get_session: GetSessionUseCase<MongoSessionRepository>,
}

impl SessionServiceProvider {
    pub fn new(databases: &Databases) -> Self {
        let session_repo = MongoSessionRepository::new(databases.account_db.clone());

        Self {
            get_session: GetSessionUseCase::new(session_repo),
        }
    }
}

