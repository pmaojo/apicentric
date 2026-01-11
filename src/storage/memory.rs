use crate::errors::ApicentricResult;
use crate::simulator::config::ServiceDefinition;
use crate::simulator::log::RequestLogEntry;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct InMemoryStorage {
    services: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
    logs: Arc<RwLock<Vec<RequestLogEntry>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Storage for InMemoryStorage {
    fn save_service(&self, service: &ServiceDefinition) -> ApicentricResult<()> {
        let mut services = self.services.write().unwrap();
        services.insert(service.name.clone(), service.clone());
        Ok(())
    }

    fn load_service(&self, name: &str) -> ApicentricResult<Option<ServiceDefinition>> {
        let services = self.services.read().unwrap();
        Ok(services.get(name).cloned())
    }

    fn append_log(&self, entry: &RequestLogEntry) -> ApicentricResult<()> {
        let mut logs = self.logs.write().unwrap();
        logs.push(entry.clone());
        Ok(())
    }

    fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> ApicentricResult<Vec<RequestLogEntry>> {
        let logs = self.logs.read().unwrap();
        let mut result = logs.iter().rev()
            .filter(|log| {
                if let Some(s) = service {
                    if &log.service != s { return false; }
                }
                if let Some(r) = route {
                    if &log.path != r { return false; }
                }
                if let Some(m) = method {
                    if &log.method != m { return false; }
                }
                if let Some(s) = status {
                    if log.status != s { return false; }
                }
                true
            })
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        Ok(result)
    }

    fn clear_logs(&self) -> ApicentricResult<()> {
        let mut logs = self.logs.write().unwrap();
        logs.clear();
        Ok(())
    }
}
