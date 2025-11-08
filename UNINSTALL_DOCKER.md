# Desinstalación de Docker y Docker Compose en Debian

## Desinstalación Completa

### 1. Detener todos los contenedores y servicios Docker
```bash
# Detener todos los contenedores en ejecución
sudo docker stop $(sudo docker ps -aq)

# Eliminar todos los contenedores
sudo docker rm $(sudo docker ps -aq)

# Eliminar todas las imágenes
sudo docker rmi $(sudo docker images -q)

# Eliminar todos los volúmenes (⚠️ esto borra datos)
sudo docker volume prune -a -f

# Eliminar todas las redes personalizadas
sudo docker network prune -f
```

### 2. Desinstalar paquetes de Docker
```bash
sudo apt-get purge -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

### 3. Eliminar imágenes, contenedores y volúmenes (opcional)
```bash
sudo rm -rf /var/lib/docker
sudo rm -rf /var/lib/containerd
```

### 4. Eliminar archivos de configuración
```bash
sudo rm -rf /etc/docker
sudo rm -rf ~/.docker
```

### 5. Eliminar repositorio de Docker
```bash
sudo rm -f /etc/apt/keyrings/docker.gpg
sudo rm -f /etc/apt/sources.list.d/docker.list
```

### 6. Limpiar dependencias no utilizadas
```bash
sudo apt-get autoremove -y
sudo apt-get autoclean
```

### 7. Eliminar grupo docker (si existe)
```bash
sudo groupdel docker
```

## Desinstalación Parcial (solo Docker Compose)

Si solo quieres eliminar Docker Compose pero mantener Docker:

```bash
sudo apt-get purge -y docker-compose-plugin
```

## Verificación

Verificar que Docker se ha eliminado completamente:

```bash
# Estos comandos deberían fallar o no encontrar docker
docker --version
docker compose version
which docker
```

## Limpieza Adicional (Opcional)

Si quieres eliminar también los datos de usuario:

```bash
# Eliminar configuración de usuario
rm -rf ~/.docker

# Eliminar logs (si existen)
sudo rm -rf /var/log/docker
```

## Nota

Si añadiste tu usuario al grupo `docker`, el grupo puede seguir existiendo pero sin miembros. Esto no causa problemas, pero si quieres eliminarlo completamente:

```bash
# Verificar si el grupo tiene miembros
getent group docker

# Si está vacío, eliminarlo
sudo groupdel docker
```

