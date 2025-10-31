use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use actix_web::body::BoxBody; // BoxBody: cuerpo unificado para evitar tipos opacos en middlewares
use actix_web::http::header::{HeaderName, AUTHORIZATION};
use actix_web::middleware::Next;
use actix_web::HttpMessage; // para extensions_mut()
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::json;
use crate::response::ApiResponse;

use crate::types::{AuthContext, JwtClaims};

fn unauthorized(msg: &str) -> HttpResponse {
    HttpResponse::Unauthorized()
        .content_type("application/json")
        .body(serde_json::to_string(&ApiResponse::error(msg)).unwrap())
}

fn internal_error(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("application/json")
        .body(serde_json::to_string(&ApiResponse::error(msg)).unwrap())
}

pub async fn auth_guard( // middleware: valida y decodifica JWT HS256 y añade AuthContext en Extensions
    mut req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // Authorization: lee el header y quita el prefijo "Bearer " si existe
    let auth = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let token_str = auth.strip_prefix("Bearer ").unwrap_or(auth);

    // x-client-platform (paridad con el código JS; ahora no lo validamos aquí)
    let client_platform = req
        .headers()
        .get(HeaderName::from_lowercase(b"x-client-platform").unwrap())
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let _ = client_platform; // reservado por si se quiere validar algo más adelante

    // Config JWT: HS256 fijo; secreto desde entorno (o valor por defecto en dev)
    let secret = "f9^NcW3%z@Y!r6$Lm1B#"; // sustituir por std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into()) en producción

    // Decodificar y validar firma/exp con HS256
    let claims = match decode::<JwtClaims>(
        token_str,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token) => token.claims, // éxito: obtenemos los claims
        Err(err) => {
            use jsonwebtoken::errors::ErrorKind;
            let resp = match err.kind() { // mapear a 401 con mensaje
                ErrorKind::ExpiredSignature => unauthorized("Token expired"),
                ErrorKind::InvalidSignature => unauthorized("Token has invalid signature"),
                _ => unauthorized("Not Authorized"),
            };
            return Ok(req.into_response(resp.map_into_boxed_body()));
        }
    };

    // businessId requerido para continuar (equivalente al check del código JS)
    let Some(business_id) = claims.business_id.clone() else {
        return Ok(req.into_response(
            internal_error("Business id is required").map_into_boxed_body(),
        ));
    };

    // Contexto de autenticación: guardamos en Extensions para que los handlers lo extraigan
    let ctx = AuthContext {
        user_id: claims.user_id,
        account_type_id: claims.type_id,
        session_id: claims.session_id,
        business_id: business_id,
    };
    req.extensions_mut().insert(ctx); // disponible vía req.extensions()

    let res = next.call(req).await?.map_into_boxed_body(); // continúa la cadena
    Ok(res) // devuelve la respuesta resultante
}


