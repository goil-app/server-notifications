use crate::application::queue::AddJobUseCase;
use crate::infrastructure::queue::RedisQueue;
use crate::infrastructure::redis::RedisClient;

#[derive(Clone)]
pub struct QueueServiceProvider {
    pub add_job: AddJobUseCase<RedisQueue>,
}

impl QueueServiceProvider {
    pub fn new(redis_client: RedisClient, queue_name: Option<String>) -> Self {
        let queue_name = queue_name.unwrap_or_else(|| "default".to_string());
        let redis_queue = RedisQueue::new(redis_client, queue_name);

        Self {
            add_job: AddJobUseCase::new(redis_queue),
        }
    }
}

