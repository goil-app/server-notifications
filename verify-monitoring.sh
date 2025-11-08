#!/bin/bash

echo "🔍 Verificando configuración de monitoreo..."
echo ""

# Colores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Función para verificar servicio
check_service() {
    local service=$1
    local url=$2
    local description=$3
    
    echo -n "Verificando $description... "
    if curl -s -f "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ OK${NC}"
        return 0
    else
        echo -e "${RED}✗ FALLO${NC}"
        return 1
    fi
}

# Verificar que los containers estén corriendo
echo "📦 Verificando containers Docker..."
docker-compose ps | grep -E "(loki|prometheus|promtail|grafana)" | while read line; do
    if echo "$line" | grep -q "Up"; then
        service=$(echo "$line" | awk '{print $1}')
        echo -e "  ${GREEN}✓${NC} $service está corriendo"
    else
        service=$(echo "$line" | awk '{print $1}')
        echo -e "  ${RED}✗${NC} $service NO está corriendo"
    fi
done
echo ""

# Verificar Loki
echo "📊 Verificando Loki..."
check_service "loki" "http://localhost:3100/ready" "Loki readiness"
check_service "loki" "http://localhost:3100/metrics" "Loki metrics"

# Verificar que Loki puede recibir logs
echo -n "  Verificando que Loki puede recibir logs... "
if curl -s -X POST "http://localhost:3100/loki/api/v1/push" \
    -H "Content-Type: application/json" \
    -d '{"streams":[{"stream":{"job":"test"},"values":[["'$(date +%s)000000000'","test log"]]}]}' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FALLO${NC}"
fi

# Verificar que hay logs en Loki
echo -n "  Verificando logs en Loki... "
response=$(curl -s "http://localhost:3100/loki/api/v1/query?query={job=\"server-notifications\"}")
if echo "$response" | grep -q "streams"; then
    echo -e "${GREEN}✓ OK${NC} (hay logs)"
else
    echo -e "${YELLOW}⚠ Sin logs aún${NC} (normal si acabas de iniciar)"
fi
echo ""

# Verificar Prometheus
echo "📈 Verificando Prometheus..."
check_service "prometheus" "http://localhost:9090/-/healthy" "Prometheus health"
check_service "prometheus" "http://localhost:9090/api/v1/status/config" "Prometheus config"

# Verificar targets de Prometheus
echo -n "  Verificando targets de Prometheus... "
targets=$(curl -s "http://localhost:9090/api/v1/targets" | grep -o '"health":"up"' | wc -l)
if [ "$targets" -gt 0 ]; then
    echo -e "${GREEN}✓ OK${NC} ($targets targets activos)"
else
    echo -e "${YELLOW}⚠ Sin targets activos${NC}"
fi

# Verificar que Prometheus puede scrapear
echo -n "  Verificando scrape de Prometheus... "
if curl -s "http://localhost:9090/api/v1/query?query=up" | grep -q "result"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FALLO${NC}"
fi
echo ""

# Verificar Promtail
echo "📥 Verificando Promtail..."
if docker-compose ps promtail | grep -q "Up"; then
    echo -e "  ${GREEN}✓${NC} Promtail está corriendo"
    
    # Verificar logs de Promtail
    echo -n "  Verificando logs de Promtail... "
    if docker-compose logs promtail 2>&1 | grep -q "msg=\"Starting Promtail\""; then
        echo -e "${GREEN}✓ OK${NC}"
    else
        echo -e "${YELLOW}⚠ Revisar logs${NC}"
    fi
else
    echo -e "  ${RED}✗${NC} Promtail NO está corriendo"
fi
echo ""

# Verificar Grafana
echo "📊 Verificando Grafana..."
check_service "grafana" "http://localhost:3000/api/health" "Grafana health"

# Verificar datasources de Grafana
echo -n "  Verificando datasources de Grafana... "
if curl -s -u admin:admin "http://localhost:3000/api/datasources" | grep -q "Loki"; then
    echo -e "${GREEN}✓ OK${NC} (Loki configurado)"
else
    echo -e "${YELLOW}⚠ Datasources no configurados${NC}"
fi
echo ""

# Verificar archivos de log
echo "📝 Verificando archivos de log..."
if [ -d "./logs" ]; then
    if [ -f "./logs/server-notifications.log" ]; then
        log_lines=$(wc -l < ./logs/server-notifications.log 2>/dev/null || echo "0")
        echo -e "  ${GREEN}✓${NC} Archivo de log existe ($log_lines líneas)"
    else
        echo -e "  ${YELLOW}⚠${NC} Archivo de log no existe aún"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Directorio logs no existe"
fi
echo ""

# Resumen
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Resumen:"
echo ""
echo "Loki:      http://localhost:3100"
echo "Prometheus: http://localhost:9090"
echo "Grafana:   http://localhost:3000 (admin/admin)"
echo ""
echo "Para ver logs en tiempo real:"
echo "  docker-compose logs -f server-notifications"
echo ""
echo "Para consultar logs en Loki:"
echo "  curl 'http://localhost:3100/loki/api/v1/query?query={job=\"server-notifications\"}'"
echo ""

