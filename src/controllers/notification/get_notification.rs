use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::HttpMessage;
use std::collections::HashSet;
use crate::infrastructure::services::AppServices;
use crate::response::ApiResponse;
use crate::types::AuthContext;
use crate::mappers::{notification::domain_to_response, common::sha512_hash};

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

        // Obtener notificaciones del usuario y notification reads en paralelo si existe el usuario
        let (user_notifications, notification_reads) = if let Some(ref user) = user {
            // Crear un usuario con el phone hasheado en SHA512
            let mut user_with_hashed_phone = user.clone();
            user_with_hashed_phone.phone = sha512_hash(&user.phone);
            
            // Ejecutar ambas consultas en paralelo
            let (user_notifications_result, notification_reads_result) = tokio::join!(
                services.get_user_notifications.execute(&user_with_hashed_phone),
                services.get_notification_reads.execute(&user_with_hashed_phone)
            );
            
            let user_notifications = match user_notifications_result {
                Ok(un) => Some(un),
                Err(e) => {
                    eprintln!("[NotificationController::get_notification] Error fetching user notifications {}: {:?}", auth_ctx.user_id, e);
                    None
                }
            };
            
            let notification_reads = match notification_reads_result {
                Ok(nr) => Some(nr),
                Err(e) => {
                    eprintln!("[NotificationController::get_notification] Error fetching notification reads {}: {:?}", auth_ctx.user_id, e);
                    None
                }
            };
            
            (user_notifications, notification_reads)
        } else {
            (None, None)
        };
        
        // Calcular notificaciones no leídas: notificaciones en user_notifications pero no en notification_reads
        let _unread_notification_ids: Vec<String> = {
            let user_notifications = user_notifications.unwrap_or_default();
            let reads = notification_reads.unwrap_or_default();
            let notifications_set: HashSet<&String> = user_notifications.iter().collect();
            let reads_set: HashSet<&String> = reads.iter().collect();
            notifications_set
                .difference(&reads_set)
                .map(|id| (*id).clone())
                .collect()
        };
        let resp = domain_to_response(notification);
        HttpResponse::Ok().json(ApiResponse::ok(resp))
    }
}

