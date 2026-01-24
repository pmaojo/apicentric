# syntax=docker/dockerfile:1

# ============================================
# Multi-stage Dockerfile for Apicentric
# Builds both Rust backend and Next.js frontend
# ============================================

# Stage 1: Build Frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/webui

# Copy package files
COPY webui/package*.json ./
RUN npm ci --only=production

# Copy frontend source
COPY webui/ ./

# Build Next.js application
RUN npm run build

# Stage 2: Build Backend
FROM rust:1.75-bookworm AS backend-builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/
COPY examples/ ./examples/

# Build with all features for cloud deployment
RUN cargo build --release --bin apicentric

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /app/target/release/apicentric /usr/local/bin/apicentric

# Copy frontend build
COPY --from=frontend-builder /app/webui/.next/standalone ./webui
COPY --from=frontend-builder /app/webui/.next/static ./webui/.next/static
COPY --from=frontend-builder /app/webui/public ./webui/public

# Create directories for data and services
RUN mkdir -p /app/data /app/services

# Set environment variables
ENV APICENTRIC_PORT=8000
ENV APICENTRIC_HOST=0.0.0.0
ENV APICENTRIC_DATA_DIR=/app/data
ENV APICENTRIC_SERVICES_DIR=/app/services
ENV RUST_LOG=info,apicentric=debug

# Expose ports
# 8000: Backend API and WebSocket
# 3000: Frontend (if running separately)
EXPOSE 8000 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# Default command: start cloud server
CMD ["apicentric", "cloud", "--port", "8000"]
