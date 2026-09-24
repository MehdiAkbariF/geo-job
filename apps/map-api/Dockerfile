# ============================================
# Stage 1: Builder
# ============================================
FROM rust:1.80-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# کپی فایل‌های workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY apps/biz-api/ ./apps/biz-api/
COPY apps/map-api/ ./apps/map-api/
COPY apps/geo-importer/ ./apps/geo-importer/

# Build فقط map-api
RUN cargo build --release --bin map-api

# ============================================
# Stage 2: Runtime
# ============================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# کپی باینری از مرحله builder
COPY --from=builder /app/target/release/map-api /usr/local/bin/map-api

# پورت
EXPOSE 8080

# اجرا
CMD ["map-api"]
