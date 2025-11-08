#!/bin/bash

# Script para probar el build de Docker y verificar el binario

echo "🔨 Construyendo imagen Docker..."
docker compose build server-notifications

echo ""
echo "🔍 Verificando que el binario existe en la imagen..."
docker compose run --rm --entrypoint /bin/bash server-notifications -c "
  echo 'Verificando binario...'
  ls -lh /usr/local/bin/server
  echo ''
  echo 'Información del binario:'
  file /usr/local/bin/server
  echo ''
  echo 'Dependencias del binario:'
  ldd /usr/local/bin/server 2>&1 || echo 'Binario estático'
  echo ''
  echo 'Intentando ejecutar el binario con --help (si lo soporta)...'
  /usr/local/bin/server --help 2>&1 || echo 'No tiene --help, continuando...'
"

echo ""
echo "✅ Si el binario existe y tiene las dependencias correctas,"
echo "   el problema puede estar en las variables de entorno o MongoDB."
echo ""
echo "Para probar ejecutando el servidor:"
echo "  docker compose run --rm server-notifications"

