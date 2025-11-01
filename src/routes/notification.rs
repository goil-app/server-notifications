use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use actix_web::HttpMessage; // para extensions()
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use crate::middleware::session::session_guard;
use crate::response::ApiResponse;
use crate::infrastructure::services::AppServices;
use crate::mappers::notification::domain_to_response;
use crate::types::AuthContext;

async fn get_notification(req: HttpRequest, services: web::Data<AppServices>, path: web::Path<String>) -> impl Responder {
    let id: String = path.into_inner();
    
    // Obtener el language de la sesión desde extensions (inyectado por session_guard)
    // Si no hay language, usar "es" por defecto
    let language = req.extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "es".to_string());
    
    // Obtener business_id del AuthContext (inyectado por auth_guard)
    let business_id = req.extensions()
        .get::<AuthContext>()
        .map(|ctx| ctx.business_id.clone())
        .filter(|bid| !bid.is_empty());
    
    let Some(business_id) = business_id else {
        return HttpResponse::Forbidden()
            .json(ApiResponse::<()>::error("Missing or invalid business_id"));
    };
    
    match services.get_notification.execute(&id, &language, &business_id).await {
        Ok(n) => {
            let resp = domain_to_response(n);
            HttpResponse::Ok().json(ApiResponse::ok(resp))
        },
        Err(e) => {
            eprintln!("[get_notification] Error fetching notification {}: {:?}", id, e);
            HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Notification not found"))
        }
    }
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        .wrap(from_fn(session_guard))
        .wrap(from_fn(auth_guard))
        .wrap(from_fn(mobile_platform_guard))
        .route("/{id}", web::get().to(get_notification))
}

