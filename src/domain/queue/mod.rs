use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Representa un trabajo en la cola
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub data: serde_json::Value,
    pub opts: JobOptions,
}

/// Opciones de configuración para un trabajo
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobOptions {
    /// Tiempo de espera antes de procesar el trabajo (en segundos)
    pub delay: Option<u64>,
    /// Intentos máximos si falla
    pub attempts: Option<u32>,
    /// Tiempo de expiración del trabajo (en segundos)
    pub timeout: Option<u64>,
    /// Prioridad del trabajo (mayor número = mayor prioridad)
    pub priority: Option<i32>,
}

impl Default for JobOptions {
    fn default() -> Self {
        Self {
            delay: None,
            attempts: Some(3),
            timeout: None,
            priority: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QueueError {
    #[error("queue error: {0}")]
    Queue(String),
    #[error("job not found")]
    JobNotFound,
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

#[async_trait]
pub trait Queue: Send + Sync {
    /// Encola un nuevo trabajo
    async fn add(&self, name: &str, data: serde_json::Value, opts: Option<JobOptions>) -> Result<Job, QueueError>;
    
    /// Obtiene un trabajo por su ID
    async fn get_job(&self, job_id: &str) -> Result<Job, QueueError>;
    
    /// Elimina un trabajo de la cola
    async fn remove(&self, job_id: &str) -> Result<(), QueueError>;
}

