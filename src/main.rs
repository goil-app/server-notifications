use actix_web::{App, HttpServer};
use actix_web::middleware::{NormalizePath, TrailingSlash};
mod routes;
mod middleware;
mod types;
mod domain;
mod application;
mod infrastructure { pub mod notification; pub mod db; }
mod response;
mod mappers;
// mod state; // eliminado: no usamos AppState

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno desde .env si existe
    let _ = dotenvy::dotenv();
    // Inicializa Mongo una vez y compártelo
    let db = infrastructure::db::init_database().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("mongo init error: {}", e)))?;

    HttpServer::new(move || App::new()
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(db.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}


