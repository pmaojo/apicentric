//! Ports and supporting types for contract testing.

use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::contract_testing::*;

/// Supported output formats for contract reports.
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Json,
    JUnit,
    Html,
    Badge,
}

/// Port for persisting and retrieving contracts.
#[async_trait]
pub trait ContractRepository {
    async fn save(&self, contract: &Contract) -> Result<(), RepositoryError>;
    async fn get(&self, id: &ContractId) -> Result<Option<Contract>, RepositoryError>;
    async fn list(&self) -> Result<Vec<Contract>, RepositoryError>;
    async fn delete(&self, id: &ContractId) -> Result<(), RepositoryError>;
    async fn find_by_service(&self, service_name: &str) -> Result<Vec<Contract>, RepositoryError>;
}

/// Port for storing validation results and generating reports.
#[async_trait]
pub trait ContractReportSink {
    async fn write_report(
        &self,
        format: ReportFormat,
        result: &ContractValidationResult,
    ) -> Result<Option<String>, ReportError>;
}

/// Port for making HTTP requests to real APIs.
#[async_trait]
pub trait ContractHttpClient {
    async fn execute_request(
        &self,
        base_url: &ApiUrl,
        config: &RealApiConfig,
        scenario: &ValidationScenario,
    ) -> Result<ApiResponse, HttpClientError>;

    async fn health_check(&self, base_url: &ApiUrl) -> Result<bool, HttpClientError>;
}

/// Port for managing the simulator.
#[async_trait]
pub trait ContractMockApiRunner {
    async fn start(&self, service_spec_path: &str) -> Result<MockApiHandle, MockApiError>;
    async fn stop(&self, handle: MockApiHandle) -> Result<(), MockApiError>;
    async fn execute_request(
        &self,
        handle: &MockApiHandle,
        scenario: &ValidationScenario,
    ) -> Result<ApiResponse, MockApiError>;
    async fn render_response(
        &self,
        scenario: &ValidationScenario,
        fixtures: &serde_json::Value,
    ) -> Result<ApiResponse, MockApiError>;
}

/// Port for loading and parsing service definitions (YAML files).
#[async_trait]
pub trait ServiceSpecLoader {
    async fn load(&self, path: &str) -> Result<ServiceSpec, SpecLoaderError>;
    async fn validate(&self, spec: &ServiceSpec) -> Result<(), SpecLoaderError>;
    fn extract_scenarios(
        &self,
        spec: &ServiceSpec,
    ) -> Result<Vec<ValidationScenario>, SpecLoaderError>;
}

/// Port for rendering Handlebars templates in mock responses.
pub trait TemplateRenderer {
    fn render(
        &self,
        template: &str,
        fixtures: &serde_json::Value,
        helpers: &HashMap<String, String>,
    ) -> Result<String, TemplateError>;
}

/// Port for emitting contract testing metrics.
pub trait ContractMetricsCollector {
    fn record_validation_started(&self, contract_id: &ContractId);
    fn record_validation_completed(
        &self,
        contract_id: &ContractId,
        duration_ms: u64,
        compliance_score: f64,
    );
    fn record_compliance_issue(
        &self,
        contract_id: &ContractId,
        issue_type: &ComplianceIssueType,
        severity: &ComplianceSeverity,
    );
    fn record_api_response_time(&self, endpoint: &str, response_time_ms: u64, status: u16);
}

/// Port for distributed tracing.
pub trait ContractTracingCollector {
    fn start_span(&self, operation: &str, contract_id: &ContractId) -> SpanHandle;
    fn add_span_attribute(&self, span: &SpanHandle, key: &str, value: &str);
    fn finish_span(&self, span: SpanHandle);
}

/// Port for publishing domain events.
#[async_trait]
pub trait ContractEventPublisher {
    async fn publish(&self, event: ContractEvent) -> Result<(), EventError>;
}

/// Port for sending notifications about contract validation results.
#[async_trait]
pub trait ContractNotificationSender {
    async fn send_slack_notification(
        &self,
        webhook_url: &str,
        result: &ContractValidationResult,
    ) -> Result<(), NotificationError>;

    async fn send_email_notification(
        &self,
        recipients: &[String],
        result: &ContractValidationResult,
    ) -> Result<(), NotificationError>;
}

/// Port for generating unique identifiers.
pub trait ContractIdGenerator {
    fn generate_contract_id(&self) -> ContractId;
    fn generate_scenario_id(&self) -> String;
    fn generate_validation_id(&self) -> String;
}

/// Port for loading configuration from environment/files.
pub trait ContractConfigurationProvider {
    fn get_real_api_config(&self, environment: &str) -> Result<RealApiConfig, ConfigError>;
    fn get_compatibility_policy(
        &self,
        policy_name: &str,
    ) -> Result<CompatibilityPolicy, ConfigError>;
    fn get_notification_config(&self) -> Result<NotificationConfig, ConfigError>;
    fn get_reporting_config(&self) -> Result<ReportingConfig, ConfigError>;
}

// === CONTRACT TESTING DATA STRUCTURES ===

#[derive(Debug, Clone)]
pub struct MockApiHandle {
    pub port: u16,
    pub base_url: String,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ServiceSpec {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub fixtures: serde_json::Value,
    pub endpoints: Vec<EndpointSpec>,
}

#[derive(Debug, Clone)]
pub struct EndpointSpec {
    pub path: String,
    pub method: HttpMethod,
    pub conditions: Vec<String>,
    pub response: ResponseSpec,
}

#[derive(Debug, Clone)]
pub struct ResponseSpec {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body_template: String,
}

#[derive(Debug, Clone)]
pub struct SpanHandle {
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub slack_webhook_url: Option<String>,
    pub email_smtp_host: Option<String>,
    pub email_smtp_port: Option<u16>,
    pub email_from: Option<String>,
    pub email_recipients: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReportingConfig {
    pub output_dir: String,
    pub generate_html: bool,
    pub generate_badges: bool,
    pub retention_days: u32,
    pub template_path: Option<String>,
}

// === CONTRACT TESTING ERROR TYPES ===

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Contract not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Failed to write report: {0}")]
    WriteError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Format error: {0}")]
    FormatError(String),
}

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

#[derive(Debug, Error)]
pub enum MockApiError {
    #[error("Failed to start mock API: {0}")]
    StartupError(String),

    #[error("Mock API not responding: {0}")]
    NotResponding(String),

    #[error("Template rendering error: {0}")]
    TemplateError(String),

    #[error("Invalid service specification: {0}")]
    InvalidSpec(String),

    #[error("Port already in use: {0}")]
    PortInUse(u16),
}

#[derive(Debug, Error)]
pub enum SpecLoaderError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template compilation error: {0}")]
    CompilationError(String),

    #[error("Template rendering error: {0}")]
    RenderingError(String),

    #[error("Missing helper: {0}")]
    MissingHelper(String),

    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Failed to publish event: {0}")]
    PublishError(String),

    #[error("Failed to handle event: {0}")]
    HandleError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Failed to send notification: {0}")]
    SendError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("File read error: {0}")]
    FileReadError(String),
}
<<<<<<< HEAD
=======

>>>>>>> origin/main
