use crate::errors::ApicentricResult;
use crate::simulator::config::ServiceDefinition;
use crate::simulator::log::RequestLogEntry;

pub trait Storage: Send + Sync {
    fn save_service(&self, service: &ServiceDefinition) -> ApicentricResult<()>;
    fn load_service(&self, name: &str) -> ApicentricResult<Option<ServiceDefinition>>;
    fn append_log(&self, entry: &RequestLogEntry) -> ApicentricResult<()>;
    fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> ApicentricResult<Vec<RequestLogEntry>>;
    fn clear_logs(&self) -> ApicentricResult<()>;
}

#[cfg(feature = "database")]
pub mod sqlite;
