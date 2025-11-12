use crate::simulator::service::state::ServiceState;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// High level wrapper around [`ServiceState`] providing async accessors.
///
/// This allows callers to interact with service state without needing to
/// manage locking directly, keeping concerns separated from the
/// `ServiceInstance` orchestration logic.
#[derive(Clone)]
pub struct StateService {
    inner: Arc<RwLock<ServiceState>>,
}

impl StateService {
    pub fn new(state: ServiceState) -> Self {
        Self { inner: Arc::new(RwLock::new(state)) }
    }

    pub async fn update(&self, key: &str, value: Value) {
        let mut state = self.inner.write().await;
        state.set_runtime_data(key.to_string(), value);
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        let state = self.inner.read().await;
        state
            .get_runtime_data(key)
            .cloned()
            .or_else(|| state.get_fixture(key).cloned())
    }

    /// Access to the underlying state for advanced operations.
    pub fn raw(&self) -> Arc<RwLock<ServiceState>> {
        Arc::clone(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ApicentricResult;
    use crate::simulator::config::ServiceDefinition;
    use crate::simulator::log::RequestLogEntry;
    use crate::storage::Storage;

    struct DummyStorage;

    impl Storage for DummyStorage {
        fn save_service(&self, _service: &ServiceDefinition) -> ApicentricResult<()> {
            Ok(())
        }

        fn load_service(&self, _name: &str) -> ApicentricResult<Option<ServiceDefinition>> {
            Ok(None)
        }

        fn append_log(&self, _entry: &RequestLogEntry) -> ApicentricResult<()> {
            Ok(())
        }

        fn query_logs(
            &self,
            _service: Option<&str>,
            _route: Option<&str>,
            _method: Option<&str>,
            _status: Option<u16>,
            _limit: usize,
        ) -> ApicentricResult<Vec<RequestLogEntry>> {
            Ok(vec![])
        }

        fn clear_logs(&self) -> ApicentricResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn state_roundtrip() {
        let service = StateService::new(ServiceState::new(
            None,
            None,
            Arc::new(DummyStorage),
            None,
        ));
        service.update("key", Value::String("value".into())).await;
        assert_eq!(service.get("key").await, Some(Value::String("value".into())));
    }
}
