//! Request logging utilities for the simulator

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

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
}

impl RequestLogEntry {
    /// Create a new log entry with the current timestamp
    pub fn new(
        service: String,
        endpoint: Option<usize>,
        method: String,
        path: String,
        status: u16,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            service,
            endpoint,
            method,
            path,
            status,
        }
    }
}

/// Ring buffer storing recent request logs
#[derive(Debug, Clone)]
pub struct RequestLog {
    capacity: usize,
    entries: VecDeque<RequestLogEntry>,
}

impl RequestLog {
    /// Create a new request log with a fixed capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        }
    }

    /// Append a new entry, removing the oldest if capacity is exceeded
    pub fn add(&mut self, entry: RequestLogEntry) {
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Retrieve the most recent `limit` entries in chronological order
    pub fn recent(&self, limit: usize) -> Vec<RequestLogEntry> {
        let len = self.entries.len();
        self.entries
            .iter()
            .skip(len.saturating_sub(limit))
            .cloned()
            .collect()
    }

    /// Query log entries using optional filters and return most recent `limit` entries
    /// in chronological order.
    pub fn query(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> Vec<RequestLogEntry> {
        let filtered: Vec<_> = self
            .entries
            .iter()
            .filter(|e| match service {
                Some(s) => e.service == s,
                None => true,
            })
            .filter(|e| match route {
                Some(r) => e.path.contains(r),
                None => true,
            })
            .filter(|e| match method {
                Some(m) => e.method.eq_ignore_ascii_case(m),
                None => true,
            })
            .filter(|e| match status {
                Some(s) => e.status == s,
                None => true,
            })
            .cloned()
            .collect();

        let len = filtered.len();
        filtered
            .into_iter()
            .skip(len.saturating_sub(limit))
            .collect()
    }
}
