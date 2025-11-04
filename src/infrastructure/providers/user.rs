use crate::application::{GetUserUseCase, GetUserByBusinessIdsUseCase, GetUsersUseCase};
use crate::infrastructure::user::mongo::MongoUserRepository;
use crate::infrastructure::db::Databases;

#[derive(Clone)]
pub struct UserServiceProvider {
    pub get_user: GetUserUseCase<MongoUserRepository>,
    pub get_user_by_business_ids: GetUserByBusinessIdsUseCase<MongoUserRepository>,
    pub get_users: GetUsersUseCase<MongoUserRepository>,
}

impl UserServiceProvider {
    pub fn new(databases: &Databases) -> Self {
        let user_repo = MongoUserRepository::new(databases.account_db.clone());

        Self {
            get_user: GetUserUseCase::new(user_repo.clone()),
            get_user_by_business_ids: GetUserByBusinessIdsUseCase::new(user_repo.clone()),
            get_users: GetUsersUseCase::new(user_repo),
        }
    }
}

