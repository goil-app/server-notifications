use async_trait::async_trait;

use crate::domain::Notification;
use serde_json::Value;
use crate::domain::getstream::{GetStreamRepository, GetStreamRepoError};
use crate::infrastructure::external::getstream_auth::generate_getstream_jwt;

#[derive(Clone, Default)]
pub struct HttpGetStreamRepository;

// Nota: las claims ahora viven en getstream_auth.rs

#[async_trait]
impl GetStreamRepository for HttpGetStreamRepository {
    async fn find_message_by_uuid(&self, id: &str, user_id: &str, _language: &str, _business_id: &str) -> Result<Notification, GetStreamRepoError> {
        // 1) Generar JWT
        let token = generate_getstream_jwt(user_id, 60)?;

        // 3) Preparar request HTTP
        let api_key = std::env::var("GETSTREAM_API_KEY").map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
        let url = format!("https://chat.stream-io-api.com/messages/{}", id);
        let client = reqwest::Client::new();

        let resp = client
            .get(&url)
            .header("Stream-Auth-Type", "jwt")
            .header("Authorization", token)
            .header("api_key", api_key)
            .send()
            .await
            .map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
        
        println!("[GetStream] status={} body={}", status, body);
        
        // Validar status HTTP antes de continuar
        if !status.is_success() {
            return Err(GetStreamRepoError::Unexpected(format!("GetStream API returned status {}: {}", status, body)));
        }

        // 4) Mapear title y body según reglas
        let parsed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        let message = parsed.get("message");
        if message.is_none() {
            return Ok(Notification {
                id: id.to_string(),
                title: "".to_string(),
                body: "".to_string(),
                image_paths: vec![],
            });
        }

        let m = message.unwrap();
        let channel_name = m
            .get("channel")
            .and_then(|c| c.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let channel_type = m
            .get("channel")
            .and_then(|c| c.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let user_name = m
            .get("user")
            .and_then(|u| u.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let text = m.get("text").and_then(|v| v.as_str()).unwrap_or("");

        let mut title = channel_name.to_string();
        let body_text = format!("{}: {}", user_name, text);
        if channel_type == "messaging-oneToOne" {
            title = user_name.to_string();
        }

        Ok(Notification {
            id: id.to_string(),
            title,
            body: body_text,
            image_paths: vec![],
        })
    }

    async fn get_unread_count(&self, user_id: &str) -> Result<i32, GetStreamRepoError> {
        let token = generate_getstream_jwt(user_id, 60)?;

        let api_key = std::env::var("GETSTREAM_API_KEY").map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
        let url = "https://chat.stream-io-api.com/unread";
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("Stream-Auth-Type", "jwt")
            .header("Authorization", token)
            .header("api_key", api_key)
            .send()
            .await
            .map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;

        let status = resp.status();
        let body = resp.text().await.map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
        println!("[GetStream unread] status={} body={}", status, body);
        if !status.is_success() { return Ok(0); }

        // Intentar extraer un contador plausible de la respuesta
        let value: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
        let count = value.get("total_unread_count")
            .and_then(|v| v.as_i64())
            .or_else(|| value.get("unread_count").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        Ok(count as i32)
    }
}


