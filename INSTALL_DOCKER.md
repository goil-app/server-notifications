# Instalación de Docker y Docker Compose en Ubuntu

## Instalación de Docker

### 1. Actualizar el sistema
```bash
sudo apt-get update
sudo apt-get upgrade -y
```

### 2. Instalar dependencias necesarias
```bash
sudo apt-get install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release
```

### 3. Añadir la clave GPG oficial de Docker
```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg
```

### 4. Configurar el repositorio de Docker
```bash
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
```

### 5. Instalar Docker Engine
```bash
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

### 6. Verificar la instalación
```bash
sudo docker --version
sudo docker compose version
```

### 7. (Opcional) Añadir tu usuario al grupo docker para no usar sudo
```bash
sudo usermod -aG docker $USER
```

**Importante:** Después de añadir tu usuario al grupo docker, necesitas cerrar sesión y volver a iniciar sesión, o ejecutar:
```bash
newgrp docker
```

## Verificación

### Verificar que Docker funciona
```bash
sudo docker run hello-world
```

### Verificar que Docker Compose funciona
```bash
sudo docker compose version
```

## Uso

Ahora puedes usar Docker Compose sin sudo (si añadiste tu usuario al grupo docker):

```bash
# Levantar servicios
docker compose up -d

# Ver estado
docker compose ps

# Ver logs
docker compose logs -f

# Detener servicios
docker compose down
```

## Nota sobre Docker Compose V2

Las versiones modernas de Docker incluyen Docker Compose V2 como plugin (`docker compose`), que es diferente de la versión antigua (`docker-compose` con guión).

Si encuentras comandos antiguos que usan `docker-compose` (con guión), puedes:
1. Usar `docker compose` (sin guión) - recomendado
2. O crear un alias: `alias docker-compose='docker compose'`

## Troubleshooting

### Si tienes problemas con permisos:
```bash
# Verificar que estás en el grupo docker
groups

# Si no estás, añadirte de nuevo
sudo usermod -aG docker $USER
newgrp docker
```

### Si Docker no inicia:
```bash
sudo systemctl start docker
sudo systemctl enable docker
```

### Verificar estado del servicio Docker:
```bash
sudo systemctl status docker
```

