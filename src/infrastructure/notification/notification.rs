use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::domain::{Notification, NotificationRepository, RepoError};

#[derive(Clone)]
pub struct MockNotificationRepository {
    items: Arc<HashMap<String, Notification>>,
}

impl MockNotificationRepository {
    pub fn with_seed(seed: Vec<Notification>) -> Self {
        let mut map = HashMap::new();
        for n in seed { map.insert(n.id.clone(), n); }
        Self { items: Arc::new(map) }
    }
}

#[async_trait]
impl NotificationRepository for MockNotificationRepository {
    async fn find_by_id(&self, id: &str) -> Result<Notification, RepoError> {
        self.items.get(id).cloned().ok_or(RepoError::NotFound)
    }
}

