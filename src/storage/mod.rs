use crate::errors::ApicentricResult;
use crate::simulator::config::ServiceDefinition;
use crate::simulator::log::RequestLogEntry;

#[derive(Debug, Default)]
pub struct LogStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
}

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
    fn get_log_stats(&self) -> ApicentricResult<LogStats>;
    fn clear_logs(&self) -> ApicentricResult<()>;
}

#[cfg(feature = "database")]
pub mod sqlite;
