use actix_web::HttpRequest;

/// Extrae los headers relevantes de la petición HTTP para el tracking de notificaciones
#[derive(Debug, Clone)]
pub struct RequestHeaders {
    pub device_client_os: String,
    pub device_client_model: String,
    pub device_client_type: String,
    pub account_id: String,
    pub authorization: Option<String>,
}

impl RequestHeaders {
    pub fn from_request(req: &HttpRequest) -> Self {
        let device_client_os = req.headers()
            .get("x-client-os")
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_default();
        
        let device_client_model = req.headers()
            .get("x-client-device")
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_default();
        
        let device_client_type = req.headers()
            .get("x-client-platform")
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_default();
        
        let account_id = req.headers()
            .get("x-client-id")
            .and_then(|h| h.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        let authorization = req.headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .map(String::from);

        Self {
            device_client_os,
            device_client_model,
            device_client_type,
            account_id,
            authorization,
        }
    }
}

