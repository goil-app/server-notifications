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

/// Carga las variables de entorno desde el archivo .env
fn load_environment() {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("[main] Warning: Could not load .env file: {}. Using system environment variables.", e);
    }
}

/// Inicializa el sistema de logging con Loki
fn init_logging(loki_url: &str, hostname: &str) -> std::io::Result<()> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use url::Url;
    
    let url = Url::parse(loki_url)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid Loki URL: {}", e)))?;
    
    let (layer, task) = tracing_loki::builder()
        .label("job", "server-notifications")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .label("service", "server-notifications")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .label("host", hostname)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .build_url(url)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki init error: {}", e)))?;
    
    let stdout_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::stdout);
    
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    
    eprintln!("[main] Tracing filter: {:?}", filter);
    
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .with(stdout_layer)
        .init();
    
    eprintln!("[main] Tracing subscriber inicializado");
    
    tokio::spawn(async move {
        eprintln!("[main] Tarea de Loki iniciada");
        task.await;
        eprintln!("[main] ⚠️  Tarea de Loki terminó (no debería pasar)");
    });
    
    eprintln!("[main] Logging inicializado: enviando logs a Loki en {}", loki_url);
    eprintln!("[main] Hostname: {}", hostname);
    
    Ok(())
}

/// Crea la configuración de logging compartida
fn create_logging_config() -> middleware::logging::LoggingConfig {
    middleware::logging::LoggingConfig::from_env()
}

/// Calcula el número de workers basado en CPUs disponibles
fn calculate_workers() -> usize {
    std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|w| w.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
}

/// Calcula la configuración del pool de MongoDB basado en el número de workers
fn calculate_mongodb_pool_config(num_workers: usize) -> (u32, u32, usize) {
    let connections_per_worker = match num_workers {
        1 => 150,
        2 => 100,
        3..=4 => 80,
        5..=8 => 60,
        _ => 50,
    };
    
    let min_pool_size = (connections_per_worker / 4).max(10) as u32;
    let max_pool_size = connections_per_worker as u32;
    let total_connections = num_workers * connections_per_worker;
    
    (min_pool_size, max_pool_size, total_connections)
}

/// Obtiene el puerto del servidor desde variable de entorno o usa el default
fn get_server_port() -> u16 {
    std::env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080)
}

/// Inicializa las bases de datos de MongoDB
async fn init_databases(max_pool_size: u32, min_pool_size: u32) -> std::io::Result<infrastructure::db::Databases> {
    infrastructure::db::Databases::init_with_pool_config(
        Some(max_pool_size),
        Some(min_pool_size),
    ).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("mongo init error: {}", e)))
}

/// Inicializa todos los servicios de la aplicación
async fn init_services(databases: &infrastructure::db::Databases) -> std::io::Result<infrastructure::services::AppServices> {
    infrastructure::services::AppServices::new(databases).await
        .map_err(|e| {
            eprintln!("[main] Error initializing services: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, format!("services init error: {}", e))
        })
}

/// Configura y arranca el servidor HTTP
async fn start_server(
    services: infrastructure::services::AppServices,
    port: u16,
    num_workers: usize,
    logging_config: middleware::logging::LoggingConfig,
) -> std::io::Result<()> {
    HttpServer::new(move || App::new()
        .wrap(middleware::logging::StructuredLogging::new(logging_config.clone()))
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(services.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", port))?
        .workers(num_workers)
        .client_request_timeout(Duration::from_millis(5000))
        .client_disconnect_timeout(Duration::from_millis(1000))
        .keep_alive(Duration::from_secs(30))
        .backlog(8192)
        .run()
        .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_environment();
    
    // Crear configuración de logging una sola vez (se reutiliza en middleware)
    let logging_config = create_logging_config();
    
    // Inicializar tracing-loki (solo para stdout, el middleware envía directamente a Loki)
    let loki_url = logging_config.loki_url.clone();
    let hostname = logging_config.hostname.clone();
    init_logging(&loki_url, &hostname)?;
    
    let num_workers = calculate_workers();
    let (min_pool_size, max_pool_size, total_connections) = calculate_mongodb_pool_config(num_workers);
    let connections_per_worker = max_pool_size as usize;
    
    println!("[main] Detected {} CPU cores, starting with {} workers", 
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1), 
        num_workers);
    println!("[main] MongoDB pool: {} connections per worker (total: {} connections)", 
        connections_per_worker, total_connections);

    let databases = init_databases(max_pool_size, min_pool_size).await?;
    let services = init_services(&databases).await?;
    let port = get_server_port();
    
    start_server(services, port, num_workers, logging_config).await
}
