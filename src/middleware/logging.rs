use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::body::MessageBody;
use futures_util::future::LocalBoxFuture;
use serde_json::{json, Value};
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::SystemTime,
};

/// Middleware de logging estructurado en formato JSON compatible con Grafana
pub struct StructuredLogging;

impl<S, B> Transform<S, ServiceRequest> for StructuredLogging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    B::Error: std::fmt::Display,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = StructuredLoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(StructuredLoggingMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct StructuredLoggingMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for StructuredLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    B::Error: std::fmt::Display,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let start_time = SystemTime::now();

        // Capturar información del request
        let method = req.method().to_string();
        let path = req.path();
        let query_string = req.query_string();
        let full_path = if query_string.is_empty() {
            path.to_string()
        } else {
            format!("{}?{}", path, query_string)
        };

        // Capturar headers del request
        let mut request_headers = serde_json::Map::new();
        for (name, value) in req.headers().iter() {
            let header_name = name.as_str().to_lowercase();
            if let Ok(header_value) = value.to_str() {
                request_headers.insert(header_name, Value::String(header_value.to_string()));
            }
        }

        // Obtener hostname del sistema
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        // Obtener nombre del servicio desde env o usar default
        let service_name = std::env::var("SERVICE_NAME")
            .unwrap_or_else(|_| "server-notifications".to_string());

        // Obtener PID
        let pid = std::process::id();

        Box::pin(async move {
            let res = service.call(req).await?;

            // Calcular tiempo de respuesta
            let duration = start_time.elapsed().unwrap_or_default();
            let duration_ms = duration.as_millis();

            // Capturar información del response
            let status_code = res.status().as_u16();

            // Capturar TODOS los headers del response (incluyendo los que se añaden después)
            let mut response_headers = serde_json::Map::new();
            for (name, value) in res.headers().iter() {
                let header_name = name.as_str().to_lowercase();
                if let Ok(header_value) = value.to_str() {
                    // Si el header ya existe, convertir a array para mantener todos los valores
                    if let Some(existing) = response_headers.get_mut(&header_name) {
                        match existing {
                            Value::String(s) => {
                                // Convertir a array con el valor anterior y el nuevo
                                let old_value = s.clone();
                                response_headers.insert(
                                    header_name.clone(),
                                    Value::Array(vec![Value::String(old_value), Value::String(header_value.to_string())]),
                                );
                            }
                            Value::Array(arr) => {
                                // Añadir al array existente
                                arr.push(Value::String(header_value.to_string()));
                            }
                            _ => {
                                // Convertir a array
                                let old = existing.clone();
                                response_headers.insert(
                                    header_name.clone(),
                                    Value::Array(vec![old, Value::String(header_value.to_string())]),
                                );
                            }
                        }
                    } else {
                        response_headers.insert(header_name, Value::String(header_value.to_string()));
                    }
                }
            }

            // Convertir el response a BoxBody y leer el body
            let res = res.map_into_boxed_body();
            let (req_parts, res_body) = res.into_parts();
            
            // Extraer el body usando map_body
            let (head, body) = res_body.into_parts();
            let body_bytes = actix_web::body::to_bytes(body).await;
            
            let response_body: Value = match &body_bytes {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        Value::Null
                    } else {
                        // Intentar parsear como JSON, si falla usar el string
                        match serde_json::from_slice::<Value>(bytes) {
                            Ok(json) => {
                                // Si es un objeto JSON, extraer solo el campo "data"
                                if let Some(obj) = json.as_object() {
                                    if let Some(data) = obj.get("data") {
                                        data.clone()
                                    } else {
                                        // Si no tiene "data", devolver el objeto completo pero sin "timestamp"
                                        let mut filtered = serde_json::Map::new();
                                        for (key, value) in obj.iter() {
                                            if key != "timestamp" {
                                                filtered.insert(key.clone(), value.clone());
                                            }
                                        }
                                        Value::Object(filtered)
                                    }
                                } else {
                                    json
                                }
                            }
                            Err(_) => {
                                // Si no es JSON, intentar como string UTF-8
                                match String::from_utf8(bytes.to_vec()) {
                                    Ok(s) => Value::String(s),
                                    Err(_) => Value::String(format!("<binary data: {} bytes>", bytes.len())),
                                }
                            }
                        }
                    }
                }
                Err(_) => Value::Null,
            };

            // Reconstruir el response con el body leído
            let body_bytes_final = body_bytes.unwrap_or_default();
            let mut res_body_rebuilt = actix_web::HttpResponse::with_body(head.status(), BoxBody::new(body_bytes_final));
            // Copiar headers del head original
            for (name, value) in head.headers().iter() {
                res_body_rebuilt.headers_mut().insert(name.clone(), value.clone());
            }
            let res = ServiceResponse::new(req_parts, res_body_rebuilt);

            // Determinar nivel de log basado en status code
            let level = if status_code >= 500 {
                50 // ERROR
            } else if status_code >= 400 {
                40 // WARN
            } else {
                30 // INFO
            };

            // Obtener timestamp ISO 8601
            let time = chrono::Utc::now().to_rfc3339();

            // Construir log estructurado
            let log_entry = json!({
                "name": service_name,
                "hostname": hostname,
                "pid": pid,
                "level": level,
                "http": {
                    "path": full_path,
                    "method": method,
                    "requestBody": {},
                    "statusCode": status_code,
                    "responseBody": response_body,
                    "requestHeaders": request_headers,
                    "responseHeaders": response_headers,
                    "duration": duration_ms
                },
                "msg": "",
                "time": time,
                "v": 0
            });

            let log_json = serde_json::to_string(&log_entry).unwrap_or_default();
            
            // Escribir a stdout (para Docker logs)
            println!("{}", log_json);
            
            // También escribir a archivo para Promtail (si está configurado)
            if let Ok(log_dir) = std::env::var("LOG_DIR") {
                use std::fs::OpenOptions;
                use std::io::Write;
                let log_file = format!("{}/server-notifications.log", log_dir);
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)
                {
                    let _ = writeln!(file, "{}", log_json);
                }
            }

            Ok(res)
        })
    }
}

