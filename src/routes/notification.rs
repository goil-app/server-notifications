use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::dev::HttpServiceFactory;
use actix_web::middleware::from_fn;
use crate::middleware::platform::mobile_platform_guard;
use crate::middleware::auth::auth_guard;
use crate::response::ApiResponse;
use crate::application::GetNotificationUseCase;
use crate::infrastructure::notification::mongo::MongoNotificationRepository;
use crate::mappers::notification::domain_to_response;
use mongodb::Database;

async fn get_notification(_req: HttpRequest, db: web::Data<Database>, path: web::Path<String>) -> impl Responder {
    let id: String = path.into_inner();
    let repo = MongoNotificationRepository::new(db.get_ref().clone());
    let usecase = GetNotificationUseCase::new(repo);
    match usecase.execute(&id).await {
        Ok(n) => {
            let resp = domain_to_response(n);
            HttpResponse::Ok().json(ApiResponse::ok(resp))
        },
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub fn router() -> impl HttpServiceFactory {
    web::scope("/api/v2/notification")
        // .wrap(from_fn(mobile_platform_guard))
        // .wrap(from_fn(auth_guard))
        .route("/{id}", web::get().to(get_notification))
}

