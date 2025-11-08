# Guía de Monitoreo con Loki, Prometheus y Grafana

Este proyecto incluye una configuración completa de monitoreo usando Docker Compose con:
- **Loki**: Agregador de logs
- **Prometheus**: Recolector de métricas
- **Grafana**: Visualización y dashboards
- **Promtail**: Recolector de logs para Loki

## Requisitos Previos

- Docker y Docker Compose instalados
- Variables de entorno configuradas en un archivo `.env` (opcional)

## Inicio Rápido

1. **Levantar todos los servicios:**
```bash
docker-compose up -d
```

2. **Verificar que todos los servicios estén corriendo:**
```bash
docker-compose ps
```

3. **Acceder a Grafana:**
   - URL: http://localhost:3000
   - Usuario: `admin`
   - Contraseña: `admin` (cambiar en producción)

4. **Acceder a Prometheus:**
   - URL: http://localhost:9090

5. **Acceder a Loki:**
   - URL: http://localhost:3100

## Servicios Incluidos

### server-notifications
- Puerto: `8080`
- Logs: Se escriben en `./logs/server-notifications.log` y stdout
- Métricas: Disponibles en `/metrics` (si se implementa)

### Loki
- Puerto: `3100`
- Almacena logs estructurados en formato JSON
- Configuración: `loki-config.yml`

### Promtail
- Recolecta logs de:
  - Archivos en `./logs/`
  - Containers Docker
  - Logs de stdout/stderr
- Configuración: `promtail-config.yml`

### Prometheus
- Puerto: `9090`
- Recolecta métricas de:
  - El servicio Rust (si implementas `/metrics`)
  - Loki
  - Grafana
  - Prometheus mismo
- Configuración: `prometheus.yml`

### Grafana
- Puerto: `3000`
- Datasources pre-configurados:
  - Loki (default)
  - Prometheus
- Configuración: `grafana-datasources.yml`

## Configuración de Logs

Los logs se generan en formato JSON estructurado compatible con Grafana/Loki. Cada log incluye:
- `name`: Nombre del servicio
- `hostname`: Hostname del servidor
- `pid`: Process ID
- `level`: Nivel de log (30=INFO, 40=WARN, 50=ERROR)
- `http`: Información HTTP (path, method, statusCode, headers, etc.)
- `time`: Timestamp ISO 8601
- `v`: Versión del formato

## Consultas en Grafana

### Logs (Loki)

**Ver todos los logs del servicio:**
```logql
{job="server-notifications"}
```

**Filtrar por status code:**
```logql
{job="server-notifications"} | json | http_statusCode=200
```

**Buscar errores:**
```logql
{job="server-notifications"} | json | level>=40
```

**Filtrar por path:**
```logql
{job="server-notifications"} | json | http_path=~"/api/.*"
```

### Métricas (Prometheus)

Si implementas un endpoint `/metrics` en el servicio Rust, podrás consultar métricas como:
- `http_requests_total`
- `http_request_duration_seconds`
- `http_request_size_bytes`

## Desarrollo

### Ver logs en tiempo real:
```bash
docker-compose logs -f server-notifications
```

### Reiniciar un servicio:
```bash
docker-compose restart server-notifications
```

### Detener todos los servicios:
```bash
docker-compose down
```

### Detener y eliminar volúmenes:
```bash
docker-compose down -v
```

## Producción

Para producción, considera:

1. **Cambiar credenciales de Grafana** en `docker-compose.yml`
2. **Configurar autenticación** para Prometheus y Loki
3. **Ajustar retención de datos** en `loki-config.yml` y `prometheus.yml`
4. **Configurar alertas** en Prometheus
5. **Usar un MongoDB externo** en lugar del container incluido
6. **Configurar backups** de los volúmenes de datos

## Troubleshooting

### Los logs no aparecen en Grafana
1. Verifica que Promtail esté corriendo: `docker-compose ps promtail`
2. Revisa los logs de Promtail: `docker-compose logs promtail`
3. Verifica que el directorio `./logs` exista y tenga permisos correctos

### Prometheus no scrapea métricas
1. Verifica que el endpoint `/metrics` esté implementado
2. Revisa la configuración en `prometheus.yml`
3. Verifica la conectividad de red: `docker-compose exec prometheus wget -O- http://server-notifications:8080/metrics`

### Grafana no se conecta a Loki/Prometheus
1. Verifica que los servicios estén en la misma red Docker
2. Revisa la configuración en `grafana-datasources.yml`
3. Verifica los logs de Grafana: `docker-compose logs grafana`

