#!/bin/bash

# Script para debuggear el contenedor que se reinicia constantemente

echo "🔍 Deteniendo el contenedor para debug..."
docker compose stop server-notifications

echo "📋 Ver logs del último intento:"
docker compose logs --tail=50 server-notifications

echo ""
echo "🔍 Intentando ejecutar el contenedor sin restart para ver el error:"
docker compose run --rm --entrypoint /bin/bash server-notifications -c "
  echo 'Verificando binario...'
  ls -lh /usr/local/bin/server || echo 'Binario no existe!'
  
  echo ''
  echo 'Verificando variables de entorno...'
  env | grep -E 'MONGODB|JWT|GETSTREAM|AWS|API_PORT' || echo 'Variables no encontradas'
  
  echo ''
  echo 'Intentando ejecutar el binario...'
  /usr/local/bin/server || echo 'Error al ejecutar binario'
"

