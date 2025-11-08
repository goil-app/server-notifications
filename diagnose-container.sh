#!/bin/bash

# Script para diagnosticar por qué el contenedor se reinicia

echo "🔍 Diagnosticando contenedor server-notifications..."
echo ""

# 1. Detener el contenedor
echo "1️⃣ Deteniendo contenedor..."
docker compose stop server-notifications 2>/dev/null || true

# 2. Ver logs
echo ""
echo "2️⃣ Últimos logs del contenedor:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker compose logs --tail=50 server-notifications
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 3. Verificar binario
echo ""
echo "3️⃣ Verificando binario en la imagen:"
docker compose run --rm --entrypoint /bin/bash server-notifications -c "
  echo 'Verificando existencia...'
  ls -lh /usr/local/bin/server 2>&1 || echo '❌ Binario no existe!'
  
  echo ''
  echo 'Verificando tipo de archivo...'
  file /usr/local/bin/server 2>&1 || echo '❌ No se puede leer el archivo!'
  
  echo ''
  echo 'Verificando dependencias...'
  ldd /usr/local/bin/server 2>&1 || echo 'Binario estático o error al leer dependencias'
  
  echo ''
  echo 'Verificando permisos...'
  ls -l /usr/local/bin/server
" 2>&1 || echo "❌ No se pudo ejecutar el contenedor"

# 4. Verificar variables de entorno
echo ""
echo "4️⃣ Verificando variables de entorno críticas:"
docker compose run --rm --entrypoint /bin/bash server-notifications -c "
  echo 'MONGODB_URI:' \${MONGODB_URI:-\"❌ NO CONFIGURADA\"}
  echo 'JWT_MOBILE_PLATFORM:' \${JWT_MOBILE_PLATFORM:-\"❌ NO CONFIGURADA\"}
  echo 'API_PORT:' \${API_PORT:-\"❌ NO CONFIGURADA (usará 8080)\"}
  echo 'LOG_DIR:' \${LOG_DIR:-\"❌ NO CONFIGURADA\"}
" 2>&1 || echo "❌ No se pudo verificar variables"

# 5. Intentar ejecutar manualmente
echo ""
echo "5️⃣ Intentando ejecutar el binario manualmente:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
timeout 5 docker compose run --rm server-notifications /usr/local/bin/server 2>&1 || echo "Proceso terminó o timeout"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "✅ Diagnóstico completado"
echo ""
echo "📋 Próximos pasos:"
echo "  1. Revisa los logs arriba para ver el error específico"
echo "  2. Verifica que MONGODB_URI esté correctamente configurada"
echo "  3. Si el binario no existe, reconstruye: docker compose build --no-cache server-notifications"

