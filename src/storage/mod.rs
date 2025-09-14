use crate::errors::PulseResult;
use crate::simulator::config::ServiceDefinition;
use crate::simulator::log::RequestLogEntry;

pub trait Storage: Send + Sync {
    fn save_service(&self, service: &ServiceDefinition) -> PulseResult<()>;
    fn load_service(&self, name: &str) -> PulseResult<Option<ServiceDefinition>>;
    fn append_log(&self, entry: &RequestLogEntry) -> PulseResult<()>;
    fn query_logs(
        &self,
        service: Option<&str>,
        route: Option<&str>,
        method: Option<&str>,
        status: Option<u16>,
        limit: usize,
    ) -> PulseResult<Vec<RequestLogEntry>>;
}

pub mod sqlite;
