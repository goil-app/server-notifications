# Flujo de Trabajo con Docker

## ¿Cuándo se reconstruye la imagen?

Docker **NO reconstruye** la imagen automáticamente si:
- ✅ La imagen ya existe
- ✅ No hay cambios en el `Dockerfile`
- ✅ No hay cambios en los archivos que se copian (según `.dockerignore`)

Docker **SÍ reconstruye** automáticamente si:
- ❌ Cambias el `Dockerfile`
- ❌ Cambias archivos que se copian en el Dockerfile (como `Cargo.toml`, `Cargo.lock`, código fuente)
- ❌ La imagen no existe

## Comandos Comunes

### Levantar servicios (sin reconstruir si ya existe la imagen)
```bash
docker compose up -d
```
**Usa la imagen existente** si no hay cambios.

### Levantar y reconstruir (si hay cambios)
```bash
docker compose up -d --build
```
**Reconstruye la imagen** antes de levantar.

### Solo reconstruir la imagen
```bash
docker compose build server-notifications
```

### Reconstruir sin cache (forzar reconstrucción completa)
```bash
docker compose build --no-cache server-notifications
```

### Ver qué imágenes tienes
```bash
docker images | grep server-notifications
```

## Optimización para Desarrollo

### Opción 1: Reconstruir solo cuando cambies código
```bash
# Primera vez o cuando cambies código
docker compose build server-notifications
docker compose up -d

# Siguientes veces (sin cambios)
docker compose up -d  # No reconstruye, usa imagen existente
```

### Opción 2: Forzar reconstrucción cuando lo necesites
```bash
# Reconstruir y levantar
docker compose up -d --build server-notifications

# O solo reconstruir
docker compose build server-notifications
docker compose up -d
```

### Opción 3: Reconstruir sin cache (cuando hay problemas)
```bash
docker compose build --no-cache server-notifications
docker compose up -d
```

## Verificar si hay cambios

Docker detecta automáticamente cambios en:
- `Dockerfile`
- Archivos copiados con `COPY` o `ADD`
- Archivos listados en `.dockerignore` (estos se ignoran)

## Mejores Prácticas

1. **Primera vez:**
   ```bash
   docker compose build
   docker compose up -d
   ```

2. **Desarrollo normal (sin cambios en código):**
   ```bash
   docker compose up -d  # No reconstruye
   ```

3. **Después de cambiar código Rust:**
   ```bash
   docker compose up -d --build server-notifications
   ```

4. **Después de cambiar Dockerfile:**
   ```bash
   docker compose build --no-cache server-notifications
   docker compose up -d
   ```

## Ver logs de build

```bash
# Ver el proceso de build
docker compose build server-notifications

# Ver logs detallados
docker compose build --progress=plain server-notifications
```

## Limpiar imágenes viejas

```bash
# Eliminar imágenes no utilizadas
docker image prune -a

# Eliminar todo (imágenes, contenedores, volúmenes)
docker system prune -a --volumes
```

## Resumen

- **`docker compose up -d`** → Usa imagen existente (rápido)
- **`docker compose up -d --build`** → Reconstruye si hay cambios
- **`docker compose build`** → Solo reconstruye, no levanta
- **`docker compose build --no-cache`** → Reconstruye desde cero

**Recomendación:** Usa `docker compose up -d` normalmente. Solo reconstruye cuando cambies código o el Dockerfile.

