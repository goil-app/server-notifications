use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::HttpMessage;
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
        business_ids: Vec<String>,
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
        
        // Preparar businessIds: si no hay en query params, usar solo el business_id del token
        let business_ids_to_use = if business_ids.is_empty() {
            vec![business_id.clone()]
        } else {
            business_ids.clone()
        };
        
        // Preparar futures concurrentes
        // 1) Usuario (por businessIds de query o por token)
        let user_future = async {
            if business_ids.is_empty() {
                services.user.get_user.execute(&auth_ctx.user_id, &business_id).await
            } else {
                services.user.get_user_by_business_ids.execute(&auth_ctx.user_id, &business_ids_to_use).await
            }
        };
        // 2) Notificación (Mongo o GetStream si id es UUID)
        let is_uuid = uuid::Uuid::parse_str(&id).is_ok();
        let notification_future = async {
            if is_uuid {
                services.notification.get_getstream_message.execute(&id, &auth_ctx.user_id, &language, &business_id).await
                    .map_err(|e| crate::domain::NotificationRepoError::Unexpected(format!("getstream: {}", e)))
            } else {
                services.notification.get_notification.execute(&id, &language, &business_id).await
            }
        };
        // 3) Business
        let business_future = services.business.get_business.execute(&business_id);
        // 4) Unread de GetStream
        let getstream_unread_future = services.notification.get_getstream_unread_count.execute(&auth_ctx.user_id);

        // Ejecutar primera hornada en paralelo
        let (user_result, notification_result, business_result, getstream_unread_result) = tokio::join!(
            user_future,
            notification_future,
            business_future,
            getstream_unread_future,
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

        // Procesar resultado de business (opcional, continuamos aunque falle)
        let business = match business_result {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("[NotificationController::get_notification] Error fetching business {}: {:?}", business_id, e);
                None // Continuamos aunque falle obtener el business
            }
        };

        // Nueva lógica: buscar usuarios con el mismo teléfono y obtener notificaciones de todos
        // Los usuarios deben pertenecer a los businessIds de la query (o al del token si no hay query params)
        let (all_notifications, notification_reads) = if let Some(ref user) = user {
            let phone = &user.phone;
            // Buscar usuarios que pertenezcan a los businessIds especificados
            let users_result = services.user.get_users.execute(phone, &business_ids_to_use).await;
            let users_found = match users_result {
                Ok(users) => users,
                Err(e) => {
                    eprintln!("[NotificationController::get_notification] Error fetching users by phone {}: {:?}", phone, e);
                    vec![] // Continuar con array vacío si falla
                }
            };

            // 2. Preparar usuarios con phone hasheado y obtener todas las notificaciones en una sola query
            let users_with_hashed_phone: Vec<_> = users_found.iter()
                .map(|u| {
                    let mut user_with_hashed_phone = u.clone();
                    user_with_hashed_phone.phone = sha512_hash(&u.phone);
                    user_with_hashed_phone
                })
                .collect();

            // Preparar phone hasheado antes del tokio::join! para evitar problemas de ownership
            let hashed_phone = sha512_hash(phone);

            // Ejecutar búsqueda de notificaciones para todos los usuarios en paralelo con notification reads
            let (all_notifications_result, notification_reads_result) = tokio::join!(
                services.notification.get_users_notifications.execute(&users_with_hashed_phone, &business_ids_to_use),
                services.analytics.get_notification_reads.execute(&hashed_phone, &business_ids_to_use)
            );
            
            let all_notifications = match all_notifications_result {
                Ok(notifications) => Some(notifications),
                Err(e) => {
                    eprintln!("[NotificationController::get_notification] Error fetching notifications for users: {:?}", e);
                    None
                }
            };

            let notification_reads = match notification_reads_result {
                Ok(nr) => Some(nr),
                Err(e) => {
                    eprintln!("[NotificationController::get_notification] Error fetching notification reads for phone: {:?}", e);
                    None
                }
            };
            
            (all_notifications, notification_reads)
        } else {
            (None, None)
        };
        
        // 4. Comparar qué notificaciones no tienen ningún read
        // Nota: Esta lógica se calcula pero no se usa actualmente en la respuesta.
        // Se mantiene para futuras implementaciones donde se pueda usar isRead.
        let unread_notification_ids: Vec<String> = {
            use std::collections::HashSet;
            let all_notifications: Vec<String> = all_notifications.unwrap_or_default();
            let reads = notification_reads.unwrap_or_default();
            let notifications_set: HashSet<&String> = all_notifications.iter().collect();
            let reads_set: HashSet<&String> = reads.iter().collect();
            notifications_set
                .difference(&reads_set)
                .map(|id| (*id).clone())
                .collect()
        };
        // Obtener businessName del business si existe, sino usar "Goil" por defecto
        let business_name = business.map(|b| b.name).unwrap_or_else(|| "Goil".to_string());
        
        // Obtener el número de notificaciones pendientes de leer
        let server_unread_count = unread_notification_ids.len() as i32;
        
        // Unread de GetStream ya resuelto en paralelo (si falla, 0)
        let getstream_unread_count = getstream_unread_result.unwrap_or(0);
        let unread_count = server_unread_count + getstream_unread_count;
        
        let resp = domain_to_response(notification, &services.storage.s3_signer, Some(business_id.clone()), Some(business_name), unread_count).await;
        HttpResponse::Ok().json(ApiResponse::ok(resp))
    }
}

