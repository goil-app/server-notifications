use crate::infrastructure::providers::{
    NotificationServiceProvider,
    UserServiceProvider,
    SessionServiceProvider,
    BusinessServiceProvider,
    AnalyticsServiceProvider,
    StorageServiceProvider,
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

        eprintln!("[AppServices] All service providers initialized successfully");

        Ok(Self {
            notification: notification_provider,
            user: user_provider,
            session: session_provider,
            business: business_provider,
            analytics: analytics_provider,
            storage: storage_provider,
        })
    }
}

