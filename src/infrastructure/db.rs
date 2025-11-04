use mongodb::{Client, Database};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use std::time::Duration;

/// Estructura que contiene todas las bases de datos de MongoDB
#[derive(Clone)]
pub struct Databases {
    pub notifications_db: Database,
    pub account_db: Database,
    pub analytics_db: Database,
    pub client_db: Database,
}

impl Databases {
    /// Inicializa el cliente MongoDB y crea las conexiones a las bases de datos necesarias
    pub async fn init() -> mongodb::error::Result<Self> {
        let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let notifications_db_name = std::env::var("MONGODB_NOTIFICATIONS_DB")
            .unwrap_or_else(|_| "NotificationDB".to_string());
        let account_db_name = std::env::var("MONGODB_ACCOUNT_DB")
            .unwrap_or_else(|_| "AccountDB".to_string());
        let analytics_db_name = std::env::var("MONGODB_ANALYTICS_DB")
            .unwrap_or_else(|_| "AnalyticsDB".to_string());
        let client_db_name = std::env::var("MONGODB_CLIENT_DB")
            .unwrap_or_else(|_| "ClientDB".to_string());
        
        let parsed = ClientOptions::parse(&uri).await?;
        let mut opts = parsed;
        opts.app_name = Some("server-notifications".to_string());
        opts.server_selection_timeout = Some(Duration::from_secs(3));
        opts.connect_timeout = Some(Duration::from_secs(3));
        // Pool optimizado para 2 vCPU y 4GB RAM:
        // - Con 2 workers: total = 2 × 30 = 60 conexiones (dentro de límites razonables)
        // - Para 1000 req/s: ~500 req/s por worker, pool de 30 es más que suficiente
        // - min_pool_size pequeño para ahorrar memoria
        // - max_idle_time corto para liberar recursos rápidamente
        opts.max_pool_size = Some(30); // Reducido para recursos limitados (2vCPU, 4GB RAM)
        opts.min_pool_size = Some(3);  // Mínimo para mantener latencia baja
        opts.max_idle_time = Some(Duration::from_secs(20)); // Cerrar conexiones inactivas más rápido
        opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

        let client = Client::with_options(opts)?;
        
        Ok(Self {
            notifications_db: client.database(&notifications_db_name),
            account_db: client.database(&account_db_name),
            analytics_db: client.database(&analytics_db_name),
            client_db: client.database(&client_db_name),
        })
    }
}


