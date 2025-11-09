use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::HttpMessage;
use crate::infrastructure::services::AppServices;
use crate::response::ApiResponse;
use crate::types::AuthContext;
use crate::mappers::{notification::domain_to_response, common::sha512_hash};
use crate::infrastructure::external::queue::QueueRequestHeaders;

/// Controlador para endpoints de notificaciones
pub struct NotificationController;

impl NotificationController {
    /// Obtiene una notificación por ID, incluyendo datos del usuario autenticado
    pub async fn get_notification(
        req: HttpRequest,
        services: actix_web::web::Data<AppServices>,
        id: String,
        business_ids: Vec<String>,
    ) -> impl Responder {
        // Extraer contexto y validar autenticación
        let (language, auth_ctx, business_id) = match Self::extract_context(&req) {
            Ok(ctx) => ctx,
            Err(response) => return response,
        };

        // Preparar businessIds para usar en queries
        let business_ids_to_use = if business_ids.is_empty() {
            vec![business_id.clone()]
        } else {
            business_ids.clone()
        };

        // Ejecutar queries en paralelo
        let is_mongo_id = mongodb::bson::oid::ObjectId::parse_str(&id).is_ok();
        let (user_result, notification_result, business_result, getstream_unread_result) = tokio::join!(
            Self::fetch_user(&services, &auth_ctx.user_id, &business_id, &business_ids),
            Self::fetch_notification(&services, &id, is_mongo_id, &auth_ctx.user_id, &language, &business_id),
            services.business.get_business.execute(&business_id),
            services.notification.get_getstream_unread_count.execute(&auth_ctx.user_id),
        );

        // Procesar notificación (obligatoria)
        let notification = match notification_result {
            Ok(n) => n,
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching notification {}: {:?}", id, e);
                return HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Notification not found"));
            }
        };

        // Procesar resultados opcionales
        let user = user_result.ok();
        let business = business_result.ok();

        // Obtener datos adicionales para calcular unread count real
        // Estas queries están optimizadas para ejecutarse rápidamente
        let getstream_unread_count = getstream_unread_result.unwrap_or(0);
        
        let (all_notifications, notification_reads) = if user.is_some() {
            Self::fetch_additional_data(&services, &user, &business_ids_to_use).await
        } else {
            (None, None)
        };

        // Calcular unread count
        // Si las queries adicionales fallaron o timeout, usamos solo GetStream (que es más rápido)
        let unread_count = Self::calculate_unread_count(
            &all_notifications,
            &notification_reads,
            getstream_unread_count,
        );

        // Encolar tracking solo si la notificación es de MongoDB (es decir, es una notificación del servidor)
        if is_mongo_id {
            let tracking_headers = Self::extract_tracking_headers(&req);
            let _ = services.notification.enqueue_track_notification.execute(
                &id,
                &auth_ctx.user_id,
                Some(business_id.clone()),
                auth_ctx.session_id.clone(), // Extraer sessionId del token
                tracking_headers,
            ).await;
            // Ignoramos errores de tracking para no afectar la respuesta principal
        }

        // Construir respuesta
        let business_name = business.map(|b| b.name).unwrap_or_else(|| "Goil".to_string());
        let resp = domain_to_response(
            notification,
            &services.storage.s3_signer,
            Some(business_id.clone()),
            Some(business_name),
            unread_count,
        ).await;

