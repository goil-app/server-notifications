# Configuración: Servidor Rust fuera de Docker

Esta configuración permite que el servidor Rust corra directamente en Ubuntu mientras que Grafana, Loki, Promtail y Prometheus corren en Docker.

## Estructura

- **Docker**: Grafana, Loki, Promtail, Prometheus
- **Ubuntu (host)**: Servidor Rust compilado y ejecutándose directamente

## Configuración del Servidor Rust

### 1. Compilar el servidor

```bash
# En el servidor Ubuntu
cd ~/notis/server-notifications
cargo build --release
```

### 2. Configurar variables de entorno

Asegúrate de que el archivo `.env` esté en el directorio del proyecto con todas las variables necesarias.

### 3. Crear directorio de logs

```bash
mkdir -p ~/notis/server-notifications/logs
```

El middleware de logging escribirá los logs en `./logs/server-notifications.log` que será leído por Promtail.

### 4. Ejecutar el servidor

```bash
# Opción 1: Ejecutar directamente
cd ~/notis/server-notifications
./target/release/server-notifications

# Opción 2: Usar systemd (recomendado para producción)
sudo cp server-notifications.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable server-notifications
sudo systemctl start server-notifications
```

## Configuración de Docker (Solo Monitoreo)

### Levantar servicios de monitoreo

```bash
# Solo levantar Grafana, Loki, Promtail y Prometheus
docker compose up -d
```

Esto levantará:
- **Loki** (puerto 3100)
- **Promtail** (lee logs de `./logs/`)
- **Prometheus** (puerto 9090)
- **Grafana** (puerto 3000)

### Verificar que Promtail lee los logs

```bash
# Ver logs de Promtail
docker compose logs -f promtail

# Verificar que está leyendo el archivo
docker compose exec promtail ls -lh /var/log/app/
```

## Configuración de Logs

El middleware de logging está configurado para escribir logs en:
- **stdout**: Para ver logs en tiempo real
- **Archivo**: `./logs/server-notifications.log` (para Promtail)

El archivo se crea automáticamente cuando `LOG_DIR` está configurado en el `.env`:

```env
LOG_DIR=./logs
```

O puedes configurarlo como variable de entorno:

```bash
export LOG_DIR=./logs
```

## Verificar que Funciona

### 1. Verificar que el servidor Rust está corriendo

```bash
# Verificar proceso
ps aux | grep server-notifications

# Verificar que responde
curl http://localhost:8080/health
```

### 2. Verificar que los logs se están generando

```bash
# Ver logs en tiempo real
tail -f ~/notis/server-notifications/logs/server-notifications.log
```

### 3. Verificar que Promtail está leyendo los logs

```bash
# Ver logs de Promtail
docker compose logs promtail | grep "server-notifications"
```

### 4. Verificar en Grafana

1. Abre Grafana: http://localhost:3000
2. Ve a "Explore"
3. Selecciona "Loki"
4. Query: `{job="server-notifications"}`
5. Deberías ver los logs del servidor Rust

## Configuración de Prometheus (Opcional)

Si quieres que Prometheus scrapee métricas del servidor Rust (cuando implementes `/metrics`):

1. El servidor Rust debe estar accesible desde Docker
2. Usa `host.docker.internal:8080` o la IP del host
3. Ajusta `prometheus.yml` según tu configuración de red

## Troubleshooting

### Promtail no lee los logs

1. **Verificar que el archivo existe:**
   ```bash
   ls -lh ~/notis/server-notifications/logs/server-notifications.log
   ```

2. **Verificar permisos:**
   ```bash
   chmod 644 ~/notis/server-notifications/logs/server-notifications.log
   ```

3. **Verificar que Promtail puede acceder:**
   ```bash
   docker compose exec promtail ls -lh /var/log/app/
   ```

### Los logs no aparecen en Grafana

1. **Verificar que Loki está corriendo:**
   ```bash
   curl http://localhost:3100/ready
   ```

2. **Verificar que Promtail está enviando logs:**
   ```bash
   docker compose logs promtail | grep -i error
   ```

3. **Probar query en Loki directamente:**
   ```bash
   curl 'http://localhost:3100/loki/api/v1/query?query={job="server-notifications"}'
   ```

## Ventajas de esta Configuración

- ✅ El servidor Rust corre nativamente (mejor rendimiento)
- ✅ No necesitas reconstruir la imagen Docker cada vez que cambias código
- ✅ Más fácil de debuggear (logs directos en el sistema)
- ✅ Monitoreo completo con Grafana/Loki/Prometheus
- ✅ Puedes usar systemd para gestionar el servidor Rust

