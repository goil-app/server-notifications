use crate::application::{GetNotificationUseCase, GetSessionUseCase, GetUserUseCase, GetUserByBusinessIdsUseCase, GetUsersUseCase, GetUsersNotificationsUseCase, GetNotificationReadsUseCase, GetBusinessUseCase};
use crate::infrastructure::notification::mongo::MongoNotificationRepository;
use crate::infrastructure::session::mongo::MongoSessionRepository;
use crate::infrastructure::user::mongo::MongoUserRepository;
use crate::infrastructure::analytics::mongo::MongoNotificationReadRepository;
use crate::infrastructure::business::mongo::MongoBusinessRepository;
use crate::infrastructure::db::Databases;
use crate::infrastructure::s3::S3UrlSigner;

/// Contenedor centralizado de todos los servicios de la aplicación
/// Encapsula la creación de repositorios y use cases
#[derive(Clone)]
pub struct AppServices {
    pub get_notification: GetNotificationUseCase<MongoNotificationRepository>,
    pub get_users_notifications: GetUsersNotificationsUseCase<MongoNotificationRepository>,
    pub get_session: GetSessionUseCase<MongoSessionRepository>,
    pub get_user: GetUserUseCase<MongoUserRepository>,
    pub get_user_by_business_ids: GetUserByBusinessIdsUseCase<MongoUserRepository>,
    pub get_users: GetUsersUseCase<MongoUserRepository>,
    pub get_notification_reads: GetNotificationReadsUseCase<MongoNotificationReadRepository>,
    pub get_business: GetBusinessUseCase<MongoBusinessRepository>,
    pub s3_signer: S3UrlSigner,
}

impl AppServices {
    /// Crea todos los servicios a partir de las bases de datos
    pub async fn new(databases: &Databases) -> Result<Self, String> {
        let notification_repo = MongoNotificationRepository::new(databases.notifications_db.clone());
        let session_repo = MongoSessionRepository::new(databases.account_db.clone());
        let user_repo = MongoUserRepository::new(databases.account_db.clone());
        let notification_read_repo = MongoNotificationReadRepository::new(databases.analytics_db.clone());
        let business_repo = MongoBusinessRepository::new(databases.client_db.clone());
        let s3_signer = S3UrlSigner::new().await?;

        Ok(Self {
            get_notification: GetNotificationUseCase::new(notification_repo.clone()),
            get_users_notifications: GetUsersNotificationsUseCase::new(notification_repo),
            get_session: GetSessionUseCase::new(session_repo),
            get_user: GetUserUseCase::new(user_repo.clone()),
            get_user_by_business_ids: GetUserByBusinessIdsUseCase::new(user_repo.clone()),
            get_users: GetUsersUseCase::new(user_repo),
            get_notification_reads: GetNotificationReadsUseCase::new(notification_read_repo),
            get_business: GetBusinessUseCase::new(business_repo),
            s3_signer,
        })
    }
}

