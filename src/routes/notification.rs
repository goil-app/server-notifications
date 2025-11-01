use actix_web::{web, HttpRequest};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use serde::Deserialize;
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use crate::middleware::session::session_guard;
use crate::controllers::NotificationController;

#[derive(Deserialize)]
pub struct NotificationQueryParams {
    #[serde(rename = "businessIds[]")]
    pub business_ids: Option<Vec<String>>,
}

async fn get_notification(
    req: HttpRequest,
    services: web::Data<crate::infrastructure::services::AppServices>,
    path: web::Path<String>,
    query: web::Query<NotificationQueryParams>,
) -> impl actix_web::Responder {
    let id = path.into_inner();
    let business_ids = query.business_ids.clone().unwrap_or_default();
    NotificationController::get_notification(req, services, id, business_ids).await
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        .wrap(from_fn(session_guard))
        .wrap(from_fn(auth_guard))
        .wrap(from_fn(mobile_platform_guard))
        .route("/{id}/me", web::get().to(get_notification))
}

