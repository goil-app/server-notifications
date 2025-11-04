use crate::domain::queue::{Queue, QueueError, Job, JobOptions};
use crate::infrastructure::redis::RedisClient;
use redis::AsyncCommands;
use serde_json;

/// Implementación de Queue usando Redis
#[derive(Clone)]
pub struct RedisQueue {
    redis_client: RedisClient,
    queue_name: String,
}

impl RedisQueue {
    pub fn new(redis_client: RedisClient, queue_name: String) -> Self {
        Self {
            redis_client,
            queue_name,
        }
    }

    fn job_key(&self, job_id: &str) -> String {
        format!("bull:{}:{}", self.queue_name, job_id)
    }

    fn wait_key(&self) -> String {
        format!("bull:{}:wait", self.queue_name)
    }

    fn delayed_key(&self) -> String {
        format!("bull:{}:delayed", self.queue_name)
    }

    fn meta_key(&self) -> String {
        format!("bull:{}:meta", self.queue_name)
    }
}

#[async_trait::async_trait]
impl Queue for RedisQueue {
    async fn add(&self, name: &str, data: serde_json::Value, opts: Option<JobOptions>) -> Result<Job, QueueError> {
        println!("[RedisQueue::add] Adding job to queue: {}", name);
        println!("[RedisQueue::add] Data: {:?}", data);
        println!("[RedisQueue::add] Options: {:?}", opts);
        println!("[RedisQueue::add] Queue name: {}", self.queue_name);

        let opts = opts.unwrap_or_default();
        let job_id = uuid::Uuid::new_v4().to_string();
        
        let job = Job {
            id: job_id.clone(),
            name: name.to_string(),
            data,
            opts: opts.clone(),
        };

        let mut conn = self.redis_client
            .get_connection()
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to get Redis connection: {}", e)))?;

        // Serializar el trabajo
        let job_json = serde_json::to_string(&job)
            .map_err(|e| QueueError::Unexpected(format!("Failed to serialize job: {}", e)))?;

        let job_key = self.job_key(&job.id);
        let wait_key = self.wait_key();
        let delayed_key = self.delayed_key();
        let meta_key = self.meta_key();

        // Guardar el trabajo en Redis con el formato de BullMQ
        conn.set::<_, _, ()>(job_key.clone(), job_json.clone())
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to save job: {}", e)))?;

        // Inicializar metadata de la cola si no existe (formato BullMQ)
        let exists: bool = conn.exists(&meta_key)
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to check queue metadata: {}", e)))?;
        
        if !exists {
            let meta_data = serde_json::json!({
                "name": self.queue_name,
                "ns": "bull"
            });
            conn.set::<_, _, ()>(&meta_key, serde_json::to_string(&meta_data).unwrap_or_default())
                .await
                .map_err(|e| QueueError::Queue(format!("Failed to create queue metadata: {}", e)))?;
        }

        // BullMQ usa estructuras diferentes según el tipo de trabajo:
        // - wait: LISTA para trabajos inmediatos
        // - delayed: SORTED SET para trabajos con delay
        
        // Limpiar keys si existen con el tipo incorrecto (por compatibilidad con implementaciones anteriores)
        // Verificar el tipo de wait y eliminar si no es lista
        let wait_exists: bool = conn.exists(&wait_key)
            .await
            .unwrap_or(false);
        if wait_exists {
            // Verificar el tipo usando el comando TYPE de Redis
            let key_type: String = redis::cmd("TYPE")
                .arg(&wait_key)
                .query_async::<_, String>(&mut conn)
                .await
                .unwrap_or_else(|_| "none".to_string());
            if key_type != "list" {
                // Eliminar la key si no es una lista
                let _: Result<(), _> = conn.del::<_, ()>(&wait_key).await;
            }
        }
        
        // Verificar y limpiar delayed si existe como lista
        let delayed_exists: bool = conn.exists(&delayed_key)
            .await
            .unwrap_or(false);
        if delayed_exists {
            let key_type: String = redis::cmd("TYPE")
                .arg(&delayed_key)
                .query_async::<_, String>(&mut conn)
                .await
                .unwrap_or_else(|_| "none".to_string());
            if key_type != "zset" {
                // Eliminar la key si no es un sorted set
                let _: Result<(), _> = conn.del::<_, ()>(&delayed_key).await;
            }
        }
        
        if let Some(delay) = opts.delay {
            // Trabajos con delay van a "delayed" como sorted set
            let timestamp = chrono::Utc::now().timestamp() as u64 + delay;
            conn.zadd::<_, _, _, ()>(&delayed_key, job_id.clone(), timestamp as f64)
                .await
                .map_err(|e| QueueError::Queue(format!("Failed to add job to delayed queue: {}", e)))?;
        } else {
            // Trabajos inmediatos van a "wait" como lista
            conn.lpush::<_, _, ()>(&wait_key, job_id.clone())
                .await
                .map_err(|e| QueueError::Queue(format!("Failed to add job to wait queue: {}", e)))?;
        }

        Ok(job)
    }

    async fn get_job(&self, job_id: &str) -> Result<Job, QueueError> {
        let mut conn = self.redis_client
            .get_connection()
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to get Redis connection: {}", e)))?;

        let job_key = self.job_key(job_id);
        let job_json: Option<String> = conn.get(job_key)
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to get job: {}", e)))?;

        match job_json {
            Some(json) => {
                serde_json::from_str(&json)
                    .map_err(|e| QueueError::Unexpected(format!("Failed to deserialize job: {}", e)))
            }
            None => Err(QueueError::JobNotFound),
        }
    }

    async fn remove(&self, job_id: &str) -> Result<(), QueueError> {
        let mut conn = self.redis_client
            .get_connection()
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to get Redis connection: {}", e)))?;

        let job_key = self.job_key(job_id);
        let wait_key = self.wait_key();
        let delayed_key = self.delayed_key();

        // Eliminar el trabajo
        conn.del::<_, ()>(job_key)
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to delete job: {}", e)))?;

        // Eliminar de la cola wait (lista)
        conn.lrem::<_, _, ()>(&wait_key, 0, job_id)
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to remove job from wait queue: {}", e)))?;

        // Eliminar de la cola delayed (sorted set)
        conn.zrem::<_, _, ()>(&delayed_key, job_id)
            .await
            .map_err(|e| QueueError::Queue(format!("Failed to remove job from delayed queue: {}", e)))?;

        Ok(())
    }
}

