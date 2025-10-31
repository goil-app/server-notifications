use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { timestamp: chrono::Utc::now().timestamp_millis(), data: Some(data), message: None }
    }
}

impl ApiResponse<()> {
    pub fn error(msg: impl Into<String>) -> Self {
        Self { timestamp: chrono::Utc::now().timestamp_millis(), data: None, message: Some(msg.into()) }
    }
}


