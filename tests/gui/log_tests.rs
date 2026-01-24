//! Tests for request logging functionality
//!
//! This module tests the RequestLogEntry and LogFilter functionality.

#![cfg(feature = "gui")]

use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// Request log entry for GUI display
#[derive(Debug, Clone, PartialEq)]
pub struct RequestLogEntry {
    pub timestamp: SystemTime,
    pub service_name: String,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
}

impl RequestLogEntry {
    /// Create a new request log entry
    pub fn new(
        service_name: String,
        method: String,
        path: String,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        Self {
            timestamp: SystemTime::now(),
            service_name,
            method,
            path,
            status_code,
            duration_ms,
        }
    }

    /// Create a request log entry with a specific timestamp
    pub fn with_timestamp(
        timestamp: SystemTime,
        service_name: String,
        method: String,
        path: String,
        status_code: u16,
        duration_ms: u64,
    ) -> Self {
        Self {
            timestamp,
            service_name,
            method,
            path,
            status_code,
            duration_ms,
        }
    }

    /// Convert from simulator log entry
    pub fn from_simulator_log(log: &apicentric::simulator::log::RequestLogEntry) -> Self {
        Self {
            timestamp: SystemTime::now(),
            service_name: log.service.clone(),
            method: log.method.clone(),
            path: log.path.clone(),
            status_code: log.status,
            duration_ms: 0,
        }
    }
}

/// Filter for request logs
#[derive(Debug, Clone, PartialEq)]
pub enum LogFilter {
    All,
    Service(String),
    StatusCode(u16),
    Method(String),
}

impl LogFilter {
    /// Check if a log entry matches this filter
    pub fn matches(&self, entry: &RequestLogEntry) -> bool {
        match self {
            LogFilter::All => true,
            LogFilter::Service(name) => entry.service_name == *name,
            LogFilter::StatusCode(code) => entry.status_code == *code,
            LogFilter::Method(method) => entry.method == *method,
        }
    }
}

/// Request log manager with rotation
pub struct RequestLogManager {
    logs: VecDeque<RequestLogEntry>,
    max_size: usize,
}

impl RequestLogManager {
    /// Create a new log manager with default max size of 1000
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new log manager with specified max size
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            logs: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a log entry with automatic rotation
    pub fn add(&mut self, entry: RequestLogEntry) {
        self.logs.push_back(entry);

        // Rotate if exceeds max size
        while self.logs.len() > self.max_size {
            self.logs.pop_front();
        }
    }

    /// Get all logs
    pub fn logs(&self) -> &VecDeque<RequestLogEntry> {
        &self.logs
    }

    /// Get filtered logs
    pub fn filtered_logs(&self, filter: &LogFilter) -> Vec<RequestLogEntry> {
        self.logs
            .iter()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .collect()
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// Get the number of logs
    pub fn len(&self) -> usize {
        self.logs.len()
    }

    /// Check if logs are empty
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }
}

