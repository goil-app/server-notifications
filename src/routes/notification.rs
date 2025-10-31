use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use serde::Serialize;
use crate::response::ApiResponse;
use crate::application::GetNotificationUseCase;
use crate::infrastructure::notification::random::RandomNotificationRepository;

#[derive(Serialize)]
struct ApiOkResponse<T> { timestamp: i64, data: T }

#[derive(Serialize)]
struct Data { message: String }

async fn get_notification(_req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let id: String = path.into_inner();
    let repo = RandomNotificationRepository::new();
    let usecase = GetNotificationUseCase::new(repo);
    match usecase.execute(&id).await {
        Ok(n) => HttpResponse::Ok().json(ApiResponse::ok(Data { message: n.message })),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        .wrap(from_fn(mobile_platform_guard))
        .wrap(from_fn(auth_guard))
        .route("/{id}", web::get().to(get_notification))
}

