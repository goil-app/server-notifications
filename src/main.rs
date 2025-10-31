use actix_web::{App, HttpServer};
use actix_web::middleware::{NormalizePath, TrailingSlash};
mod routes;
mod middleware;
mod types;
mod domain;
mod application;
mod infrastructure { pub mod notification; }
mod response;
// mod state; // eliminado: no usamos AppState

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno desde .env si existe
    let _ = dotenvy::dotenv();
    HttpServer::new(|| App::new()
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}


