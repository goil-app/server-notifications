use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::HttpMessage;
use crate::infrastructure::services::AppServices;
use crate::response::ApiResponse;
use crate::types::AuthContext;
use crate::mappers::{notification::domain_to_response, common::sha512_hash, request_headers::RequestHeaders};
use crate::application::notification::TrackNotificationParams;

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
        let is_uuid = uuid::Uuid::parse_str(&id).is_ok();
        let (user_result, notification_result, business_result, getstream_unread_result) = tokio::join!(
            Self::fetch_user(&services, &auth_ctx.user_id, &business_id, &business_ids),
            Self::fetch_notification(&services, &id, is_uuid, &auth_ctx.user_id, &language, &business_id),
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

        // Encolar tracking si es necesario
        if !is_uuid && crate::mappers::common::is_object_id_or_hex_string(&id) {
            Self::enqueue_tracking(&services, &req, &id, &business_id, &auth_ctx);
        }

        // Procesar resultados opcionales
        let user = user_result.ok();
        let business = business_result.ok();

        // Obtener notificaciones adicionales y reads si tenemos usuario
        let (all_notifications, notification_reads) = Self::fetch_additional_data(
            &services,
            &user,
            &business_ids_to_use,
        ).await;

        // Calcular unread count
        let unread_count = Self::calculate_unread_count(
            &all_notifications,
            &notification_reads,
            getstream_unread_result.unwrap_or(0),
        );

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
        is_uuid: bool,
        user_id: &str,
        language: &str,
        business_id: &str,
    ) -> Result<crate::domain::Notification, crate::domain::NotificationRepoError> {
        if is_uuid {
            services.notification.get_getstream_message.execute(id, user_id, language, business_id).await
                .map_err(|e| crate::domain::NotificationRepoError::Unexpected(format!("getstream: {}", e)))
        } else {
            services.notification.get_notification.execute(id, language, business_id).await
        }
    }

    fn enqueue_tracking(
        services: &AppServices,
        req: &HttpRequest,
        notification_id: &str,
        business_id: &str,
        auth_ctx: &AuthContext,
    ) {
        let headers = RequestHeaders::from_request(req);
        
        let params = TrackNotificationParams {
            id: notification_id.to_string(),
            business_id: business_id.to_string(),
            account_id: headers.account_id,
            device_client_type: headers.device_client_type,
            device_client_model: headers.device_client_model,
            device_client_os: headers.device_client_os,
            session_id: auth_ctx.session_id.clone().unwrap_or_default(),
        };

        services.notification.enqueue_track_notification.execute_async(params, headers.authorization);
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
        let users_result = services.user.get_users.execute(phone, business_ids).await;
        let users_found = match users_result {
            Ok(users) => users,
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching users by phone {}: {:?}", phone, e);
                return (None, None);
            }
        };

        let users_with_hashed_phone: Vec<_> = users_found.iter()
            .map(|u| {
                let mut user_with_hashed_phone = u.clone();
                user_with_hashed_phone.phone = sha512_hash(&u.phone);
                user_with_hashed_phone
            })
            .collect();

        let hashed_phone = sha512_hash(phone);

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
}

