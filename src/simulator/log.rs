//! Request logging utilities for the simulator

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::storage::Storage;

/// Individual request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogEntry {
    /// Timestamp when the request was processed
    pub timestamp: DateTime<Utc>,
    /// Service handling the request
    pub service: String,
    /// Index of the matched endpoint within the service
    pub endpoint: Option<usize>,
    /// HTTP method of the request
    pub method: String,
    /// Request path
    pub path: String,
    /// Response status code
    pub status: u16,
    /// Optional payload (e.g. JSON for telemetry or request body)
    pub payload: Option<String>,
}

impl RequestLogEntry {
    /// Create a new log entry with the current timestamp
    pub fn new(
        service: String,
        endpoint: Option<usize>,
        method: String,
        path: String,
        status: u16,
        payload: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            service,
            endpoint,
            method,
            path,
            status,
            payload,
        }
    }
}

/// Request log backed by persistent storage
#[derive(Clone)]
pub struct RequestLog {
    storage: Arc<dyn Storage>,
}

impl std::fmt::Debug for RequestLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestLog").finish()
    }
}

impl RequestLog {
    /// Create a new request log using the provided storage backend
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Append a new entry to storage
    pub fn add(&self, entry: RequestLogEntry) {
        let _ = self.storage.append_log(&entry);
    }

    /// Retrieve the most recent `limit` entries
    pub fn recent(&self, limit: usize) -> Vec<RequestLogEntry> {
        self.storage
            .query_logs(None, None, None, None, limit)
            .unwrap_or_default()
    }

    /// Query log entries using optional filters
    pub fn query(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> Vec<RequestLogEntry> {
        self.storage
            .query_logs(service, route, method, status, limit)
            .unwrap_or_default()
    }
}
