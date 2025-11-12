# Monitoring and Observability Guide

This guide covers monitoring, logging, and observability features in Apicentric.

## Table of Contents

- [Health Checks](#health-checks)
- [Metrics](#metrics)
- [Logging](#logging)
- [Error Tracking](#error-tracking)
- [Performance Monitoring](#performance-monitoring)
- [Alerting](#alerting)

## Health Checks

### Basic Health Check

The application exposes a health check endpoint:

```bash
curl http://localhost:8000/health
```

Response:

```json
{
  "status": "healthy",
  "service": "apicentric-cloud",
  "version": "1.0.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "uptime_seconds": 3600
}
```

### Docker Health Check

The Docker container includes a built-in health check:

```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:8000/health || exit 1"]
  interval: 30s
  timeout: 3s
  retries: 3
  start_period: 5s
```

Check container health:

```bash
docker ps
# Look for "healthy" status

docker inspect apicentric | jq '.[0].State.Health'
```

### Kubernetes Liveness and Readiness Probes

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8000
  initialDelaySeconds: 10
  periodSeconds: 30
  timeoutSeconds: 3
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health
    port: 8000
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
```

## Metrics

### Application Metrics

Get current application metrics:

```bash
curl http://localhost:8000/api/metrics
```

Response:

```json
{
  "success": true,
  "data": {
    "total_requests": 1234,
    "successful_requests": 1200,
    "failed_requests": 34,
    "avg_response_time_ms": 45.2,
    "active_websocket_connections": 5,
    "active_services": 3,
    "total_log_entries": 5678,
    "uptime_seconds": 7200,
    "memory_usage_bytes": 52428800
  }
}
```

### Metrics Breakdown

#### Request Metrics
- **total_requests**: Total number of API requests processed
- **successful_requests**: Requests with 2xx status codes
- **failed_requests**: Requests with 4xx or 5xx status codes
- **avg_response_time_ms**: Average response time in milliseconds

#### Service Metrics
- **active_services**: Number of currently running services
- **total_log_entries**: Total number of request logs

#### Connection Metrics
- **active_websocket_connections**: Number of active WebSocket connections

#### System Metrics
- **uptime_seconds**: Server uptime in seconds
- **memory_usage_bytes**: Current memory usage (Linux only)

### Prometheus Integration

To expose metrics in Prometheus format, add the following to your `Cargo.toml`:

```toml
[dependencies]
prometheus = "0.13"
```

Then create a metrics endpoint:

```rust
use prometheus::{Encoder, TextEncoder, Registry};

async fn prometheus_metrics() -> impl IntoResponse {
    let registry = Registry::new();
    
    // Register metrics
    let requests_total = IntCounter::new("requests_total", "Total requests").unwrap();
    registry.register(Box::new(requests_total.clone())).unwrap();
    
    // Encode metrics
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        buffer,
    )
}
```

Add to router:

```rust
.route("/metrics", get(prometheus_metrics))
```

Configure Prometheus to scrape:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'apicentric'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Logging

### Log Levels

Configure log level via environment variable:

```bash
# Options: error, warn, info, debug, trace
export RUST_LOG=info,apicentric=debug
```

### Structured Logging

The application uses structured JSON logging:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "message": "Service started",
  "context": {
    "service_name": "user-api",
    "port": 8080
  },
  "request_id": "req-123",
  "user_id": "user-456"
}
```

### Log to File

Redirect logs to a file:

```bash
apicentric cloud --port 8000 2>&1 | tee apicentric.log
```

### Log Rotation

Use `logrotate` for log rotation:

```bash
# /etc/logrotate.d/apicentric
/var/log/apicentric/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0644 apicentric apicentric
    postrotate
        systemctl reload apicentric
    endscript
}
```

### Centralized Logging

#### Using Fluentd

```yaml
# docker-compose.yml
services:
  apicentric:
    logging:
      driver: fluentd
      options:
        fluentd-address: localhost:24224
        tag: apicentric

  fluentd:
    image: fluent/fluentd:latest
    ports:
      - "24224:24224"
    volumes:
      - ./fluentd.conf:/fluentd/etc/fluent.conf
```

#### Using ELK Stack

```yaml
# docker-compose.yml
services:
  apicentric:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"

  filebeat:
    image: docker.elastic.co/beats/filebeat:8.0.0
    volumes:
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./filebeat.yml:/usr/share/filebeat/filebeat.yml:ro
```

## Error Tracking

### Error Response Format

All API errors follow a consistent format:

```json
{
  "error": "Service not found",
  "code": "SERVICE_NOT_FOUND",
  "details": {
    "service_name": "unknown-api"
  }
}
```

### Error Codes

Common error codes:

- `SERVICE_NOT_FOUND`: Service does not exist
- `SERVICE_ALREADY_RUNNING`: Service is already running
- `INVALID_YAML`: YAML syntax error
- `AI_NOT_CONFIGURED`: AI provider not configured
- `AUTHENTICATION_REQUIRED`: Authentication required
- `INVALID_TOKEN`: Invalid or expired JWT token
- `VALIDATION_ERROR`: Input validation failed

### Sentry Integration

Add Sentry for error tracking:

```toml
[dependencies]
sentry = "0.31"
```

Initialize Sentry:

```rust
let _guard = sentry::init((
    "https://your-dsn@sentry.io/project-id",
    sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    },
));
```

Capture errors:

```rust
sentry::capture_error(&error);
```

## Performance Monitoring

### Response Time Tracking

Track response times using middleware:

```rust
use tower_http::trace::TraceLayer;
use std::time::Instant;

let app = Router::new()
    .layer(
        TraceLayer::new_for_http()
            .on_request(|request: &Request<_>, _span: &Span| {
                tracing::info!("request: {} {}", request.method(), request.uri());
            })
            .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                tracing::info!(
                    "response: {} {}ms",
                    response.status(),
                    latency.as_millis()
                );
            })
    );
```

### Database Query Performance

Monitor SQLite query performance:

```rust
let start = Instant::now();
let result = conn.execute(query, params)?;
let duration = start.elapsed();

if duration > Duration::from_millis(100) {
    tracing::warn!(
        "Slow query: {}ms - {}",
        duration.as_millis(),
        query
    );
}
```

### Memory Profiling

Use `heaptrack` for memory profiling:

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Run with profiling
heaptrack ./target/release/apicentric cloud --port 8000

# Analyze results
heaptrack_gui heaptrack.apicentric.*.gz
```

### CPU Profiling

Use `perf` for CPU profiling:

```bash
# Record
sudo perf record -F 99 -g ./target/release/apicentric cloud --port 8000

# Report
sudo perf report
```

## Alerting

### Prometheus Alerting Rules

```yaml
# alerts.yml
groups:
  - name: apicentric
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(requests_failed_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} requests/sec"

      - alert: ServiceDown
        expr: up{job="apicentric"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Apicentric service is down"
          description: "Service has been down for more than 1 minute"

      - alert: HighMemoryUsage
        expr: memory_usage_bytes > 1073741824  # 1GB
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }} bytes"
```

### Alertmanager Configuration

```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'email'

receivers:
  - name: 'email'
    email_configs:
      - to: 'alerts@example.com'
        from: 'alertmanager@example.com'
        smarthost: 'smtp.example.com:587'
        auth_username: 'alertmanager@example.com'
        auth_password: 'password'
```

### Health Check Monitoring

Use a service like UptimeRobot or Pingdom:

```bash
# Endpoint to monitor
https://yourdomain.com/health

# Expected response
Status: 200 OK
Body contains: "healthy"
```

### Custom Alerts

Create custom alert scripts:

```bash
#!/bin/bash
# check-apicentric.sh

HEALTH_URL="http://localhost:8000/health"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $HEALTH_URL)

if [ $RESPONSE -ne 200 ]; then
    echo "CRITICAL: Apicentric health check failed (HTTP $RESPONSE)"
    # Send alert (email, Slack, PagerDuty, etc.)
    exit 2
fi

echo "OK: Apicentric is healthy"
exit 0
```

Add to cron:

```bash
# Check every 5 minutes
*/5 * * * * /usr/local/bin/check-apicentric.sh
```

## Dashboards

### Grafana Dashboard

Create a Grafana dashboard with panels for:

1. **Request Rate**: Requests per second
2. **Error Rate**: Errors per second
3. **Response Time**: P50, P95, P99 latencies
4. **Active Services**: Number of running services
5. **Memory Usage**: Current memory usage
6. **CPU Usage**: Current CPU usage
7. **WebSocket Connections**: Active connections

Example Grafana query:

```promql
# Request rate
rate(requests_total[5m])

# Error rate
rate(requests_failed_total[5m])

# Response time P95
histogram_quantile(0.95, rate(response_time_bucket[5m]))
```

### Custom Dashboard

Create a simple HTML dashboard:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Apicentric Monitoring</title>
    <script>
        async function updateMetrics() {
            const response = await fetch('http://localhost:8000/api/metrics');
            const data = await response.json();
            
            document.getElementById('requests').textContent = data.data.total_requests;
            document.getElementById('errors').textContent = data.data.failed_requests;
            document.getElementById('services').textContent = data.data.active_services;
        }
        
        setInterval(updateMetrics, 5000);
        updateMetrics();
    </script>
</head>
<body>
    <h1>Apicentric Metrics</h1>
    <p>Total Requests: <span id="requests">-</span></p>
    <p>Failed Requests: <span id="errors">-</span></p>
    <p>Active Services: <span id="services">-</span></p>
</body>
</html>
```

## Best Practices

1. **Set up health checks** for all deployment environments
2. **Monitor key metrics** (requests, errors, latency, memory)
3. **Use structured logging** for easier parsing and analysis
4. **Set up alerts** for critical issues (service down, high error rate)
5. **Regularly review logs** for patterns and anomalies
6. **Profile performance** in production-like environments
7. **Track error rates** and investigate spikes
8. **Monitor resource usage** (CPU, memory, disk)
9. **Set up log rotation** to prevent disk space issues
10. **Use centralized logging** for distributed deployments

## Troubleshooting

### High Memory Usage

```bash
# Check memory usage
docker stats apicentric

# Get detailed memory info
docker exec apicentric cat /proc/self/status | grep Vm
```

### High CPU Usage

```bash
# Check CPU usage
docker stats apicentric

# Profile CPU
sudo perf record -p $(pgrep apicentric) -g -- sleep 30
sudo perf report
```

### Slow Responses

```bash
# Check response times in logs
grep "response:" apicentric.log | awk '{print $NF}' | sort -n | tail -10

# Enable debug logging
export RUST_LOG=trace
```

### Database Issues

```bash
# Check database size
ls -lh data/apicentric.db

# Vacuum database
sqlite3 data/apicentric.db "VACUUM;"

# Check for locks
lsof data/apicentric.db
```

## Support

For monitoring and observability questions:
- GitHub Issues: https://github.com/yourusername/apicentric/issues
- Documentation: https://docs.apicentric.io
- Community: https://discord.gg/apicentric
