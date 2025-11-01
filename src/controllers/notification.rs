use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::HttpMessage;
use crate::infrastructure::services::AppServices;
use crate::response::ApiResponse;
use crate::types::AuthContext;
use crate::mappers::notification::domain_to_response;

/// Controlador para endpoints de notificaciones
pub struct NotificationController;

impl NotificationController {
    /// Obtiene una notificación por ID, incluyendo datos del usuario autenticado
    pub async fn get_notification(
        req: HttpRequest,
        services: actix_web::web::Data<AppServices>,
        id: String,
    ) -> impl Responder {
        // Obtener el language de la sesión desde extensions (inyectado por session_guard)
        // Si no hay language, usar "es" por defecto
        let language = req.extensions()
            .get::<String>()
            .cloned()
            .unwrap_or_else(|| "es".to_string());
        
        // Obtener business_id y user_id del AuthContext (inyectado por auth_guard)
        let Some(auth_ctx) = req.extensions().get::<AuthContext>().cloned() else {
            return HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Missing authentication context"));
        };
        
        let business_id = auth_ctx.business_id.clone();
        if business_id.is_empty() {
            return HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Missing or invalid business_id"));
        };
        
        // Ejecutar ambas consultas en paralelo
        let (notification_result, user_result) = tokio::join!(
            services.get_notification.execute(&id, &language, &business_id),
            services.get_user.execute(&auth_ctx.user_id, &business_id)
        );
        
        // Procesar resultado de notificación
        let notification = match notification_result {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching notification {}: {:?}", id, e);
                return HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Notification not found"));
            }
        };
        
        // Procesar resultado de usuario (opcional, continuamos aunque falle)
        let user = match user_result {
            Ok(u) => Some(u),
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching user {}: {:?}", auth_ctx.user_id, e);
                None // Continuamos aunque falle obtener el usuario
            }
        };
        println!("user: {:?}", user);
        
        let resp = domain_to_response(notification);
        HttpResponse::Ok().json(ApiResponse::ok(resp))
    }
}

