# ===== Builder =====
FROM rust:1.91 as builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN cargo build --release || true

# Build
COPY . .
RUN cargo build --release

# ===== Runtime =====
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/server-notifications /usr/local/bin/server
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/usr/local/bin/server"]

