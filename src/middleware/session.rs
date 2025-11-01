use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use actix_web::body::BoxBody; // BoxBody: tipo único para el cuerpo de la respuesta que evita genéricos opacos en middlewares
use actix_web::middleware::Next;
use actix_web::web;
use actix_web::HttpMessage; // para extensions()
use crate::infrastructure::services::AppServices;
use crate::types::AuthContext;
use crate::response::ApiResponse;

fn unauthorized(msg: &str) -> HttpResponse {
    HttpResponse::Unauthorized()
        .content_type("application/json")
        .body(serde_json::to_string(&ApiResponse::<()>::error(msg)).unwrap())
}

fn internal_error(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("application/json")
        .body(serde_json::to_string(&ApiResponse::<()>::error(msg)).unwrap())
}

/// Middleware: valida la sesión usando el use case de session
/// Extrae AppServices desde app_data y usa get_session para validar la sesión
pub async fn session_guard(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // Extraer AppServices desde app_data (inyectado en main.rs)
    let Some(services) = req.app_data::<web::Data<AppServices>>() else {
        // Si no están disponibles los servicios, retornar error interno
        return Ok(req.into_response(
            internal_error("Services not available").map_into_boxed_body(),
        ));
    };

    // Obtener session_id del AuthContext (inyectado por auth_guard)
    let Some(session_id) = req.extensions()
        .get::<AuthContext>()
        .and_then(|ctx| ctx.session_id.clone()) else {
        // Si no hay session_id, retornar error
        return Ok(req.into_response(
            unauthorized("Session ID is required").map_into_boxed_body(),
        ));
    };
    
    // Consultar la sesión en MongoDB usando el use case
    match services.get_session.execute(&session_id).await {
        Ok(session) => {
            // Guardar el language de la sesión en extensions para que los handlers lo usen
            req.extensions_mut().insert(session.language);
            // Sesión válida, continuar
            let res = next.call(req).await?.map_into_boxed_body();
            Ok(res)
        }
        Err(_) => {
            // Sesión no encontrada o inválida
            Ok(req.into_response(
                unauthorized("Invalid session").map_into_boxed_body(),
            ))
        }
    }
}