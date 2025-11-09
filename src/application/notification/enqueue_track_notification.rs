use crate::infrastructure::external::queue::{QueueService, TrackNotificationParams, QueueRequestHeaders};

#[derive(Clone)]
#[allow(dead_code)] // Se usa cuando el tracking está activo en el controlador
pub struct EnqueueTrackNotificationUseCase {
    queue_service: QueueService,
}

impl EnqueueTrackNotificationUseCase {
    pub fn new(queue_service: QueueService) -> Self {
        Self { queue_service }
    }

    #[allow(dead_code)] // Se usa cuando el tracking está activo en el controlador
    pub async fn execute(
        &self,
        notification_id: &str,
        account_id: &str,
        business_id: Option<String>,
        session_id: Option<String>,
        headers: QueueRequestHeaders,
    ) -> Result<(), String> {
        let params = TrackNotificationParams {
            id: notification_id.to_string(),
            business_id,
            account_id: Some(account_id.to_string()),
            device_client_type: headers.x_client_device.clone(),
            device_client_model: headers.x_client_device.clone(),
            device_client_os: headers.x_client_os.clone(),
            session_id, // Extraído del token JWT
        };

        self.queue_service
            .enqueue_track_notification(params, headers)
            .await
    }
}

