use actix_web::{web, HttpRequest};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use crate::middleware::session::session_guard;
use crate::controllers::NotificationController;

async fn get_notification(
    req: HttpRequest,
    services: web::Data<crate::infrastructure::services::AppServices>,
    path: web::Path<String>,
) -> impl actix_web::Responder {
    let id = path.into_inner();
    
    // Extraer businessIds[] manualmente del query string
    // Actix Web ya decodifica la URL, así que buscamos tanto "businessIds[]" como "businessIds%5B%5D"
    let business_ids: Vec<String> = req.uri().query()
        .map(|query| {
            query
                .split('&')
                .filter_map(|pair| {
                    if let Some((key, value)) = pair.split_once('=') {
                        // Buscar tanto la versión decodificada como codificada del nombre del campo
                        if key == "businessIds[]" || key == "businessIds%5B%5D" {
                            Some(value.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    
    NotificationController::get_notification(req, services, id, business_ids).await
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        .wrap(from_fn(session_guard))
        .wrap(from_fn(auth_guard))
        .wrap(from_fn(mobile_platform_guard))
        .route("/{id}/me", web::get().to(get_notification))
}

