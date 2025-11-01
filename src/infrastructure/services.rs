use crate::application::{GetNotificationUseCase, GetSessionUseCase, GetUserUseCase, GetUserNotificationsUseCase};
use crate::infrastructure::notification::mongo::MongoNotificationRepository;
use crate::infrastructure::session::mongo::MongoSessionRepository;
use crate::infrastructure::user::mongo::MongoUserRepository;
use crate::infrastructure::db::Databases;

/// Contenedor centralizado de todos los servicios de la aplicación
/// Encapsula la creación de repositorios y use cases
#[derive(Clone)]
pub struct AppServices {
    pub get_notification: GetNotificationUseCase<MongoNotificationRepository>,
    pub get_user_notifications: GetUserNotificationsUseCase<MongoNotificationRepository>,
    pub get_session: GetSessionUseCase<MongoSessionRepository>,
    pub get_user: GetUserUseCase<MongoUserRepository>,
}

impl AppServices {
    /// Crea todos los servicios a partir de las bases de datos
    pub fn new(databases: &Databases) -> Self {
        let notification_repo = MongoNotificationRepository::new(databases.notifications_db.clone());
        let session_repo = MongoSessionRepository::new(databases.account_db.clone());
        let user_repo = MongoUserRepository::new(databases.account_db.clone());

        Self {
            get_notification: GetNotificationUseCase::new(notification_repo.clone()),
            get_user_notifications: GetUserNotificationsUseCase::new(notification_repo),
            get_session: GetSessionUseCase::new(session_repo),
            get_user: GetUserUseCase::new(user_repo),
        }
    }
}

