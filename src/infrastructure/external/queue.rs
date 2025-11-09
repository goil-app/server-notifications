use serde::Serialize;
use std::sync::Arc;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct QueueService {
    client: Arc<Client>,
    queue_url: String,
}

impl QueueService {
    pub fn new() -> Self {
        let client = Arc::new(
            Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client for queue service")
        );
        
        let queue_url = std::env::var("QUEUE_URL")
            .unwrap_or_else(|_| "https://community.goil.app/api/v2/queue".to_string());
        
        Self {
            client,
            queue_url,
        }
    }
}

#[derive(Serialize)]
#[allow(dead_code)] // Se usa cuando el tracking está activo
struct QueuePayload {
    name: String,
    params: TrackNotificationParams,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackNotificationParams {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_client_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_client_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_client_os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug)]
pub struct QueueRequestHeaders {
    pub authorization: Option<String>,
    pub x_client_platform: Option<String>,
    pub x_client_os: Option<String>,
    pub x_client_device: Option<String>,
    pub x_client_id: Option<String>,
}

impl QueueService {
    pub async fn enqueue_track_notification(
        &self,
        params: TrackNotificationParams,
        headers: QueueRequestHeaders,
    ) -> Result<(), String> {
        let payload = QueuePayload {
            name: "TRACK_NOTIFICATION".to_string(),
            params,
        };

        let mut request = self.client
            .post(&self.queue_url)
            .json(&payload);

        // Añadir headers del request original
        if let Some(auth) = headers.authorization {
            request = request.header("authorization", auth);
        }
        if let Some(platform) = headers.x_client_platform {
            request = request.header("x-client-platform", platform);
        }
        if let Some(os) = headers.x_client_os {
            request = request.header("x-client-os", os);
        }
        if let Some(device) = headers.x_client_device {
            request = request.header("x-client-device", device);
        }
        if let Some(client_id) = headers.x_client_id {
            request = request.header("x-client-id", client_id);
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("Queue API returned error {}: {}", status, error_text))
                }
            }
            Err(e) => Err(format!("Failed to send request to queue API: {}", e)),
        }
    }
}