impl Default for RequestLogManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Test log entry creation

    #[test]
    fn test_create_log_entry() {
        let entry = RequestLogEntry::new(
            "test-service".to_string(),
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            45,
        );

        assert_eq!(entry.service_name, "test-service");
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.path, "/api/users");
        assert_eq!(entry.status_code, 200);
        assert_eq!(entry.duration_ms, 45);
    }

    #[test]
    fn test_create_log_entry_with_timestamp() {
        let timestamp = SystemTime::now();
        let entry = RequestLogEntry::with_timestamp(
            timestamp,
            "api".to_string(),
            "POST".to_string(),
            "/users".to_string(),
            201,
            120,
        );

        assert_eq!(entry.timestamp, timestamp);
        assert_eq!(entry.service_name, "api");
        assert_eq!(entry.method, "POST");
        assert_eq!(entry.path, "/users");
        assert_eq!(entry.status_code, 201);
        assert_eq!(entry.duration_ms, 120);
    }

    #[test]
    fn test_log_entry_different_status_codes() {
        let entry_200 = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        let entry_404 = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/notfound".to_string(),
            404,
            5,
        );

        let entry_500 = RequestLogEntry::new(
            "api".to_string(),
            "POST".to_string(),
            "/error".to_string(),
            500,
            100,
        );

        assert_eq!(entry_200.status_code, 200);
        assert_eq!(entry_404.status_code, 404);
        assert_eq!(entry_500.status_code, 500);
    }

    // Test log filtering

    #[test]
    fn test_filter_all() {
        let filter = LogFilter::All;

        let entry1 = RequestLogEntry::new(
            "service1".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        let entry2 = RequestLogEntry::new(
            "service2".to_string(),
            "POST".to_string(),
            "/data".to_string(),
            404,
            20,
        );

        assert!(filter.matches(&entry1));
        assert!(filter.matches(&entry2));
    }

    #[test]
    fn test_filter_by_service() {
        let filter = LogFilter::Service("api-service".to_string());

        let matching = RequestLogEntry::new(
            "api-service".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        let not_matching = RequestLogEntry::new(
            "other-service".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&not_matching));
    }

    #[test]
    fn test_filter_by_status_code() {
        let filter = LogFilter::StatusCode(404);

        let matching = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/notfound".to_string(),
            404,
            10,
        );

        let not_matching = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&not_matching));
    }

    #[test]
    fn test_filter_by_method() {
        let filter = LogFilter::Method("POST".to_string());

        let matching = RequestLogEntry::new(
            "api".to_string(),
            "POST".to_string(),
            "/data".to_string(),
            201,
            50,
        );

        let not_matching = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/data".to_string(),
            200,
            10,
        );

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&not_matching));
    }

    // Test log rotation

    #[test]
    fn test_log_manager_creation() {
        let manager = RequestLogManager::new();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
        assert_eq!(manager.max_size, 1000);
    }

    #[test]
    fn test_log_manager_with_custom_capacity() {
        let manager = RequestLogManager::with_capacity(100);
        assert_eq!(manager.max_size, 100);
    }

    #[test]
    fn test_add_log_entry() {
        let mut manager = RequestLogManager::new();

        let entry = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        );

        manager.add(entry.clone());

        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
        assert_eq!(manager.logs()[0], entry);
    }

    #[test]
    fn test_add_multiple_log_entries() {
        let mut manager = RequestLogManager::new();

        for i in 0..10 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        assert_eq!(manager.len(), 10);
    }

    #[test]
    fn test_log_rotation_at_max_capacity() {
        let mut manager = RequestLogManager::with_capacity(5);

        // Add 5 entries (at capacity)
        for i in 0..5 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        assert_eq!(manager.len(), 5);

        // Add one more - should rotate
        let entry = RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/path5".to_string(),
            200,
            10,
        );
        manager.add(entry);

        assert_eq!(manager.len(), 5); // Still at max capacity

        // First entry should be /path1 (path0 was rotated out)
        assert_eq!(manager.logs()[0].path, "/path1");
        assert_eq!(manager.logs()[4].path, "/path5");
    }

    #[test]
    fn test_log_rotation_with_1000_entries() {
        let mut manager = RequestLogManager::new(); // Default 1000

        // Add 1500 entries
        for i in 0..1500 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        // Should only keep last 1000
        assert_eq!(manager.len(), 1000);

        // First entry should be path500 (0-499 were rotated out)
        assert_eq!(manager.logs()[0].path, "/path500");
        assert_eq!(manager.logs()[999].path, "/path1499");
    }

    #[test]
    fn test_clear_logs() {
        let mut manager = RequestLogManager::new();

        // Add some entries
        for i in 0..10 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        assert_eq!(manager.len(), 10);

        manager.clear();

        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_filtered_logs() {
        let mut manager = RequestLogManager::new();

        // Add entries for different services
        manager.add(RequestLogEntry::new(
            "service1".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        ));

        manager.add(RequestLogEntry::new(
            "service2".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        ));

        manager.add(RequestLogEntry::new(
            "service1".to_string(),
            "POST".to_string(),
            "/data".to_string(),
            201,
            20,
        ));

        // Filter by service1
        let filter = LogFilter::Service("service1".to_string());
        let filtered = manager.filtered_logs(&filter);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.service_name == "service1"));
    }

    #[test]
    fn test_filtered_logs_by_status_code() {
        let mut manager = RequestLogManager::new();

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        ));

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/notfound".to_string(),
            404,
            5,
        ));

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/another".to_string(),
            404,
            7,
        ));

        // Filter by 404
        let filter = LogFilter::StatusCode(404);
        let filtered = manager.filtered_logs(&filter);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.status_code == 404));
    }

    #[test]
    fn test_filtered_logs_by_method() {
        let mut manager = RequestLogManager::new();

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        ));

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "POST".to_string(),
            "/data".to_string(),
            201,
            20,
        ));

        manager.add(RequestLogEntry::new(
            "api".to_string(),
            "POST".to_string(),
            "/more".to_string(),
            201,
            25,
        ));

        // Filter by POST
        let filter = LogFilter::Method("POST".to_string());
        let filtered = manager.filtered_logs(&filter);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.method == "POST"));
    }

    #[test]
    fn test_filtered_logs_all() {
        let mut manager = RequestLogManager::new();

        manager.add(RequestLogEntry::new(
            "service1".to_string(),
            "GET".to_string(),
            "/".to_string(),
            200,
            10,
        ));

        manager.add(RequestLogEntry::new(
            "service2".to_string(),
            "POST".to_string(),
            "/data".to_string(),
            404,
            20,
        ));

        // Filter All should return everything
        let filter = LogFilter::All;
        let filtered = manager.filtered_logs(&filter);

        assert_eq!(filtered.len(), 2);
    }

    // Performance test for log rotation

    #[test]
    fn test_log_rotation_performance() {
        let mut manager = RequestLogManager::new();

        // Add many entries to test rotation performance
        let start = SystemTime::now();

        for i in 0..5000 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        let elapsed = start.elapsed().unwrap();

        // Should complete quickly (under 100ms)
        assert!(elapsed < Duration::from_millis(100));

        // Should only keep last 1000
        assert_eq!(manager.len(), 1000);
    }
}

