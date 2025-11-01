use mongodb::{Client, Database};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use std::time::Duration;

/// Estructura que contiene todas las bases de datos de MongoDB
#[derive(Clone)]
pub struct Databases {
    pub notifications_db: Database,
    pub account_db: Database,
}

impl Databases {
    /// Inicializa el cliente MongoDB y crea las conexiones a las bases de datos necesarias
    pub async fn init() -> mongodb::error::Result<Self> {
        let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let notifications_db_name = std::env::var("MONGODB_NOTIFICATIONS_DB")
            .unwrap_or_else(|_| "NotificationDB".to_string());
        let account_db_name = std::env::var("MONGODB_ACCOUNT_DB")
            .unwrap_or_else(|_| "AccountDB".to_string());
        
        println!("MongoDB URI: {}", uri);
        println!("Notifications DB: {}", notifications_db_name);
        println!("Account DB: {}", account_db_name);

        let mut opts = ClientOptions::parse(&uri).await?;
        opts.app_name = Some("server-notifications".to_string());
        opts.server_selection_timeout = Some(Duration::from_secs(3));
        opts.connect_timeout = Some(Duration::from_secs(3));
        opts.max_pool_size = Some(128);
        opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

        let client = Client::with_options(opts)?;
        
        Ok(Self {
            notifications_db: client.database(&notifications_db_name),
            account_db: client.database(&account_db_name),
        })
    }
}


