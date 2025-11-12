//! Monitoring and observability utilities for the cloud server.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Application metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests (2xx status codes)
    pub successful_requests: u64,
    /// Number of failed requests (4xx, 5xx status codes)
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Number of active WebSocket connections
    pub active_websocket_connections: u64,
    /// Number of active services
    pub active_services: u64,
    /// Total number of log entries
    pub total_log_entries: u64,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes (if available)
    pub memory_usage_bytes: Option<u64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            active_websocket_connections: 0,
            active_services: 0,
            total_log_entries: 0,
            uptime_seconds: 0,
            memory_usage_bytes: None,
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<Metrics>>,
    start_time: Instant,
    response_times: Arc<RwLock<Vec<Duration>>>,
}

impl MetricsCollector {
    /// Creates a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
            start_time: Instant::now(),
            response_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Records a request
    pub async fn record_request(&self, duration: Duration, status_code: u16) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        if status_code >= 200 && status_code < 300 {
            metrics.successful_requests += 1;
        } else if status_code >= 400 {
            metrics.failed_requests += 1;
        }

        // Update response times
        let mut response_times = self.response_times.write().await;
        response_times.push(duration);

        // Keep only last 1000 response times
        let len = response_times.len();
        if len > 1000 {
            response_times.drain(0..len - 1000);
        }

        // Calculate average
        let total_ms: f64 = response_times.iter().map(|d| d.as_millis() as f64).sum();
        metrics.avg_response_time_ms = total_ms / response_times.len() as f64;
    }

    /// Updates WebSocket connection count
    pub async fn set_websocket_connections(&self, count: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.active_websocket_connections = count;
    }

    /// Updates active services count
    pub async fn set_active_services(&self, count: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.active_services = count;
    }

    /// Updates log entries count
    pub async fn set_log_entries(&self, count: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_log_entries = count;
    }

    /// Gets current metrics
    pub async fn get_metrics(&self) -> Metrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();

        // Try to get memory usage
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                metrics.memory_usage_bytes = Some(kb_val * 1024);
                            }
                        }
                        break;
                    }
                }
            }
        }

        metrics
    }

    /// Resets metrics
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = Metrics::default();
        let mut response_times = self.response_times.write().await;
        response_times.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLog {
    /// Timestamp
    pub timestamp: String,
    /// Log level
    pub level: String,
    /// Message
    pub message: String,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Request ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// User ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl StructuredLog {
    /// Creates a new structured log entry
    pub fn new(level: &str, message: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            context: None,
            request_id: None,
            user_id: None,
        }
    }

    /// Adds context to the log entry
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Adds request ID to the log entry
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Adds user ID to the log entry
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Logs the entry
    pub fn log(&self) {
        let json = serde_json::to_string(self).unwrap_or_else(|_| {
            format!("{{\"level\":\"{}\",\"message\":\"{}\"}}", self.level, self.message)
        });

        match self.level.as_str() {
            "error" => tracing::error!("{}", json),
            "warn" => tracing::warn!("{}", json),
            "info" => tracing::info!("{}", json),
            "debug" => tracing::debug!("{}", json),
            "trace" => tracing::trace!("{}", json),
            _ => tracing::info!("{}", json),
        }
    }
}

/// Macro for structured logging
#[macro_export]
macro_rules! log_structured {
    ($level:expr, $message:expr) => {
        $crate::cloud::monitoring::StructuredLog::new($level, $message).log()
    };
    ($level:expr, $message:expr, $context:expr) => {
        $crate::cloud::monitoring::StructuredLog::new($level, $message)
            .with_context($context)
            .log()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        // Record some requests
        collector.record_request(Duration::from_millis(100), 200).await;
        collector.record_request(Duration::from_millis(200), 200).await;
        collector.record_request(Duration::from_millis(150), 500).await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert!(metrics.avg_response_time_ms > 0.0);
    }

    #[test]
    fn test_structured_log() {
        let log = StructuredLog::new("info", "Test message")
            .with_context(serde_json::json!({"key": "value"}))
            .with_request_id("req-123".to_string());

        assert_eq!(log.level, "info");
        assert_eq!(log.message, "Test message");
        assert!(log.context.is_some());
        assert_eq!(log.request_id, Some("req-123".to_string()));
    }
}
