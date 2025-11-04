use actix_web::{App, HttpServer};
use actix_web::middleware::{NormalizePath, TrailingSlash};
use std::time::Duration;
mod routes;
mod middleware;
mod types;
mod domain;
mod application;
mod infrastructure { pub mod notification; pub mod session; pub mod user; pub mod analytics; pub mod business; pub mod external; pub mod db; pub mod services; pub mod s3; pub mod providers; }
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
        .map_err(|e| {
            eprintln!("[main] Error initializing services: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, format!("services init error: {}", e))
        })?;

    // Configurar workers: Para 4 vCPU y 8GB RAM, optimizamos recursos
    // Cada worker tiene su propio pool de MongoDB, así que el pool total = workers × max_pool_size
    // Con 4 vCPUs, usar 4 workers (uno por vCPU) aprovecha mejor el paralelismo
    let num_workers = std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|w| w.parse::<usize>().ok())
        .unwrap_or_else(|| {
            // Para 4 vCPU: usar 4 workers es óptimo
            // Con más RAM disponible, podemos mantener más conexiones
            std::thread::available_parallelism()
                .map(|n| n.get().min(4)) // Máximo 4 workers para aprovechar los 4 vCPUs
                .unwrap_or(4)
        });
    
    println!("[main] Starting server with {} workers (machine: 4vCPU, 8GB RAM)", num_workers);
    println!("[main] MongoDB pool: {} connections per worker (total: {} connections)", 50, num_workers * 50);
    
    let port =  std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);
    HttpServer::new(move || App::new()
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(services.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", port))?
        .workers(num_workers)
        // Optimizaciones para recursos mejorados (8GB RAM)
        .client_request_timeout(Duration::from_millis(10000)) // Timeout de cliente: 10 segundos (más tiempo para operaciones complejas)
        .client_disconnect_timeout(Duration::from_millis(2000)) // Desconectar clientes inactivos (más tiempo con más recursos)
        .keep_alive(Duration::from_secs(60)) // Keep-alive más largo para aprovechar conexiones persistentes
        .run()
        .await
}


