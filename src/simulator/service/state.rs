use crate::errors::{ApicentricError, ApicentricResult};
use crate::simulator::config::ScenarioStrategy;
use crate::simulator::log::{RequestLog, RequestLogEntry};
use crate::storage::Storage;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock as StdRwLock};

/// Shared in-memory data bucket for stateful routes
#[derive(Debug, Clone)]
pub struct DataBucket {
    data: Arc<StdRwLock<HashMap<String, Value>>>,
}

impl DataBucket {
    pub fn new(initial: Option<HashMap<String, Value>>) -> Self {
        Self {
            data: Arc::new(StdRwLock::new(initial.unwrap_or_default())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.data.read().ok().and_then(|map| map.get(key).cloned())
    }

    pub fn set(&self, key: String, value: Value) {
        if let Ok(mut map) = self.data.write() {
            map.insert(key, value);
        }
    }

    pub fn remove(&self, key: &str) -> Option<Value> {
        self.data.write().ok().and_then(|mut map| map.remove(key))
    }

    pub fn all(&self) -> HashMap<String, Value> {
        self.data.read().map(|m| m.clone()).unwrap_or_default()
    }
}

/// Service state for managing fixtures and runtime data
#[derive(Debug, Clone)]
pub struct ServiceState {
    pub(crate) fixtures: HashMap<String, Value>,
    pub(crate) runtime_data: HashMap<String, Value>,
    pub(crate) initial_fixtures: HashMap<String, Value>, // Backup of original fixtures for reset
    request_log: RequestLog,
    bucket: DataBucket,
    response_counters: HashMap<usize, usize>,
    log_sender: Option<tokio::sync::broadcast::Sender<RequestLogEntry>>,
}

impl ServiceState {
    pub fn new(
        fixtures: Option<HashMap<String, Value>>,
        bucket: Option<HashMap<String, Value>>,
        storage: Arc<dyn Storage>,
        log_sender: Option<tokio::sync::broadcast::Sender<RequestLogEntry>>,
    ) -> Self {
        let fixtures = fixtures.unwrap_or_default();
        Self {
            initial_fixtures: fixtures.clone(),
            fixtures,
            runtime_data: HashMap::new(),
            request_log: RequestLog::new(storage),
            bucket: DataBucket::new(bucket),
            response_counters: HashMap::new(),
            log_sender,
        }
    }

    pub fn next_response_index(
        &mut self,
        endpoint_index: usize,
        total: usize,
        strategy: ScenarioStrategy,
    ) -> usize {
        match strategy {
            ScenarioStrategy::Sequential => {
                let counter = self.response_counters.entry(endpoint_index).or_insert(0);
                let idx = *counter;
                *counter = (*counter + 1) % total;
                idx
            }
            ScenarioStrategy::Random => {
                use rand::Rng;
                rand::thread_rng().gen_range(0..total)
            }
        }
    }

    pub fn bucket(&self) -> DataBucket {
        self.bucket.clone()
    }

    /// Get a fixture by key
    pub fn get_fixture(&self, key: &str) -> Option<&Value> {
        self.fixtures.get(key)
    }

    /// Set a fixture value
    pub fn set_fixture(&mut self, key: String, value: Value) {
        self.fixtures.insert(key, value);
    }

    /// Remove a fixture
    pub fn remove_fixture(&mut self, key: &str) -> Option<Value> {
        self.fixtures.remove(key)
    }

    /// Add an item to a fixture array
    pub fn add_to_fixture_array(&mut self, fixture_key: &str, item: Value) -> ApicentricResult<()> {
        match self.fixtures.get_mut(fixture_key) {
            Some(Value::Array(arr)) => {
                arr.push(item);
                Ok(())
            }
            Some(_) => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' is not an array", fixture_key),
                Some("Use add_to_fixture_array only with array fixtures"),
            )),
            None => {
                // Create new array with the item
                self.fixtures
                    .insert(fixture_key.to_string(), Value::Array(vec![item]));
                Ok(())
            }
        }
    }

