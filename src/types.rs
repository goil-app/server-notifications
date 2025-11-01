use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtClaims {
    pub user_id: String,
    pub type_id: Option<String>,
    pub session_id: Option<String>,
    pub business_id: Option<String>,

}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: String,
    #[allow(dead_code)] // Campo de dominio, puede usarse en el futuro
    pub account_type_id: Option<String>,
    pub session_id: Option<String>,
    pub business_id: String,
}

