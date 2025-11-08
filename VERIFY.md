# Guía de Verificación de Monitoreo

## Verificación Rápida

Ejecuta el script de verificación:
```bash
./verify-monitoring.sh
```

## Verificación Manual

### 1. Verificar que los servicios estén corriendo

```bash
docker-compose ps
```

Deberías ver todos los servicios con estado "Up":
- `loki`
- `prometheus`
- `promtail`
- `grafana`
- `server-notifications`

### 2. Verificar Loki

**Health check:**
```bash
curl http://localhost:3100/ready
```
Debería responder: `ready`

**Métricas:**
```bash
curl http://localhost:3100/metrics
```
Debería mostrar métricas de Loki.

**Verificar que recibe logs:**
```bash
# Enviar un log de prueba
curl -X POST http://localhost:3100/loki/api/v1/push \
  -H "Content-Type: application/json" \
  -d '{
    "streams": [{
      "stream": {"job": "test"},
      "values": [["'$(date +%s)000000000'", "test log message"]]
    }]
  }'
```

**Consultar logs:**
```bash
# Ver logs del servicio
curl 'http://localhost:3100/loki/api/v1/query?query={job="server-notifications"}'

# Ver logs de los últimos 5 minutos
curl 'http://localhost:3100/loki/api/v1/query_range?query={job="server-notifications"}&start='$(date -d '5 minutes ago' +%s)'000000000&end='$(date +%s)'000000000'
```

### 3. Verificar Prometheus

**Health check:**
```bash
curl http://localhost:9090/-/healthy
```
Debería responder: `Prometheus is Healthy.`

**Verificar configuración:**
```bash
curl http://localhost:9090/api/v1/status/config
```
Debería mostrar la configuración de Prometheus.

**Verificar targets (servicios que scrapea):**
```bash
curl http://localhost:9090/api/v1/targets
```
Deberías ver los targets configurados en `prometheus.yml`.

**Verificar que puede scrapear:**
```bash
# Ver métricas disponibles
curl 'http://localhost:9090/api/v1/query?query=up'

# Ver métricas de un job específico
curl 'http://localhost:9090/api/v1/query?query=up{job="prometheus"}'
```

**Abrir Prometheus UI:**
- Navega a: http://localhost:9090
- Ve a "Status" > "Targets" para ver el estado de los targets
- Ve a "Graph" para hacer queries

### 4. Verificar Promtail

**Ver logs de Promtail:**
```bash
docker-compose logs promtail
```

Deberías ver mensajes como:
- `msg="Starting Promtail"`
- `level=info msg="Starting Promtail"`

**Verificar que está enviando logs a Loki:**
```bash
# Ver logs recientes de Promtail
docker-compose logs --tail=50 promtail | grep -i "error\|warn"
```

Si no hay errores, Promtail está funcionando correctamente.

### 5. Verificar Grafana

**Health check:**
```bash
curl http://localhost:3000/api/health
```
Debería responder con estado de Grafana.

**Verificar datasources:**
```bash
curl -u admin:admin http://localhost:3000/api/datasources
```

Deberías ver:
- Loki (con `isDefault: true`)
- Prometheus

**Abrir Grafana UI:**
- Navega a: http://localhost:3000
- Login: `admin` / `admin`
- Ve a "Configuration" > "Data Sources" para verificar que Loki y Prometheus están configurados

### 6. Verificar que los logs se están generando

**Ver logs del servicio:**
```bash
docker-compose logs -f server-notifications
```

Deberías ver logs en formato JSON con cada request.

**Verificar archivo de log:**
```bash
# Ver si el archivo existe
ls -lh ./logs/server-notifications.log

# Ver últimas líneas
tail -f ./logs/server-notifications.log
```

**Hacer una petición de prueba:**
```bash
# Esto debería generar un log
curl http://localhost:8080/health
```

Luego verifica que el log aparezca en:
1. Docker logs: `docker-compose logs server-notifications`
2. Archivo: `tail ./logs/server-notifications.log`
3. Loki: `curl 'http://localhost:3100/loki/api/v1/query?query={job="server-notifications"}'`

## Verificación en Grafana

### Ver Logs en Grafana

1. Abre Grafana: http://localhost:3000
2. Ve a "Explore" (icono de brújula en el menú lateral)
3. Selecciona "Loki" como datasource
4. Escribe una query:
   ```
   {job="server-notifications"}
   ```
5. Haz clic en "Run query"
6. Deberías ver los logs del servicio

### Ver Métricas en Grafana

1. Ve a "Explore"
2. Selecciona "Prometheus" como datasource
3. Escribe una query, por ejemplo:
   ```
   up
   ```
4. Haz clic en "Run query"
5. Deberías ver métricas

## Troubleshooting

### Loki no recibe logs

1. **Verificar que Promtail está corriendo:**
   ```bash
   docker-compose ps promtail
   ```

2. **Verificar logs de Promtail:**
   ```bash
   docker-compose logs promtail
   ```

3. **Verificar que el archivo de log existe:**
   ```bash
   ls -la ./logs/
   ```

4. **Verificar permisos:**
   ```bash
   chmod 755 ./logs
   ```

### Prometheus no scrapea

1. **Verificar configuración:**
   ```bash
   curl http://localhost:9090/api/v1/status/config
   ```

2. **Verificar targets:**
   ```bash
   curl http://localhost:9090/api/v1/targets
   ```

3. **Verificar conectividad de red:**
   ```bash
   docker-compose exec prometheus wget -O- http://server-notifications:8080/metrics
   ```

### Grafana no muestra datos

1. **Verificar datasources:**
   ```bash
   curl -u admin:admin http://localhost:3000/api/datasources
   ```

2. **Probar conexión a Loki:**
   - En Grafana: Configuration > Data Sources > Loki > "Save & Test"
   - Debería mostrar "Data source is working"

3. **Probar conexión a Prometheus:**
   - En Grafana: Configuration > Data Sources > Prometheus > "Save & Test"
   - Debería mostrar "Data source is working"

## Comandos Útiles

```bash
# Ver todos los logs de todos los servicios
docker-compose logs -f

# Reiniciar un servicio específico
docker-compose restart loki

# Ver métricas de un servicio
docker stats loki

# Limpiar todo y empezar de nuevo
docker-compose down -v
docker-compose up -d
```

