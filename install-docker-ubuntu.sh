#!/bin/bash

# Script de instalación de Docker y Docker Compose en Ubuntu
# Ejecutar con: bash install-docker-ubuntu.sh

set -e

echo "🚀 Instalando Docker y Docker Compose en Ubuntu..."

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

# 3. Eliminar configuración anterior si existe
echo "🧹 Limpiando configuración anterior..."
sudo rm -f /etc/apt/keyrings/docker.gpg
sudo rm -f /etc/apt/sources.list.d/docker.list

# 4. Añadir la clave GPG oficial de Docker (UBUNTU)
echo "🔑 Añadiendo clave GPG de Docker..."
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# 5. Configurar el repositorio de Docker (UBUNTU)
echo "📝 Configurando repositorio de Docker..."
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 6. Instalar Docker Engine
echo "📦 Instalando Docker Engine..."
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# 7. Añadir usuario al grupo docker (opcional)
echo "👤 Añadiendo usuario al grupo docker..."
sudo usermod -aG docker $USER

# 8. Iniciar y habilitar Docker
echo "▶️  Iniciando servicio Docker..."
sudo systemctl start docker
sudo systemctl enable docker

# 9. Verificar instalación
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

