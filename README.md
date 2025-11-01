# server-notifications

API en Rust (Actix Web) con arquitectura hexagonal para notificaciones.

## Configuración

Crea un archivo `.env` en la raíz del proyecto con las siguientes variables:

```bash
# JWT
JWT_SECRET="tu-secreto-jwt-aqui"

# MongoDB
MONGODB_URI="mongodb+srv://usuario:password@cluster.mongodb.net"
MONGODB_NOTIFICATIONS_DB="notification"
MONGODB_ACCOUNT_DB="account"
```

## Ejecutar

```bash
cargo run
```

Servidor en `http://localhost:8080`.

## Endpoints

### Health Check
- GET `/health`
  - Respuesta (200):
    ```json
    {
      "timestamp": 1234567890,
      "data": {
        "status": "ok"
      }
    }
    ```

### Notificaciones
- GET `/api/v2/notification/{id}`
  - Headers requeridos:
    - `x-client-platform: mobile-platform`
    - `Authorization: Bearer <token>`
  - Respuesta (200):
    ```json
    {
      "timestamp": 1234567890,
      "data": {
        "notification": { ... }
      }
    }
    ```

## Arquitectura

El proyecto sigue una arquitectura hexagonal con:
- **Domain**: Modelos y traits del dominio
- **Application**: Casos de uso
- **Infrastructure**: Implementaciones concretas (MongoDB, etc.)
- **Routes**: Handlers HTTP

### Bases de Datos

La aplicación soporta múltiples bases de datos MongoDB:
- `notifications_db`: Para datos de notificaciones
- `account_db`: Para datos de cuentas (preparado para uso futuro)