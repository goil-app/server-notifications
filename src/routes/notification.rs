use actix_web::{web, HttpRequest};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use crate::middleware::session::session_guard;
use crate::controllers::NotificationController;

async fn get_notification(req: HttpRequest, services: web::Data<crate::infrastructure::services::AppServices>, path: web::Path<String>) -> impl actix_web::Responder {
    let id = path.into_inner();
    NotificationController::get_notification(req, services, id).await
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        .wrap(from_fn(session_guard))
        .wrap(from_fn(auth_guard))
        .wrap(from_fn(mobile_platform_guard))
        .route("/{id}", web::get().to(get_notification))
}

