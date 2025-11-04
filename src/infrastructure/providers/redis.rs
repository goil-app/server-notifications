use crate::application::redis::ConnectRedisUseCase;
use crate::infrastructure::redis::RedisClient;

#[derive(Clone)]
pub struct RedisServiceProvider {
    pub connect_redis: ConnectRedisUseCase,
}

impl RedisServiceProvider {
    pub async fn new() -> Result<Self, String> {
        // Leer los campos individuales del .env
        let url = std::env::var("REDIS_URL")
            .map_err(|_| "REDIS_HOST environment variable not set")?;

        let redis_client = RedisClient::new(&url)
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;

        let connect_redis = ConnectRedisUseCase::new(redis_client);
        
        // Verificar la conexión al inicializar
        connect_redis.execute().await
            .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

        Ok(Self {
            connect_redis,
        })
    }
}

