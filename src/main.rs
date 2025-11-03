use actix_web::{App, HttpServer};
use actix_web::middleware::{NormalizePath, TrailingSlash};
use std::time::Duration;
mod routes;
mod middleware;
mod types;
mod domain;
mod application;
mod infrastructure { pub mod notification; pub mod session; pub mod user; pub mod analytics; pub mod business; pub mod external; pub mod db; pub mod services; pub mod s3; }
mod response;
mod mappers;
mod controllers;

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
    let services = infrastructure::services::AppServices::new(&databases).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("s3 signer init error: {}", e)))?;

    // Configurar workers: Para 2 vCPU y 4GB RAM, optimizamos recursos
    // Cada worker tiene su propio pool de MongoDB, así que el pool total = workers × max_pool_size
    // Con recursos limitados, usar 2 workers (uno por vCPU) es ideal
    let num_workers = std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|w| w.parse::<usize>().ok())
        .unwrap_or_else(|| {
            // Para 2 vCPU: usar 2 workers es óptimo
            // Más workers consumen más memoria y conexiones
            std::thread::available_parallelism()
                .map(|n| n.get().min(2)) // Máximo 2 workers para recursos limitados
                .unwrap_or(2)
        });
    
    println!("[main] Starting server with {} workers (machine: 2vCPU, 4GB RAM)", num_workers);
    println!("[main] MongoDB pool: {} connections per worker (total: {} connections)", 30, num_workers * 30);
    
    let port =  std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);
    HttpServer::new(move || App::new()
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(services.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", port))?
        .workers(num_workers)
        // Optimizaciones para recursos limitados (4GB RAM)
        .client_request_timeout(Duration::from_millis(5000)) // Timeout de cliente: 5 segundos
        .client_disconnect_timeout(Duration::from_millis(1000)) // Desconectar clientes inactivos rápidamente
        .keep_alive(Duration::from_secs(30)) // Keep-alive más corto para liberar conexiones
        .run()
        .await
}


