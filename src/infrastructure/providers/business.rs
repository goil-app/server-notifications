use crate::application::GetBusinessUseCase;
use crate::infrastructure::business::mongo::MongoBusinessRepository;
use crate::infrastructure::db::Databases;

#[derive(Clone)]
pub struct BusinessServiceProvider {
    pub get_business: GetBusinessUseCase<MongoBusinessRepository>,
}

impl BusinessServiceProvider {
    pub fn new(databases: &Databases) -> Self {
        let business_repo = MongoBusinessRepository::new(databases.client_db.clone());

        Self {
            get_business: GetBusinessUseCase::new(business_repo),
        }
    }
}

