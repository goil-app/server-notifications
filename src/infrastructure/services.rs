use crate::infrastructure::providers::{
    NotificationServiceProvider,
    UserServiceProvider,
    SessionServiceProvider,
    BusinessServiceProvider,
    AnalyticsServiceProvider,
    StorageServiceProvider,
    RedisServiceProvider,
    QueueServiceProvider,
};
use crate::infrastructure::db::Databases;

/// Contenedor centralizado de todos los servicios de la aplicación
/// Organiza los servicios por categoría usando service providers
#[derive(Clone)]
pub struct AppServices {
    pub notification: NotificationServiceProvider,
    pub user: UserServiceProvider,
    pub session: SessionServiceProvider,
    pub business: BusinessServiceProvider,
    pub analytics: AnalyticsServiceProvider,
    pub storage: StorageServiceProvider,
    pub redis: RedisServiceProvider,
    pub queue: QueueServiceProvider,
    pub queue_track_notification: QueueServiceProvider,
}

impl AppServices {
    /// Crea todos los servicios a partir de las bases de datos usando service providers
    pub async fn new(databases: &Databases) -> Result<Self, String> {
        eprintln!("[AppServices] Initializing all service providers...");
        
        let notification_provider = NotificationServiceProvider::new(databases);
        let user_provider = UserServiceProvider::new(databases);
        let session_provider = SessionServiceProvider::new(databases);
        let business_provider = BusinessServiceProvider::new(databases);
        let analytics_provider = AnalyticsServiceProvider::new(databases);
        let storage_provider = StorageServiceProvider::new().await?;
        let redis_provider = RedisServiceProvider::new().await?;
        let queue_provider = QueueServiceProvider::new(redis_provider.redis_client.clone(), None);
        let queue_track_notification_provider = QueueServiceProvider::new(redis_provider.redis_client.clone(), Some("QUEUE_TRACK_NOTIFICATION".to_string()));

        eprintln!("[AppServices] All service providers initialized successfully");

        Ok(Self {
            notification: notification_provider,
            user: user_provider,
            session: session_provider,
            business: business_provider,
            analytics: analytics_provider,
            storage: storage_provider,
            redis: redis_provider,
            queue: queue_provider,
            queue_track_notification: queue_track_notification_provider,
        })
    }
}

