# ============================================
# Stage 1: Builder
# ============================================
FROM rust:1.98-slim-bookworm AS builder

# نصب پیش‌نیازهای سیستمی
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# تنظیم پوشه کاری
WORKDIR /app

# کپی فایل‌های workspace
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY apps/biz-api/ ./apps/biz-api/
COPY apps/map-api/ ./apps/map-api/
COPY apps/geo-importer/ ./apps/geo-importer/
COPY migrations/ ./migrations/

# Build فقط biz-api
RUN cargo build --release --bin biz-api

# ============================================
# Stage 2: Runtime
# ============================================
FROM debian:bookworm-slim

# نصب پیش‌نیازهای runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# کپی باینری از مرحله builder
COPY --from=builder /app/target/release/biz-api /usr/local/bin/biz-api

# پورت
EXPOSE 8081

# اجرا
CMD ["biz-api"]
