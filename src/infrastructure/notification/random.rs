use async_trait::async_trait;
use rand::Rng;

use crate::domain::{Notification, NotificationRepository, RepoError};

#[derive(Clone, Default)]
pub struct RandomNotificationRepository;

impl RandomNotificationRepository {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl NotificationRepository for RandomNotificationRepository {
    async fn find_by_id(&self, _id: &str) -> Result<Notification, RepoError> {
        let mut rng = rand::thread_rng();
        let num: u64 = rng.gen_range(1000..=9999);
        Ok(Notification {
            id: format!("{}", num),
            title: "Random Notification".to_string(),
            message: format!("random-{}", num),
        })
    }
}


