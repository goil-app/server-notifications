# server-notifications

API mínima en Rust (Actix Web) con un único endpoint que devuelve JSON estático.

## Ejecutar

```bash
cargo run
```

Servidor en `http://localhost:8080`.

## Endpoint

- GET `/api/v2/notification/{id}`
  - Headers: `Authorization: Bearer <token>` (opcional por ahora)
  - Respuesta (200):
    ```json
    {"status":"ok","message":"static response"}
    ```