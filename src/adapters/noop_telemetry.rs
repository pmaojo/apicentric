//! No-op implementations for telemetry traits (metrics, tracing, events).
use crate::{
    domain::{
        contract_testing::{ComplianceIssueType, ComplianceSeverity},
        ports::contract::{
            ContractEventPublisher, ContractMetricsCollector, ContractTracingCollector, EventError,
            SpanHandle,
        },
    },
    ContractEvent, ContractId,
};
use async_trait::async_trait;

/// A `ContractMetricsCollector` that does nothing.
pub struct NoOpMetrics;
impl ContractMetricsCollector for NoOpMetrics {
    fn record_validation_started(&self, _id: &ContractId) {}
    fn record_validation_completed(&self, _id: &ContractId, _d: u64, _s: f64) {}
    fn record_compliance_issue(
        &self,
        _id: &ContractId,
        _t: &ComplianceIssueType,
        _s: &ComplianceSeverity,
    ) {
    }
    fn record_api_response_time(&self, _e: &str, _t: u64, _s: u16) {}
}

/// A `ContractTracingCollector` that does nothing.
pub struct NoOpTracer;
impl ContractTracingCollector for NoOpTracer {
    fn start_span(&self, _op: &str, _id: &ContractId) -> SpanHandle {
        SpanHandle {
            trace_id: String::new(),
            span_id: String::new(),
        }
    }
    fn add_span_attribute(&self, _span: &SpanHandle, _key: &str, _value: &str) {}
    fn finish_span(&self, _span: SpanHandle) {}
}

/// A `ContractEventPublisher` that does nothing.
pub struct NoOpPublisher;
#[async_trait]
impl ContractEventPublisher for NoOpPublisher {
    async fn publish(&self, _event: ContractEvent) -> Result<(), EventError> {
        Ok(())
    }
}
