use async_trait::async_trait;
use rand::Rng;
use chrono::Utc;

use crate::domain::{Notification, Linked, NotificationRepository, RepoError};

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
            body: format!("random-{}", num),
            image_paths: vec![],
            url: String::new(),
            user_targets: vec![],
            topic: None,
            notification_type: 1,
            creation_date: Utc::now(),
            payload_type: 1,
            business_id: None,
            linked: Linked { linked_type: 0, object_id: None, object: None },
            browser: 2,
        })
    }
}


