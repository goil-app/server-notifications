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
    /// 
    /// # Parámetros
    /// - `max_pool_size`: Número máximo de conexiones en el pool por worker (opcional)
    /// - `min_pool_size`: Número mínimo de conexiones en el pool (opcional)
    pub async fn init_with_pool_config(
        max_pool_size: Option<u32>,
        min_pool_size: Option<u32>,
    ) -> mongodb::error::Result<Self> {
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
        // Timeouts optimizados para alto rendimiento y estabilidad
        opts.server_selection_timeout = Some(Duration::from_secs(5)); // Tiempo suficiente para seleccionar servidor
        opts.connect_timeout = Some(Duration::from_secs(5)); // Tiempo suficiente para conectar
        
        // Configurar pool size dinámicamente o usar valores por defecto
        opts.max_pool_size = max_pool_size.or(Some(200)); // Por defecto: 200 conexiones por worker (alto rendimiento)
        opts.min_pool_size = min_pool_size.or(Some(50)); // Por defecto: 50 conexiones mínimas (mantener calientes)
        opts.max_idle_time = Some(Duration::from_secs(120)); // Más tiempo idle para evitar recrear conexiones
        opts.max_connecting = Some((max_pool_size.unwrap_or(200) / 4).max(20)); // Limitar conexiones simultáneas en establecimiento
        
        // Heartbeat para mantener conexiones vivas y evitar timeouts
        opts.heartbeat_freq = Some(Duration::from_secs(10)); // Heartbeat cada 10 segundos para mantener conexiones activas
        opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

        let client = Client::with_options(opts)?;
        
        Ok(Self {
            notifications_db: client.database(&notifications_db_name),
            account_db: client.database(&account_db_name),
            analytics_db: client.database(&analytics_db_name),
            client_db: client.database(&client_db_name),
        })
    }

    /// Inicializa el cliente MongoDB con configuración por defecto
    /// Para configuración personalizada, usar `init_with_pool_config`
    #[allow(dead_code)] // Mantenido para compatibilidad futura
    pub async fn init() -> mongodb::error::Result<Self> {
        Self::init_with_pool_config(None, None).await
    }
}


