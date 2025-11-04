use redis::aio::ConnectionManager;
use redis::Client;

/// Cliente Redis con conexión async
#[derive(Clone)]
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    /// Crea una nueva instancia del cliente Redis
    pub fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    /// Obtiene una conexión Redis
    pub async fn get_connection(&self) -> Result<ConnectionManager, redis::RedisError> {
        ConnectionManager::new(self.client.clone()).await
    }

    /// Verifica la conexión a Redis
    pub async fn ping(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        redis::cmd("PING").query_async::<String>(&mut conn).await?;
        Ok(())
    }
}

