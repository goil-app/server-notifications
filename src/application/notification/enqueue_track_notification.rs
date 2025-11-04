/// Caso de uso para encolar notificaciones de tracking
/// Encapsula la lógica de comunicación con el servidor de colas externo
#[derive(Clone)]
pub struct EnqueueTrackNotificationUseCase {
    queue_url: String,
}

impl EnqueueTrackNotificationUseCase {
    pub fn new(queue_url: Option<String>) -> Self {
        Self {
            queue_url: queue_url.unwrap_or_else(|| "https://community.goil.app/api/v2/queue".to_string()),
        }
    }

    /// Encola una notificación de tracking de forma asíncrona
    /// Esta función se ejecuta en un tokio::spawn para no bloquear
    pub fn execute_async(&self, params: TrackNotificationParams, auth_header: Option<String>) {
        let queue_url = self.queue_url.clone();
        tokio::spawn(async move {
            let queue_request = serde_json::json!({
                "name": "TRACK_NOTIFICATION",
                "params": {
                    "id": params.id.clone(),
                    "businessId": params.business_id,
                    "accountId": params.account_id,
                    "deviceClientType": params.device_client_type,
                    "deviceClientModel": params.device_client_model,
                    "deviceClientOS": params.device_client_os,
                    "sessionId": params.session_id
                }
            });

            eprintln!("[EnqueueTrackNotificationUseCase] Queue request to {}:", queue_url);
            eprintln!("{}", serde_json::to_string_pretty(&queue_request).unwrap_or_default());

            let client = reqwest::Client::new();
            let mut request_builder = client.post(&queue_url)
                .json(&queue_request)
                .header("x-client-platform", "mobile-platform");
            
            if let Some(auth) = auth_header {
                request_builder = request_builder.header("authorization", auth);
            }
            
            match request_builder.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        eprintln!("[EnqueueTrackNotificationUseCase] Successfully enqueued track notification for id: {}", params.id);
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        eprintln!("[EnqueueTrackNotificationUseCase] Failed to enqueue track notification for id: {}. Status: {}, Error: {}", params.id, status, error_text);
                    }
                }
                Err(e) => {
                    eprintln!("[EnqueueTrackNotificationUseCase] Error enqueuing track notification for id: {}: {:?}", params.id, e);
                }
            }
        });
    }
}

/// Parámetros para encolar una notificación de tracking
#[derive(Clone, Debug)]
pub struct TrackNotificationParams {
    pub id: String,
    pub business_id: String,
    pub account_id: String,
    pub device_client_type: String,
    pub device_client_model: String,
    pub device_client_os: String,
    pub session_id: String,
}

