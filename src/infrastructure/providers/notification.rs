use crate::application::{GetNotificationUseCase, GetUsersNotificationsUseCase, GetGetStreamMessageUseCase, GetGetStreamUnreadCountUseCase};
use crate::infrastructure::notification::mongo::MongoNotificationRepository;
use crate::infrastructure::external::getstream::HttpGetStreamRepository;
use crate::infrastructure::db::Databases;

#[derive(Clone)]
pub struct NotificationServiceProvider {
    pub get_notification: GetNotificationUseCase<MongoNotificationRepository>,
    pub get_users_notifications: GetUsersNotificationsUseCase<MongoNotificationRepository>,
    pub get_getstream_message: GetGetStreamMessageUseCase<HttpGetStreamRepository>,
    pub get_getstream_unread_count: GetGetStreamUnreadCountUseCase<HttpGetStreamRepository>,
}

impl NotificationServiceProvider {
    pub fn new(databases: &Databases) -> Self {
        let notification_repo = MongoNotificationRepository::new(databases.notifications_db.clone());
        let external_repo = HttpGetStreamRepository::default();

        Self {
            get_notification: GetNotificationUseCase::new(notification_repo.clone()),
            get_users_notifications: GetUsersNotificationsUseCase::new(notification_repo),
            get_getstream_message: GetGetStreamMessageUseCase::new(external_repo.clone()),
            get_getstream_unread_count: GetGetStreamUnreadCountUseCase::new(external_repo),
        }
    }
}

