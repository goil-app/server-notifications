use crate::application::GetNotificationReadsUseCase;
use crate::infrastructure::analytics::mongo::MongoNotificationReadRepository;
use crate::infrastructure::db::Databases;

#[derive(Clone)]
pub struct AnalyticsServiceProvider {
    pub get_notification_reads: GetNotificationReadsUseCase<MongoNotificationReadRepository>,
}

impl AnalyticsServiceProvider {
    pub fn new(databases: &Databases) -> Self {
        let notification_read_repo = MongoNotificationReadRepository::new(databases.analytics_db.clone());

        Self {
            get_notification_reads: GetNotificationReadsUseCase::new(notification_read_repo),
        }
    }
}

