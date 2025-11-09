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

    // Configurar URL de Loki desde variable de entorno o usar default
    let loki_url = std::env::var("LOKI_URL")
        .unwrap_or_else(|_| "https://gobs.goil.app/loki/loki/api/v1/push".to_string());
    
    // Obtener hostname para labels
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());
    
    // Inicializar tracing con Loki
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use url::Url;
    
    let url = Url::parse(&loki_url)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid Loki URL: {}", e)))?;
    
    let (layer, task) = tracing_loki::builder()
        .label("job", "server-notifications")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .label("service", "server-notifications")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .label("hostname", hostname.as_str())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki label error: {}", e)))?
        .extra_field("pid", format!("{}", std::process::id()))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki field error: {}", e)))?
        .build_url(url)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Loki init error: {}", e)))?;
    
    // También mantener stdout para debugging local
    let stdout_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(std::io::stdout);
    
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .with(stdout_layer)
        .init();
    
    // Spawn la tarea de Loki en background
    tokio::spawn(task);
    
    eprintln!("[main] Logging inicializado: enviando logs a Loki en {}", loki_url);

    // Configurar workers dinámicamente basado en los CPUs disponibles
    // Cada worker tiene su propio pool de MongoDB, así que el pool total = workers × max_pool_size
    let num_workers = std::env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|w| w.parse::<usize>().ok())
        .unwrap_or_else(|| {
            // Usar todos los CPUs disponibles (uno por vCPU es óptimo)
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });
    
    // Calcular pool size de MongoDB dinámicamente para alto rendimiento
    // Balance entre rendimiento y estabilidad (evitar timeouts)
    // Fórmula: conexiones por worker ajustadas para evitar sobrecarga de MongoDB
    let connections_per_worker = match num_workers {
        1 => 150,   // 1 worker: 150 conexiones (más estable que 300)
        2 => 100,   // 2 workers: 100 conexiones cada uno = 200 total
        3..=4 => 80, // 3-4 workers: 80 conexiones cada uno = 240-320 total
        5..=8 => 60, // 5-8 workers: 60 conexiones cada uno = 300-480 total
        _ => 50,   // 9+ workers: 50 conexiones cada uno
    };
    
    // Para alto rendimiento, mantener conexiones calientes pero sin sobrecargar
    let min_pool_size = (connections_per_worker / 4).max(10) as u32; // Mínimo 25% del pool o 10
    let max_pool_size = connections_per_worker as u32;
    
    let total_connections = num_workers * connections_per_worker;
    
    println!("[main] Detected {} CPU cores, starting with {} workers", 
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1), 
        num_workers);
    println!("[main] MongoDB pool: {} connections per worker (total: {} connections)", 
        connections_per_worker, total_connections);

    // Inicializa las bases de datos de MongoDB con configuración dinámica
    let databases = infrastructure::db::Databases::init_with_pool_config(
        Some(max_pool_size),
        Some(min_pool_size),
    ).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("mongo init error: {}", e)))?;

    // Crea todos los servicios de la aplicación
    let services = infrastructure::services::AppServices::new(&databases).await
        .map_err(|e| {
            eprintln!("[main] Error initializing services: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, format!("services init error: {}", e))
        })?;

    
    let port =  std::env::var("API_PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap_or(8080);
    HttpServer::new(move || App::new()
        .wrap(middleware::logging::StructuredLogging)
        .wrap(NormalizePath::new(TrailingSlash::Trim))
        .app_data(actix_web::web::Data::new(services.clone()))
        .service(routes::health::router())
        .service(routes::notification::router()))
        .bind(("0.0.0.0", port))?
        .workers(num_workers)
        // Optimizaciones para alto rendimiento (miles de req/s)
        .client_request_timeout(Duration::from_millis(5000)) // Timeout: 5 segundos (suficiente para queries optimizadas)
        .client_disconnect_timeout(Duration::from_millis(1000)) // Desconectar rápidamente para liberar recursos
        .keep_alive(Duration::from_secs(30)) // Keep-alive moderado
        .backlog(8192) // Aumentar backlog para aceptar más conexiones pendientes
        .run()
        .await
}