    /// Remove an item from a fixture array by index
    pub fn remove_from_fixture_array(
        &mut self,
        fixture_key: &str,
        index: usize,
    ) -> ApicentricResult<Value> {
        match self.fixtures.get_mut(fixture_key) {
            Some(Value::Array(arr)) => {
                if index < arr.len() {
                    Ok(arr.remove(index))
                } else {
                    Err(ApicentricError::runtime_error(
                        format!("Index {} out of bounds for fixture array '{}'", index, fixture_key),
                        Some("Check array length before removing items"),
                    ))
                }
            }
            Some(_) => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' is not an array", fixture_key),
                Some("Use remove_from_fixture_array only with array fixtures"),
            )),
            None => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' not found", fixture_key),
                Some("Check that the fixture exists before removing items"),
            )),
        }
    }

    /// Update an item in a fixture array by index
    pub fn update_fixture_array_item(
        &mut self,
        fixture_key: &str,
        index: usize,
        item: Value,
    ) -> ApicentricResult<()> {
        match self.fixtures.get_mut(fixture_key) {
            Some(Value::Array(arr)) => {
                if index < arr.len() {
                    arr[index] = item;
                    Ok(())
                } else {
                    Err(ApicentricError::runtime_error(
                        format!("Index {} out of bounds for fixture array '{}'", index, fixture_key),
                        Some("Check array length before updating items"),
                    ))
                }
            }
            Some(_) => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' is not an array", fixture_key),
                Some("Use update_fixture_array_item only with array fixtures"),
            )),
            None => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' not found", fixture_key),
                Some("Check that the fixture exists before updating items"),
            )),
        }
    }

    /// Find and update an item in a fixture array by a field value
    pub fn update_fixture_array_item_by_field(
        &mut self,
        fixture_key: &str,
        field: &str,
        field_value: &Value,
        new_item: Value,
    ) -> ApicentricResult<bool> {
        match self.fixtures.get_mut(fixture_key) {
            Some(Value::Array(arr)) => {
                for item in arr.iter_mut() {
                    if let Some(obj) = item.as_object() {
                        if let Some(value) = obj.get(field) {
                            if value == field_value {
                                *item = new_item;
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false) // Item not found
            }
            Some(_) => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' is not an array", fixture_key),
                Some("Use update_fixture_array_item_by_field only with array fixtures"),
            )),
            None => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' not found", fixture_key),
                Some("Check that the fixture exists before updating items"),
            )),
        }
    }

    /// Find and remove an item from a fixture array by a field value
    pub fn remove_fixture_array_item_by_field(
        &mut self,
        fixture_key: &str,
        field: &str,
        field_value: &Value,
    ) -> ApicentricResult<Option<Value>> {
        match self.fixtures.get_mut(fixture_key) {
            Some(Value::Array(arr)) => {
                for (index, item) in arr.iter().enumerate() {
                    if let Some(obj) = item.as_object() {
                        if let Some(value) = obj.get(field) {
                            if value == field_value {
                                return Ok(Some(arr.remove(index)));
                            }
                        }
                    }
                }
                Ok(None) // Item not found
            }
            Some(_) => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' is not an array", fixture_key),
                Some("Use remove_fixture_array_item_by_field only with array fixtures"),
            )),
            None => Err(ApicentricError::runtime_error(
                format!("Fixture '{}' not found", fixture_key),
                Some("Check that the fixture exists before removing items"),
            )),
        }
    }

    /// Reset fixtures to their initial state
    pub fn reset_fixtures(&mut self) {
        self.fixtures = self.initial_fixtures.clone();
    }

    /// Get runtime data by key
    pub fn get_runtime_data(&self, key: &str) -> Option<&Value> {
        self.runtime_data.get(key)
    }

    /// Set runtime data
    pub fn set_runtime_data(&mut self, key: String, value: Value) {
        self.runtime_data.insert(key, value);
    }

    /// Remove runtime data
    pub fn remove_runtime_data(&mut self, key: &str) -> Option<Value> {
        self.runtime_data.remove(key)
    }

    /// Clear all runtime data
    pub fn clear_runtime_data(&mut self) {
        self.runtime_data.clear();
    }

    /// Get all fixtures
    pub fn all_fixtures(&self) -> &HashMap<String, Value> {
        &self.fixtures
    }

    /// Get all runtime data
    pub fn all_runtime_data(&self) -> &HashMap<String, Value> {
        &self.runtime_data
    }

    /// Get fixture count
    pub fn fixture_count(&self) -> usize {
        self.fixtures.len()
    }

    /// Get runtime data count
    pub fn runtime_data_count(&self) -> usize {
        self.runtime_data.len()
    }

    /// Check if a fixture exists
    pub fn has_fixture(&self, key: &str) -> bool {
        self.fixtures.contains_key(key)
    }

    /// Check if runtime data exists
    pub fn has_runtime_data(&self, key: &str) -> bool {
        self.runtime_data.contains_key(key)
    }

    /// Append a request log entry
    pub fn add_log_entry(&mut self, entry: RequestLogEntry) {
        self.request_log.add(entry.clone());
        if let Some(sender) = &self.log_sender {
            let _ = sender.send(entry);
        }
    }

    /// Retrieve recent request log entries
    pub fn get_logs(&self, limit: usize) -> Vec<RequestLogEntry> {
        self.request_log.recent(limit)
    }

    /// Query log entries with optional filters
    pub fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> Vec<RequestLogEntry> {
        self.request_log
            .query(service, route, method, status, limit)
    }
}
