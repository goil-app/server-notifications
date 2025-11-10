use jsonwebtoken::{EncodingKey, Header, Algorithm};

use crate::domain::getstream::GetStreamRepoError;

#[derive(serde::Serialize)]
struct GetStreamClaims<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<&'a str>,
    exp: i64,
    iat: i64,
}

pub fn generate_getstream_jwt(user_id: Option<&str>, ttl_seconds: i64) -> Result<String, GetStreamRepoError> {
    let now = chrono::Utc::now().timestamp();
    let claims = GetStreamClaims { user_id, exp: now + ttl_seconds, iat: now };
    let secret = std::env::var("GETSTREAM_SECRET").map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
    let header = Header::new(Algorithm::HS256);
    let token = jsonwebtoken::encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| GetStreamRepoError::Unexpected(e.to_string()))?;
    Ok(token)
}


