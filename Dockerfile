# ===== Builder =====
# Usar Debian Bookworm para compilar y asegurar compatibilidad con el runtime
FROM rust:1.91-bookworm as builder
WORKDIR /app

# Instalar dependencias necesarias para compilar
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN cargo build --release || true

# Build
COPY . .
RUN cargo build --release

# Verificar que el binario se compiló correctamente
RUN ls -lh /app/target/release/server-notifications && \
    file /app/target/release/server-notifications

# ===== Runtime =====
# Usar Debian Bookworm (estable) para compatibilidad con servidores Debian
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y ca-certificates libc6 && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/server-notifications /usr/local/bin/server

# Verificar que el binario existe, es ejecutable y tiene las dependencias correctas
RUN chmod +x /usr/local/bin/server && \
    ls -lh /usr/local/bin/server && \
    file /usr/local/bin/server && \
    ldd /usr/local/bin/server || echo "Binario estático o sin dependencias dinámicas"

ENV RUST_LOG=info
EXPOSE 8080

# Crear directorio de logs si no existe
RUN mkdir -p /app/logs

# Usar shell form para capturar errores y mantener el proceso corriendo
# El proceso debe mantenerse en foreground
CMD ["/bin/sh", "-c", "echo 'Starting server...' && /usr/local/bin/server 2>&1 || (echo 'Server exited with code:' $? && sleep 5 && exit 1)"]

