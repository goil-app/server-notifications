#!/bin/bash

# Script de instalación de Docker y Docker Compose en Debian
# Ejecutar con: bash install-docker.sh

set -e

echo "🚀 Instalando Docker y Docker Compose en Debian..."

# 1. Actualizar el sistema
echo "📦 Actualizando sistema..."
sudo apt-get update
sudo apt-get upgrade -y

# 2. Instalar dependencias necesarias
echo "📦 Instalando dependencias..."
sudo apt-get install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# 3. Añadir la clave GPG oficial de Docker
echo "🔑 Añadiendo clave GPG de Docker..."
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# 4. Configurar el repositorio de Docker
echo "📝 Configurando repositorio de Docker..."
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 5. Instalar Docker Engine
echo "📦 Instalando Docker Engine..."
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# 6. Añadir usuario al grupo docker (opcional)
echo "👤 Añadiendo usuario al grupo docker..."
sudo usermod -aG docker $USER

# 7. Iniciar y habilitar Docker
echo "▶️  Iniciando servicio Docker..."
sudo systemctl start docker
sudo systemctl enable docker

# 8. Verificar instalación
echo "✅ Verificando instalación..."
echo ""
echo "Docker version:"
sudo docker --version
echo ""
echo "Docker Compose version:"
sudo docker compose version
echo ""

echo "✅ Instalación completada!"
echo ""
echo "⚠️  IMPORTANTE: Necesitas cerrar sesión y volver a iniciar sesión,"
echo "    o ejecutar 'newgrp docker' para usar Docker sin sudo."
echo ""
echo "Para probar Docker:"
echo "  sudo docker run hello-world"
echo ""
echo "Para probar Docker Compose:"
echo "  sudo docker compose version"

