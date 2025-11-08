#!/bin/bash

# Script de desinstalación de Docker y Docker Compose en Ubuntu
# Ejecutar con: bash uninstall-docker.sh

set -e

echo "🗑️  Desinstalando Docker y Docker Compose..."

# 1. Detener todos los contenedores y servicios Docker
echo "🛑 Deteniendo contenedores..."
sudo docker stop $(sudo docker ps -aq) 2>/dev/null || true
sudo docker rm $(sudo docker ps -aq) 2>/dev/null || true

# 2. Eliminar imágenes, volúmenes y redes
echo "🧹 Eliminando imágenes, volúmenes y redes..."
sudo docker rmi $(sudo docker images -q) 2>/dev/null || true
sudo docker volume prune -a -f 2>/dev/null || true
sudo docker network prune -f 2>/dev/null || true

# 3. Desinstalar paquetes de Docker
echo "📦 Desinstalando paquetes..."
sudo apt-get purge -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || true

# 4. Eliminar imágenes, contenedores y volúmenes
echo "🗑️  Eliminando datos de Docker..."
sudo rm -rf /var/lib/docker
sudo rm -rf /var/lib/containerd

# 5. Eliminar archivos de configuración
echo "🗑️  Eliminando configuración..."
sudo rm -rf /etc/docker
rm -rf ~/.docker

# 6. Eliminar repositorio de Docker
echo "🗑️  Eliminando repositorio..."
sudo rm -f /etc/apt/keyrings/docker.gpg
sudo rm -f /etc/apt/sources.list.d/docker.list

# 7. Limpiar dependencias no utilizadas
echo "🧹 Limpiando dependencias..."
sudo apt-get autoremove -y
sudo apt-get autoclean

# 8. Eliminar grupo docker (si existe)
echo "👤 Eliminando grupo docker..."
sudo groupdel docker 2>/dev/null || true

echo "✅ Desinstalación completada!"
echo ""
echo "Para verificar que Docker se eliminó:"
echo "  docker --version"
echo "  (debería mostrar 'command not found')"

