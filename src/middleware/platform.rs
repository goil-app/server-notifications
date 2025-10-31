use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use actix_web::body::BoxBody; // BoxBody: tipo único para el cuerpo de la respuesta que evita genéricos opacos en middlewares
use actix_web::middleware::Next;
use actix_web::http::header::HeaderName;
use crate::response::ApiResponse;

static CLIENT_PLATFORM_HEADER: &str = "x-client-platform";

pub async fn mobile_platform_guard( // middleware: valida header x-client-platform == "mobile-platform"
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let header_name = HeaderName::from_lowercase(CLIENT_PLATFORM_HEADER.as_bytes()).unwrap(); // construye el nombre del header en minúsculas
    let is_mobile = req
        .headers() // accede al mapa de cabeceras
        .get(&header_name) // obtiene la cabecera `x-client-platform`
        .and_then(|h| h.to_str().ok()) // intenta convertir el valor a &str (UTF-8)
        .map(|v| v.trim().eq_ignore_ascii_case("mobile-platform")) // compara ignorando mayúsculas/espacios
        .unwrap_or(false); // si falta o no es válido, considera false

    if !is_mobile { // si el header no es "mobile-platform"
        let body = serde_json::to_string(&ApiResponse::error("Platform not Authorized")).unwrap();
        let res = HttpResponse::Forbidden()
            .content_type("application/json")
            .body(body)
            .map_into_boxed_body(); // convierte a BoxBody
        return Ok(req.into_response(res)); // corta la cadena y devuelve la respuesta
    }

    let res = next.call(req).await?.map_into_boxed_body(); // continúa con el siguiente middleware/handler
    Ok(res) // devuelve la respuesta resultante
}

