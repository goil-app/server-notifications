use actix_web::{App, HttpServer};
use actix_web::middleware::{NormalizePath, TrailingSlash};
mod routes;
mod middleware;
mod types;
mod domain;
mod application;
mod infrastructure { pub mod notification; pub mod session; pub mod db; pub mod services; }
mod response;
mod mappers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Cargar variables de entorno desde .env si existe
    // dotenvy carga el .env pero NO sobrescribe variables que ya existen en el sistema
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("[main] Warning: Could not load .env file: {}. Using system environment variables.", e);
    }
    // Inicializa las bases de datos de MongoDB
    let databases = infrastructure::db::Databases::init().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("mongo init error: {}", e)))?;

    // Crea todos los servicios de la aplicación
    let services = infrastructure::services::AppServices::new(&databases);

    HttpServer::new(move || App::new()
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(services.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}


