use crate::infrastructure::redis::RedisClient;

#[derive(Clone)]
pub struct ConnectRedisUseCase {
    redis_client: RedisClient,
}

impl ConnectRedisUseCase {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    /// Ejecuta la conexión a Redis y verifica que esté funcionando
    pub async fn execute(&self) -> Result<(), String> {
        self.redis_client
            .ping()
            .await
            .map_err(|e| format!("Error connecting to Redis: {}", e))?;
        Ok(())
    }
}

