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
        // Pool optimizado para 4 vCPU y 8GB RAM:
        // - Con 4 workers: total = 4 × 50 = 200 conexiones (dentro de límites razonables)
        // - Para 2000+ req/s: ~500 req/s por worker, pool de 50 es adecuado
        // - min_pool_size aumentado para mantener conexiones calientes y reducir latencia
        // - max_idle_time moderado para balance entre rendimiento y uso de recursos
        opts.max_pool_size = Some(50); // Aumentado para aprovechar más recursos (4vCPU, 8GB RAM)
        opts.min_pool_size = Some(10); // Aumentado para mantener más conexiones calientes
        opts.max_idle_time = Some(Duration::from_secs(30)); // Tiempo moderado para liberar recursos
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


