use crate::domain::queue::{Queue, QueueError, Job, JobOptions};

#[derive(Clone)]
pub struct AddJobUseCase<Q: Queue> {
    queue: Q,
}

impl<Q: Queue> AddJobUseCase<Q> {
    pub fn new(queue: Q) -> Self {
        Self { queue }
    }

    /// Ejecuta el caso de uso para añadir un trabajo a la cola
    pub async fn execute(
        &self,
        name: &str,
        data: serde_json::Value,
        opts: Option<JobOptions>,
    ) -> Result<Job, QueueError> {
        self.queue.add(name, data, opts).await
    }
}