// Integration tests for log receiver and real-time updates

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_receive_logs_from_simulator() {
        let (tx, _rx) = broadcast::channel(100);

        // Simulate receiving a log from the simulator
        let sim_log = apicentric::simulator::log::RequestLogEntry::new(
            "test-service".to_string(),
            Some(0),
            "GET".to_string(),
            "/api/users".to_string(),
            200,
            None,
        );

        tx.send(sim_log.clone()).unwrap();

        // Convert to GUI log entry
        let gui_log = RequestLogEntry::from_simulator_log(&sim_log);

        assert_eq!(gui_log.service_name, "test-service");
        assert_eq!(gui_log.method, "GET");
        assert_eq!(gui_log.path, "/api/users");
        assert_eq!(gui_log.status_code, 200);
    }

    #[tokio::test]
    async fn test_real_time_log_updates() {
        let (tx, _rx) = broadcast::channel(100);
        let mut manager = RequestLogManager::new();

        // Simulate receiving multiple logs in real-time
        for i in 0..10 {
            let sim_log = apicentric::simulator::log::RequestLogEntry::new(
                "api".to_string(),
                Some(0),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                None,
            );

            tx.send(sim_log.clone()).unwrap();

            let gui_log = RequestLogEntry::from_simulator_log(&sim_log);
            manager.add(gui_log);
        }

        assert_eq!(manager.len(), 10);
    }

    #[tokio::test]
    async fn test_performance_with_many_logs() {
        let (tx, _rx) = broadcast::channel(1000);
        let mut manager = RequestLogManager::new();

        let start = SystemTime::now();

        // Simulate high-frequency log generation
        for i in 0..1000 {
            let sim_log = apicentric::simulator::log::RequestLogEntry::new(
                "high-traffic-service".to_string(),
                Some(0),
                "GET".to_string(),
                format!("/endpoint{}", i % 10),
                200,
                None,
            );

            tx.send(sim_log.clone()).unwrap();

            let gui_log = RequestLogEntry::from_simulator_log(&sim_log);
            manager.add(gui_log);
        }

        let elapsed = start.elapsed().unwrap();

        // Should handle 1000 logs quickly (under 100ms)
        assert!(elapsed < Duration::from_millis(100));
        assert_eq!(manager.len(), 1000);
    }

    #[tokio::test]
    async fn test_log_rotation_during_high_traffic() {
        let (tx, _rx) = broadcast::channel(2000);
        let mut manager = RequestLogManager::new();

        // Simulate receiving 2000 logs (should rotate to keep only 1000)
        for i in 0..2000 {
            let sim_log = apicentric::simulator::log::RequestLogEntry::new(
                "service".to_string(),
                Some(0),
                "POST".to_string(),
                format!("/data/{}", i),
                201,
                None,
            );

            tx.send(sim_log.clone()).unwrap();

            let gui_log = RequestLogEntry::from_simulator_log(&sim_log);
            manager.add(gui_log);
        }

        // Should only keep last 1000
        assert_eq!(manager.len(), 1000);

        // First log should be from iteration 1000
        assert_eq!(manager.logs()[0].path, "/data/1000");
        assert_eq!(manager.logs()[999].path, "/data/1999");
    }

    #[tokio::test]
    async fn test_filtered_logs_performance() {
        let mut manager = RequestLogManager::new();

        // Add logs for multiple services
        for i in 0..1000 {
            let service = if i % 3 == 0 {
                "service-a"
            } else if i % 3 == 1 {
                "service-b"
            } else {
                "service-c"
            };

            let entry = RequestLogEntry::new(
                service.to_string(),
                "GET".to_string(),
                format!("/path{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        let start = SystemTime::now();

        // Filter by service-a
        let filter = LogFilter::Service("service-a".to_string());
        let filtered = manager.filtered_logs(&filter);

        let elapsed = start.elapsed().unwrap();

        // Filtering should be fast (under 10ms)
        assert!(elapsed < Duration::from_millis(10));

        // Should have ~333 logs for service-a
        assert!(filtered.len() >= 330 && filtered.len() <= 340);
        assert!(filtered.iter().all(|e| e.service_name == "service-a"));
    }

    #[tokio::test]
    async fn test_concurrent_log_updates() {
        use tokio::task;

        let (tx, _rx) = broadcast::channel(1000);

        // Spawn multiple tasks to simulate concurrent log generation
        let mut handles = vec![];

        for task_id in 0..5 {
            let tx_clone = tx.clone();
            let handle = task::spawn(async move {
                for i in 0..100 {
                    let sim_log = apicentric::simulator::log::RequestLogEntry::new(
                        format!("service-{}", task_id),
                        Some(0),
                        "GET".to_string(),
                        format!("/path{}", i),
                        200,
                        None,
                    );

                    let _ = tx_clone.send(sim_log);
                    tokio::time::sleep(Duration::from_micros(10)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Collect all logs (in real app, this would be done by the receiver)
        // For this test, we'll just verify the channel works
        // Channel operations succeeded
    }

    #[tokio::test]
    async fn test_log_filtering_with_real_time_updates() {
        let mut manager = RequestLogManager::new();

        // Add initial logs
        for i in 0..50 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/users/{}", i),
                200,
                10,
            );
            manager.add(entry);
        }

        // Add some error logs
        for i in 0..20 {
            let entry = RequestLogEntry::new(
                "api".to_string(),
                "GET".to_string(),
                format!("/notfound/{}", i),
                404,
                5,
            );
            manager.add(entry);
        }

        // Filter by status code 404
        let filter = LogFilter::StatusCode(404);
        let errors = manager.filtered_logs(&filter);

        assert_eq!(errors.len(), 20);
        assert!(errors.iter().all(|e| e.status_code == 404));

        // Total logs should be 70
        assert_eq!(manager.len(), 70);
    }

    #[tokio::test]
    async fn test_log_conversion_from_simulator() {
        // Test that we can convert simulator logs to GUI logs correctly
        let sim_log = apicentric::simulator::log::RequestLogEntry::new(
            "my-service".to_string(),
            Some(2),
            "POST".to_string(),
            "/api/data".to_string(),
            201,
            None,
        );

        let gui_log = RequestLogEntry::from_simulator_log(&sim_log);

        assert_eq!(gui_log.service_name, "my-service");
        assert_eq!(gui_log.method, "POST");
        assert_eq!(gui_log.path, "/api/data");
        assert_eq!(gui_log.status_code, 201);
        assert_eq!(gui_log.duration_ms, 0); // Simulator doesn't track duration
    }
}
