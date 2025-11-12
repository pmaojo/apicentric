# Deployment Guide

This guide covers deploying Apicentric with the web GUI in various environments.

## Table of Contents

- [Quick Start](#quick-start)
- [Docker Deployment](#docker-deployment)
- [Production Deployment](#production-deployment)
- [Environment Configuration](#environment-configuration)
- [Reverse Proxy Setup](#reverse-proxy-setup)
- [Monitoring and Logging](#monitoring-and-logging)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Development Mode

Start both backend and frontend in development mode:

```bash
# Using Docker Compose
docker-compose --profile dev up

# Or manually
# Terminal 1: Backend
cargo run --features gui,cloud -- cloud --port 8000

# Terminal 2: Frontend
cd webui && npm run dev
```

Access the application:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8000
- WebSocket: ws://localhost:8000/ws

### Production Mode

Build and run the production container:

```bash
# Build the image
docker build -t apicentric:latest .

# Run the container
docker run -p 8000:8000 \
  -e APICENTRIC_JWT_SECRET=$(openssl rand -base64 32) \
  -e APICENTRIC_PROTECT_SERVICES=true \
  -v $(pwd)/services:/app/services \
  -v apicentric-data:/app/data \
  apicentric:latest
```

Access the application at http://localhost:8000

## Docker Deployment

### Using Docker Compose

#### Development Environment

```bash
# Start development services
docker-compose --profile dev up

# Stop services
docker-compose --profile dev down
```

This starts:
- Backend with hot-reload on port 8000
- Frontend with hot-reload on port 3000

#### Production Environment

```bash
# Start production services
docker-compose --profile production up -d

# View logs
docker-compose --profile production logs -f

# Stop services
docker-compose --profile production down
```

This starts:
- Combined backend + frontend on port 8000
- Nginx reverse proxy on ports 80/443 (optional)

### Custom Docker Build

Build with specific features:

```bash
# Build backend only
docker build --target backend-builder -t apicentric-backend .

# Build frontend only
docker build --target frontend-builder -t apicentric-frontend .

# Build complete image
docker build -t apicentric:latest .
```

## Production Deployment

### Prerequisites

1. **Generate JWT Secret**
   ```bash
   openssl rand -base64 32
   ```

2. **Create Environment File**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Prepare Service Definitions**
   ```bash
   mkdir -p services
   # Add your YAML service definitions
   ```

### Deployment Options

#### Option 1: Single Container (Recommended)

Deploy the combined backend + frontend container:

```bash
docker run -d \
  --name apicentric \
  -p 8000:8000 \
  -e APICENTRIC_JWT_SECRET=${JWT_SECRET} \
  -e APICENTRIC_PROTECT_SERVICES=true \
  -e ALLOWED_ORIGINS=https://yourdomain.com \
  -v $(pwd)/services:/app/services \
  -v apicentric-data:/app/data \
  --restart unless-stopped \
  apicentric:latest
```

#### Option 2: Separate Services

Deploy backend and frontend separately:

```bash
# Backend
docker run -d \
  --name apicentric-backend \
  -p 8000:8000 \
  -e APICENTRIC_JWT_SECRET=${JWT_SECRET} \
  -v $(pwd)/services:/app/services \
  apicentric-backend:latest

# Frontend (if built separately)
docker run -d \
  --name apicentric-frontend \
  -p 3000:3000 \
  -e NEXT_PUBLIC_API_URL=http://backend:8000 \
  --link apicentric-backend:backend \
  apicentric-frontend:latest
```

#### Option 3: Kubernetes

See [kubernetes/README.md](../kubernetes/README.md) for Kubernetes deployment.

### Static File Serving

The Rust backend can serve the frontend static files:

```rust
// In src/cloud/server.rs
use tower_http::services::ServeDir;

let app = Router::new()
    .nest("/api", api_routes())
    .nest_service("/", ServeDir::new("webui/dist"));
```

Build frontend for static serving:

```bash
cd webui
npm run build
npm run export  # Generates static files in out/
```

## Environment Configuration

### Required Variables

```bash
# Authentication
APICENTRIC_JWT_SECRET=<secure-random-string>

# Server
APICENTRIC_PORT=8000
APICENTRIC_HOST=0.0.0.0
```

### Optional Variables

```bash
# Authentication
APICENTRIC_PROTECT_SERVICES=true  # Require auth for all endpoints

# Database
APICENTRIC_DB_PATH=/app/data/apicentric.db

# CORS
ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com

# Logging
RUST_LOG=info,apicentric=debug

# AI (optional)
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=...
```

### Frontend Variables

```bash
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8000/ws
NODE_ENV=production
```

## Reverse Proxy Setup

### Nginx

Create `nginx.conf`:

```nginx
upstream apicentric_backend {
    server apicentric:8000;
}

server {
    listen 80;
    server_name yourdomain.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/cert.pem;
    ssl_certificate_key /etc/ssl/certs/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # API endpoints
    location /api/ {
        proxy_pass http://apicentric_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket
    location /ws {
        proxy_pass http://apicentric_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }

    # Frontend
    location / {
        proxy_pass http://apicentric_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
    }
}
```

### Traefik

Create `docker-compose.traefik.yml`:

```yaml
version: '3.8'

services:
  apicentric:
    image: apicentric:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.apicentric.rule=Host(`yourdomain.com`)"
      - "traefik.http.routers.apicentric.entrypoints=websecure"
      - "traefik.http.routers.apicentric.tls.certresolver=letsencrypt"
      - "traefik.http.services.apicentric.loadbalancer.server.port=8000"

  traefik:
    image: traefik:v2.10
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.email=your@email.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - letsencrypt:/letsencrypt

volumes:
  letsencrypt:
```

## Monitoring and Logging

### Health Checks

The application exposes health check endpoints:

```bash
# Backend health
curl http://localhost:8000/health

# Response
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

### Logging

Configure structured logging:

```bash
# Set log level
export RUST_LOG=info,apicentric=debug

# Log to file
apicentric cloud --port 8000 2>&1 | tee apicentric.log
```

### Metrics

Monitor application metrics:

```bash
# Container stats
docker stats apicentric

# Application logs
docker logs -f apicentric

# Resource usage
docker exec apicentric ps aux
```

### Prometheus Integration

Expose metrics for Prometheus:

```rust
// In src/cloud/server.rs
use prometheus::{Encoder, TextEncoder};

async fn metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    buffer
}
```

## Troubleshooting

### Common Issues

#### Port Already in Use

```bash
# Find process using port 8000
lsof -i :8000

# Kill process
kill -9 <PID>
```

#### Permission Denied

```bash
# Fix volume permissions
sudo chown -R $(id -u):$(id -g) ./data ./services
```

#### WebSocket Connection Failed

Check CORS and WebSocket configuration:

```bash
# Test WebSocket connection
wscat -c ws://localhost:8000/ws
```

#### Database Locked

```bash
# Stop all containers
docker-compose down

# Remove database lock
rm data/apicentric.db-shm data/apicentric.db-wal

# Restart
docker-compose up
```

### Debug Mode

Run with debug logging:

```bash
RUST_LOG=trace apicentric cloud --port 8000
```

### Container Debugging

```bash
# Enter container
docker exec -it apicentric sh

# Check logs
docker logs apicentric --tail 100 -f

# Inspect container
docker inspect apicentric
```

## Security Best Practices

1. **Use Strong JWT Secret**
   ```bash
   openssl rand -base64 32
   ```

2. **Enable Authentication**
   ```bash
   APICENTRIC_PROTECT_SERVICES=true
   ```

3. **Use HTTPS in Production**
   - Configure SSL certificates
   - Redirect HTTP to HTTPS

4. **Restrict CORS Origins**
   ```bash
   ALLOWED_ORIGINS=https://yourdomain.com
   ```

5. **Regular Updates**
   ```bash
   docker pull apicentric:latest
   docker-compose up -d
   ```

6. **Backup Data**
   ```bash
   docker run --rm -v apicentric-data:/data -v $(pwd):/backup \
     alpine tar czf /backup/apicentric-backup.tar.gz /data
   ```

## Scaling

### Horizontal Scaling

Use a load balancer with multiple instances:

```yaml
services:
  apicentric:
    image: apicentric:latest
    deploy:
      replicas: 3
    environment:
      - APICENTRIC_DB_PATH=/app/data/apicentric.db
```

### Database Considerations

For multiple instances, consider:
- Shared database (PostgreSQL instead of SQLite)
- Redis for session storage
- Distributed file storage for service definitions

## Support

For issues and questions:
- GitHub Issues: https://github.com/yourusername/apicentric/issues
- Documentation: https://docs.apicentric.io
- Community: https://discord.gg/apicentric
