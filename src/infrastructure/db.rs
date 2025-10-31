use mongodb::{Client, Database};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use std::time::Duration;

pub async fn init_database() -> mongodb::error::Result<Database> {
    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "app".to_string());
    println!("uri: {}", uri);
    println!("db_name: {}", db_name);

    let mut opts = ClientOptions::parse(&uri).await?;
    opts.app_name = Some("server-notifications".to_string());
    opts.server_selection_timeout = Some(Duration::from_secs(3));
    opts.connect_timeout = Some(Duration::from_secs(3));
    // opts.socket_timeout = Some(Duration::from_secs(10));
    opts.max_pool_size = Some(128);
    opts.server_api = Some(ServerApi::builder().version(ServerApiVersion::V1).build());

    let client = Client::with_options(opts)?;
    Ok(client.database(&db_name))
}


