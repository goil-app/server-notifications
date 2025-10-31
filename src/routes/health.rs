use actix_web::{web, HttpResponse, Responder, Scope};
use serde::Serialize;
use actix_web::get;
use crate::response::ApiResponse;

#[derive(Serialize)]
struct HealthData { status: &'static str }

async fn health() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::ok(HealthData { status: "ok" }))
}

pub fn router() -> Scope {
    web::scope("/health").route("", web::get().to(health))
}