        HttpResponse::Ok().json(ApiResponse::ok(resp))
    }

    // Métodos privados de ayuda

    fn extract_context(req: &HttpRequest) -> Result<(String, AuthContext, String), HttpResponse> {
        let language = req.extensions()
            .get::<String>()
            .cloned()
            .unwrap_or_else(|| "es".to_string());
        
        let Some(auth_ctx) = req.extensions().get::<AuthContext>().cloned() else {
            return Err(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Missing authentication context")));
        };
        
        let business_id = auth_ctx.business_id.clone();
        if business_id.is_empty() {
            return Err(HttpResponse::Forbidden()
                .json(ApiResponse::<()>::error("Missing or invalid business_id")));
        }

        Ok((language, auth_ctx, business_id))
    }

    async fn fetch_user(
        services: &AppServices,
        user_id: &str,
        business_id: &str,
        query_business_ids: &[String],
    ) -> Result<crate::domain::SimplifiedUser, crate::domain::UserRepoError> {
        if query_business_ids.is_empty() {
            services.user.get_user.execute(user_id, business_id).await
        } else {
            services.user.get_user_by_business_ids.execute(user_id, query_business_ids).await
        }
    }

    async fn fetch_notification(
        services: &AppServices,
        id: &str,
        is_mongo_id: bool,
        user_id: &str,
        language: &str,
        business_id: &str,
    ) -> Result<crate::domain::Notification, crate::domain::NotificationRepoError> {
        if is_mongo_id {
            // Si es ID de MongoDB, ejecutar consulta a MongoDB
            services.notification.get_notification.execute(id, language, business_id).await
        } else {
            // Si no es ID de MongoDB, ejecutar consulta a GetStream (UUID)
            services.notification.get_getstream_message.execute(id, user_id, language, business_id).await
                .map_err(|e| crate::domain::NotificationRepoError::Unexpected(format!("getstream: {}", e)))
        }
    }

    async fn fetch_additional_data(
        services: &AppServices,
        user: &Option<crate::domain::SimplifiedUser>,
        business_ids: &[String],
    ) -> (Option<Vec<String>>, Option<Vec<String>>) {
        let Some(ref user) = user else {
            return (None, None);
        };

        let phone = &user.phone;
        
        // OPTIMIZACIÓN: Ejecutar queries en paralelo desde el inicio
        // Pre-calcular hash del teléfono mientras se obtienen usuarios
        let (users_result, hashed_phone) = tokio::join!(
            services.user.get_users.execute(phone, business_ids),
            async { sha512_hash(phone) } // Calcular hash en paralelo
        );
        
        let users_found = match users_result {
            Ok(users) => users,
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching users by phone {}: {:?}", phone, e);
                // Si falla obtener usuarios, aún podemos intentar obtener reads con el phone hasheado
                let notification_reads_result = services.analytics.get_notification_reads.execute(&hashed_phone, business_ids).await;
                return (None, notification_reads_result.ok());
            }
        };

        // Si no hay usuarios encontrados, solo obtener reads
        if users_found.is_empty() {
            let notification_reads_result = services.analytics.get_notification_reads.execute(&hashed_phone, business_ids).await;
            return (None, notification_reads_result.ok());
        }

        // Optimización: hash phones en paralelo usando rayon o iteración optimizada
        let users_with_hashed_phone: Vec<_> = users_found.iter()
            .map(|u| {
                let mut user_with_hashed_phone = u.clone();
                user_with_hashed_phone.phone = sha512_hash(&u.phone);
                user_with_hashed_phone
            })
            .collect();

        // Ejecutar ambas queries en paralelo
        let (all_notifications_result, notification_reads_result) = tokio::join!(
            services.notification.get_users_notifications.execute(&users_with_hashed_phone, business_ids),
            services.analytics.get_notification_reads.execute(&hashed_phone, business_ids)
        );

        (
            all_notifications_result.ok(),
            notification_reads_result.ok(),
        )
    }

    fn calculate_unread_count(
        all_notifications: &Option<Vec<String>>,
        notification_reads: &Option<Vec<String>>,
        getstream_unread_count: i32,
    ) -> i32 {
        use std::collections::HashSet;
        
        let all_notifications = all_notifications.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let reads = notification_reads.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        
        let notifications_set: HashSet<&String> = all_notifications.iter().collect();
        let reads_set: HashSet<&String> = reads.iter().collect();
        
        let server_unread_count = notifications_set
            .difference(&reads_set)
            .count() as i32;

        server_unread_count + getstream_unread_count
    }

    fn extract_tracking_headers(req: &HttpRequest) -> QueueRequestHeaders {
        let authorization = req.headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        
        let x_client_platform = req.headers()
            .get("x-client-platform")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| Some("mobile-platform".to_string())); // Default si no existe
        
        let x_client_os = req.headers()
            .get("x-client-os")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        
        let x_client_device = req.headers()
            .get("x-client-device")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        
        let x_client_id = req.headers()
            .get("x-client-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        QueueRequestHeaders {
            authorization,
            x_client_platform,
            x_client_os,
            x_client_device,
            x_client_id,
        }
    }
}

