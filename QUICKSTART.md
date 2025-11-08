# 🚀 Guía Rápida: Levantar el Stack Completo

## Paso 1: Verificar Requisitos Previos

```bash
# Verificar que Docker está instalado
docker --version

# Verificar que Docker Compose está instalado
docker-compose --version
```

Si no tienes Docker instalado, instálalo desde: https://www.docker.com/get-started

## Paso 2: Preparar Variables de Entorno (Opcional)

Crea un archivo `.env` en la raíz del proyecto con tus variables:

```bash
# Copiar ejemplo (si existe)
cp .env.example .env

# O crear uno nuevo
touch .env
```

Edita el `.env` con tus valores:
```env
MONGODB_URI=mongodb://mongo:27017
JWT_MOBILE_PLATFORM=tu_secret_key
GETSTREAM_API_KEY=tu_api_key
GETSTREAM_SECRET_KEY=tu_secret_key
AWS_ACCESS_KEY_ID=tu_access_key
AWS_SECRET_ACCESS_KEY=tu_secret_key
SERVICE_NAME=server-notifications
```

**Nota:** Si no creas el `.env`, el `docker-compose.yml` usará valores por defecto para algunas variables.

## Paso 3: Crear Directorio de Logs

```bash
mkdir -p logs
```

Este directorio es necesario para que Promtail pueda leer los logs.

## Paso 4: Construir y Levantar los Servicios

```bash
# Construir las imágenes y levantar todos los servicios
docker-compose up -d
```

El flag `-d` ejecuta los servicios en segundo plano (detached mode).

**¿Qué hace este comando?**
- Construye la imagen de tu servicio Rust
- Descarga las imágenes de Loki, Prometheus, Grafana, Promtail y MongoDB
- Crea la red Docker para que los servicios se comuniquen
- Inicia todos los servicios

## Paso 5: Verificar que Todo Está Corriendo

```bash
# Ver el estado de todos los servicios
docker-compose ps
```

Deberías ver algo como:
```
NAME                    STATUS          PORTS
grafana                 Up              0.0.0.0:3000->3000/tcp
loki                    Up              0.0.0.0:3100->3100/tcp
mongo                   Up              0.0.0.0:27017->27017/tcp
prometheus              Up              0.0.0.0:9090->9090/tcp
promtail                Up
server-notifications    Up              0.0.0.0:8080->8080/tcp
```

Todos deberían estar en estado "Up".

## Paso 6: Verificar los Logs Iniciales

```bash
# Ver logs de todos los servicios
docker-compose logs

# O ver logs de un servicio específico
docker-compose logs server-notifications
docker-compose logs loki
docker-compose logs prometheus
```

Si ves errores, anótalos para revisarlos después.

## Paso 7: Verificar que los Servicios Responden

### Verificar Loki
```bash
curl http://localhost:3100/ready
```
Debería responder: `ready`

### Verificar Prometheus
```bash
curl http://localhost:9090/-/healthy
```
Debería responder: `Prometheus is Healthy.`

### Verificar Grafana
```bash
curl http://localhost:3000/api/health
```
Debería responder con un JSON con `"database": "ok"`

### Verificar tu Servicio
```bash
curl http://localhost:8080/health
```
Debería responder con un JSON de health check.

## Paso 8: Acceder a Grafana

1. **Abre tu navegador** y ve a: http://localhost:3000

2. **Login inicial:**
   - Usuario: `admin`
   - Contraseña: `admin`
   - Te pedirá cambiar la contraseña (puedes hacerlo o saltar)

3. **Verificar Datasources:**
   - Ve a "Configuration" (icono de engranaje) > "Data Sources"
   - Deberías ver:
     - **Loki** (marcado como default)
     - **Prometheus**
   - Haz clic en cada uno y presiona "Save & Test"
   - Debería mostrar "Data source is working" en verde

## Paso 9: Probar que los Logs Funcionan

1. **Hacer una petición a tu API:**
```bash
curl http://localhost:8080/health
```

2. **Verificar que se generó el log:**
```bash
# Ver el archivo de log
tail -f ./logs/server-notifications.log

# O ver logs de Docker
docker-compose logs -f server-notifications
```

3. **Verificar en Grafana:**
   - Ve a "Explore" (icono de brújula)
   - Selecciona "Loki" como datasource
   - Escribe la query: `{job="server-notifications"}`
   - Presiona "Run query"
   - Deberías ver el log de la petición que acabas de hacer

## Paso 10: Verificar Prometheus

1. **Abrir Prometheus UI:**
   - Ve a: http://localhost:9090

2. **Verificar Targets:**
   - Ve a "Status" > "Targets"
   - Deberías ver los targets configurados
   - Todos deberían estar en estado "UP" (verde)

3. **Hacer una query de prueba:**
   - Ve a "Graph"
   - Escribe: `up`
   - Presiona "Execute"
   - Deberías ver métricas

## Comandos Útiles

### Ver logs en tiempo real
```bash
# Todos los servicios
docker-compose logs -f

# Solo tu servicio
docker-compose logs -f server-notifications

# Solo Loki
docker-compose logs -f loki
```

### Reiniciar un servicio
```bash
docker-compose restart server-notifications
```

### Detener todo
```bash
docker-compose down
```

### Detener y eliminar volúmenes (⚠️ borra datos)
```bash
docker-compose down -v
```

### Reconstruir un servicio
```bash
docker-compose up -d --build server-notifications
```

## Troubleshooting

### Si un servicio no inicia:

1. **Ver los logs del servicio:**
```bash
docker-compose logs nombre-del-servicio
```

2. **Verificar que el puerto no esté en uso:**
```bash
# Ver qué está usando el puerto 8080
lsof -i :8080

# O en Linux
netstat -tulpn | grep 8080
```

3. **Reiniciar el servicio:**
```bash
docker-compose restart nombre-del-servicio
```

### Si los logs no aparecen en Grafana:

1. **Verificar que Promtail está corriendo:**
```bash
docker-compose ps promtail
docker-compose logs promtail
```

2. **Verificar que el archivo de log existe:**
```bash
ls -la ./logs/
```

3. **Verificar permisos:**
```bash
chmod 755 ./logs
```

### Si Prometheus no scrapea:

1. **Verificar la configuración:**
```bash
curl http://localhost:9090/api/v1/status/config
```

2. **Verificar targets:**
```bash
curl http://localhost:9090/api/v1/targets
```

## Resumen de URLs

Una vez levantado todo, puedes acceder a:

- **Tu API:** http://localhost:8080
- **Grafana:** http://localhost:3000 (admin/admin)
- **Prometheus:** http://localhost:9090
- **Loki:** http://localhost:3100

## Siguiente Paso

Una vez que todo esté funcionando, puedes:
- Crear dashboards en Grafana
- Configurar alertas en Prometheus
- Personalizar las queries de logs en Loki
- Implementar un endpoint `/metrics` en tu servicio Rust para métricas personalizadas

