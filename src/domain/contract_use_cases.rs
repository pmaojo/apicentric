// Contract Testing Use Cases - Application Layer
// Orchestrates domain logic with infrastructure concerns

use super::contract_testing::*;
use super::ports::contract::*;
use std::time::SystemTime;
use tracing::{debug, error, info, warn};

// === PRIMARY USE CASES ===

/// Main use case for validating contracts against real APIs
pub struct ValidateContractUseCase<
    T: ContractRepository,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    S: ServiceSpecLoader,
> {
    contract_repo: T,
    http_client: H,
    mock_api: M,
    spec_loader: S,
    metrics: Box<dyn ContractMetricsCollector>,
    tracer: Box<dyn ContractTracingCollector>,
    event_publisher: Box<dyn ContractEventPublisher>,
}

impl<T, H, M, S> ValidateContractUseCase<T, H, M, S>
where
    T: ContractRepository,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    S: ServiceSpecLoader,
{
    pub fn new(
        contract_repo: T,
        http_client: H,
        mock_api: M,
        spec_loader: S,
        metrics: Box<dyn ContractMetricsCollector>,
        tracer: Box<dyn ContractTracingCollector>,
        event_publisher: Box<dyn ContractEventPublisher>,
    ) -> Self {
        Self {
            contract_repo,
            http_client,
            mock_api,
            spec_loader,
            metrics,
            tracer,
            event_publisher,
        }
    }

    /// Execute contract validation workflow
    pub async fn execute(
        &self,
        contract_id: ContractId,
        real_api_config: RealApiConfig,
        policy: CompatibilityPolicy,
    ) -> Result<ContractValidationResult, ContractUseCaseError> {
        let span = self.tracer.start_span("validate_contract", &contract_id);
        self.tracer
            .add_span_attribute(&span, "contract_id", &contract_id.to_string());

        let start_time = std::time::Instant::now();
        self.metrics.record_validation_started(&contract_id);

        // Load contract from repository
        let contract = self.load_contract(&contract_id).await?;
        info!(
            "Loaded contract: {} for service: {}",
            contract_id, contract.service_name
        );

        // Load specification and scenarios
        let scenarios = self.load_and_validate_spec(&contract).await?;
        info!("Extracted {} scenarios for validation", scenarios.len());

        // Run scenarios
        let validation_result = self
            .run_validation_scenarios(&contract, &real_api_config, &scenarios, &policy)
            .await?;

        // Record metrics
        let duration = start_time.elapsed();
        self.metrics.record_validation_completed(
            &contract_id,
            duration.as_millis() as u64,
            validation_result.compliance_score,
        );

        // Publish domain event
        self.publish_completion_event(&contract_id, &validation_result)
            .await;

        self.tracer.finish_span(span);

        info!(
            "Contract validation completed - Score: {:.2}%, Issues: {}",
            validation_result.compliance_score * 100.0,
            validation_result.issues.len()
        );

        Ok(validation_result)
    }

    async fn load_contract(
        &self,
        contract_id: &ContractId,
    ) -> Result<Contract, ContractUseCaseError> {
        self.contract_repo
            .get(contract_id)
            .await
            .map_err(|e| ContractUseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| ContractUseCaseError::ContractNotFound(contract_id.to_string()))
    }

    async fn load_and_validate_spec(
        &self,
        contract: &Contract,
    ) -> Result<Vec<ValidationScenario>, ContractUseCaseError> {
        let spec = self
            .spec_loader
            .load(&contract.spec_path)
            .await
            .map_err(|e| ContractUseCaseError::SpecLoadError(e.to_string()))?;

        self.spec_loader
            .validate(&spec)
            .await
            .map_err(|e| ContractUseCaseError::SpecValidationError(e.to_string()))?;

        let scenarios = self
            .spec_loader
            .extract_scenarios(&spec)
            .map_err(|e| ContractUseCaseError::SpecLoadError(e.to_string()))?;

        Ok(scenarios)
    }

    async fn run_validation_scenarios(
        &self,
        contract: &Contract,
        real_api_config: &RealApiConfig,
        scenarios: &[ValidationScenario],
        policy: &CompatibilityPolicy,
    ) -> Result<ContractValidationResult, ContractUseCaseError> {
        let mock_handle = self
            .mock_api
            .start(&contract.spec_path)
            .await
            .map_err(|e| ContractUseCaseError::MockApiError(e.to_string()))?;

        let validation_result = match self
            .validate_scenarios(contract, real_api_config, &mock_handle, scenarios, policy)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let _ = self.mock_api.stop(mock_handle).await;
                return Err(e);
            }
        };

        let _ = self.mock_api.stop(mock_handle).await;
        Ok(validation_result)
    }

    async fn publish_completion_event(
        &self,
        contract_id: &ContractId,
        validation_result: &ContractValidationResult,
    ) {
        let event = ContractEvent::ValidationCompleted {
            contract_id: contract_id.clone(),
            validation_result: validation_result.clone(),
            timestamp: SystemTime::now(),
        };

        if let Err(e) = self.event_publisher.publish(event).await {
            warn!("Failed to publish validation event: {}", e);
        }
    }

    async fn validate_scenarios(
        &self,
        contract: &Contract,
        real_api_config: &RealApiConfig,
        mock_handle: &MockApiHandle,
        scenarios: &[ValidationScenario],
        policy: &CompatibilityPolicy,
    ) -> Result<ContractValidationResult, ContractUseCaseError> {
        let mut issues = Vec::new();
        let mut scenario_results = Vec::new();

        for scenario in scenarios {
            debug!("Validating scenario: {} {}", scenario.method, scenario.path);

            match self
                .validate_single_scenario(contract, real_api_config, mock_handle, scenario, policy)
                .await
            {
                Ok(result) => {
                    if let Some(issue) = &result.compliance_issue {
                        issues.push(issue.clone());
                        self.metrics.record_compliance_issue(
                            &contract.id,
                            &issue.issue_type,
                            &issue.severity,
                        );
                    }
                    scenario_results.push(result);
                }
                Err(e) => {
                    error!(
                        "Failed to validate scenario {} {}: {}",
                        scenario.method, scenario.path, e
                    );
                    let issue = ComplianceIssue {
                        issue_type: ComplianceIssueType::ValidationError,
                        severity: ComplianceSeverity::High,
                        description: format!("Validation failed: {}", e),
                        scenario_path: scenario.path.clone(),
                        details: None,
                    };
                    issues.push(issue.clone());

                    scenario_results.push(ScenarioValidationResult {
                        scenario_id: scenario.id.clone(),
                        mock_response: None,
                        real_response: None,
                        compliance_issue: Some(issue),
                        duration_ms: 0,
                    });
                }
            }
        }

        let compliance_score = contract.calculate_compliance_score(&issues);
        let is_compatible = policy.is_compatible(&issues);

        Ok(ContractValidationResult {
            contract_id: contract.id.clone(),
            validation_timestamp: SystemTime::now(),
            compliance_score,
            is_compatible,
            issues,
            scenario_results,
            environment: real_api_config.environment.clone(),
        })
    }

    async fn validate_single_scenario(
        &self,
        contract: &Contract,
        real_api_config: &RealApiConfig,
        mock_handle: &MockApiHandle,
        scenario: &ValidationScenario,
        policy: &CompatibilityPolicy,
    ) -> Result<ScenarioValidationResult, ContractUseCaseError> {
        let start_time = std::time::Instant::now();

        // Execute against mock API
        let mock_response = self
            .mock_api
            .execute_request(mock_handle, scenario)
            .await
            .map_err(|e| ContractUseCaseError::MockApiError(e.to_string()))?;

        // Execute against real API
        let real_response = self
            .http_client
            .execute_request(&real_api_config.base_url, real_api_config, scenario)
            .await
            .map_err(|e| ContractUseCaseError::HttpClientError(e.to_string()))?;

        // Record API response time
        self.metrics.record_api_response_time(
            &scenario.path,
            real_response.duration_ms,
            real_response.status_code,
        );

        // Validate compatibility
        let compliance_issue =
            contract.validate_response_compatibility(&mock_response, &real_response, policy);

        let duration = start_time.elapsed();

        Ok(ScenarioValidationResult {
            scenario_id: scenario.id.clone(),
            mock_response: Some(mock_response),
            real_response: Some(real_response),
            compliance_issue,
            duration_ms: duration.as_millis() as u64,
        })
    }
}

/// Use case for registering and managing contracts
pub struct ManageContractsUseCase<T: ContractRepository, S: ServiceSpecLoader> {
    contract_repo: T,
    spec_loader: S,
    id_generator: Box<dyn ContractIdGenerator>,
    event_publisher: Box<dyn ContractEventPublisher>,
}

impl<T, S> ManageContractsUseCase<T, S>
where
    T: ContractRepository,
    S: ServiceSpecLoader,
{
    pub fn new(
        contract_repo: T,
        spec_loader: S,
        id_generator: Box<dyn ContractIdGenerator>,
        event_publisher: Box<dyn ContractEventPublisher>,
    ) -> Self {
        Self {
            contract_repo,
            spec_loader,
            id_generator,
            event_publisher,
        }
    }

    /// Register a new contract from a service specification file
    pub async fn register_contract(
        &self,
        service_name: String,
        spec_path: String,
        description: Option<String>,
    ) -> Result<Contract, ContractManagementError> {
        info!("Registering contract for service: {}", service_name);

        // Validate spec file exists and is valid
        let spec = self
            .spec_loader
            .load(&spec_path)
            .await
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        self.spec_loader
            .validate(&spec)
            .await
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        // Create contract
        let contract_id = self.id_generator.generate_contract_id();
        let contract = Contract::new(contract_id.clone(), service_name, spec_path, description)
            .map_err(|e| ContractManagementError::InvalidSpec(e.to_string()))?;

        // Save to repository
        self.contract_repo
            .save(&contract)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))?;

        // Publish domain event
        let event = ContractEvent::ContractRegistered {
            contract_id: contract_id.clone(),
            service_name: contract.service_name.clone(),
            timestamp: SystemTime::now(),
        };

        if let Err(e) = self.event_publisher.publish(event).await {
            warn!("Failed to publish contract registration event: {}", e);
        }

        info!("Successfully registered contract: {}", contract_id);
        Ok(contract)
    }

    /// List all registered contracts
    pub async fn list_contracts(&self) -> Result<Vec<Contract>, ContractManagementError> {
        self.contract_repo
            .list()
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))
    }

    /// Get contract by ID
    pub async fn get_contract(
        &self,
        contract_id: ContractId,
    ) -> Result<Option<Contract>, ContractManagementError> {
        self.contract_repo
            .get(&contract_id)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))
    }

    /// Delete a contract
    pub async fn delete_contract(
        &self,
        contract_id: ContractId,
    ) -> Result<(), ContractManagementError> {
        info!("Deleting contract: {}", contract_id);

        self.contract_repo
            .delete(&contract_id)
            .await
            .map_err(|e| ContractManagementError::RepositoryError(e.to_string()))?;

        // Publish domain event
        let event = ContractEvent::ContractDeleted {
            contract_id: contract_id.clone(),
            timestamp: SystemTime::now(),
        };

        if let Err(e) = self.event_publisher.publish(event).await {
            warn!("Failed to publish contract deletion event: {}", e);
        }

        info!("Successfully deleted contract: {}", contract_id);
        Ok(())
    }
}

/// Use case for generating and publishing reports
pub struct ReportingUseCase<R: ContractReportSink> {
    report_sink: R,
    notification_sender: Box<dyn ContractNotificationSender>,
    config: ReportingConfig,
}

impl<R: ContractReportSink> ReportingUseCase<R> {
    pub fn new(
        report_sink: R,
        notification_sender: Box<dyn ContractNotificationSender>,
        config: ReportingConfig,
    ) -> Self {
        Self {
            report_sink,
            notification_sender,
            config,
        }
    }

    /// Generate and publish comprehensive validation report
    pub async fn publish_validation_report(
        &self,
        result: &ContractValidationResult,
    ) -> Result<ReportPublicationResult, ReportingError> {
        info!(
            "Publishing validation report for contract: {}",
            result.contract_id
        );

        let mut published_formats = Vec::new();

        // Generate JSON report
        if let Err(e) = self
            .report_sink
            .write_report(ReportFormat::Json, result)
            .await
        {
            error!("Failed to write JSON report: {}", e);
        } else {
            published_formats.push("JSON".to_string());
        }

        // Generate JUnit XML for CI integration
        if let Err(e) = self
            .report_sink
            .write_report(ReportFormat::JUnit, result)
            .await
        {
            error!("Failed to write JUnit report: {}", e);
        } else {
            published_formats.push("JUnit".to_string());
        }

        // Generate HTML report if configured
        if self.config.generate_html {
            if let Err(e) = self
                .report_sink
                .write_report(ReportFormat::Html, result)
                .await
            {
                error!("Failed to write HTML report: {}", e);
            } else {
                published_formats.push("HTML".to_string());
            }
        }

        // Generate badge if configured
        let badge_url = if self.config.generate_badges {
            match self
                .report_sink
                .write_report(ReportFormat::Badge, result)
                .await
            {
                Ok(url) => url,
                Err(e) => {
                    error!("Failed to generate badge: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(ReportPublicationResult {
            contract_id: result.contract_id.clone(),
            published_formats,
            badge_url,
        })
    }

    /// Send notifications about validation results
    pub async fn send_notifications(
        &self,
        result: &ContractValidationResult,
        notification_config: &NotificationConfig,
    ) -> Result<(), ReportingError> {
        // Send Slack notification if configured
        if let Some(webhook_url) = &notification_config.slack_webhook_url {
            if let Err(e) = self
                .notification_sender
                .send_slack_notification(webhook_url, result)
                .await
            {
                error!("Failed to send Slack notification: {}", e);
            } else {
                info!(
                    "Sent Slack notification for contract: {}",
                    result.contract_id
                );
            }
        }

        // Send email notifications if configured
        if !notification_config.email_recipients.is_empty() {
            if let Err(e) = self
                .notification_sender
                .send_email_notification(&notification_config.email_recipients, result)
                .await
            {
                error!("Failed to send email notifications: {}", e);
            } else {
                info!(
                    "Sent email notifications for contract: {}",
                    result.contract_id
                );
            }
        }

        Ok(())
    }
}

/// Orchestrator use case that combines validation, reporting, and notifications
pub struct ContractTestingOrchestrator<T, H, M, S, R>
where
    T: ContractRepository,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    S: ServiceSpecLoader,
    R: ContractReportSink,
{
    validate_use_case: ValidateContractUseCase<T, H, M, S>,
    reporting_use_case: ReportingUseCase<R>,
    config_provider: Box<dyn ContractConfigurationProvider>,
}

impl<T, H, M, S, R> ContractTestingOrchestrator<T, H, M, S, R>
where
    T: ContractRepository,
    H: ContractHttpClient,
    M: ContractMockApiRunner,
    S: ServiceSpecLoader,
    R: ContractReportSink,
{
    pub fn new(
        validate_use_case: ValidateContractUseCase<T, H, M, S>,
        reporting_use_case: ReportingUseCase<R>,
        config_provider: Box<dyn ContractConfigurationProvider>,
    ) -> Self {
        Self {
            validate_use_case,
            reporting_use_case,
            config_provider,
        }
    }

    /// Execute full contract testing workflow
    pub async fn execute_full_workflow(
        &self,
        contract_id: ContractId,
        environment: String,
        policy_name: String,
    ) -> Result<WorkflowResult, OrchestrationError> {
        info!(
            "Starting contract testing workflow for: {} in environment: {}",
            contract_id, environment
        );

        // Load configurations
        let real_api_config = self
            .config_provider
            .get_real_api_config(&environment)
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        let compatibility_policy = self
            .config_provider
            .get_compatibility_policy(&policy_name)
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        let notification_config = self
            .config_provider
            .get_notification_config()
            .map_err(|e| OrchestrationError::ConfigurationError(e.to_string()))?;

        // Execute validation
        let validation_result = self
            .validate_use_case
            .execute(contract_id.clone(), real_api_config, compatibility_policy)
            .await
            .map_err(|e| OrchestrationError::ValidationError(e.to_string()))?;

        // Publish reports
        let report_result = self
            .reporting_use_case
            .publish_validation_report(&validation_result)
            .await
            .map_err(|e| OrchestrationError::ReportingError(e.to_string()))?;

        // Send notifications
        self.reporting_use_case
            .send_notifications(&validation_result, &notification_config)
            .await
            .map_err(|e| OrchestrationError::ReportingError(e.to_string()))?;

        let workflow_result = WorkflowResult {
            validation_result,
            report_result,
            success: true,
        };

        info!(
            "Contract testing workflow completed successfully - Compatibility: {}",
            workflow_result.validation_result.is_compatible
        );

        Ok(workflow_result)
    }
}

// === RESULT TYPES ===

#[derive(Debug, Clone)]
pub struct ReportPublicationResult {
    pub contract_id: ContractId,
    pub published_formats: Vec<String>,
    pub badge_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub validation_result: ContractValidationResult,
    pub report_result: ReportPublicationResult,
    pub success: bool,
}

// === ERROR TYPES ===

#[derive(Debug, thiserror::Error)]
pub enum ContractUseCaseError {
    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Spec load error: {0}")]
    SpecLoadError(String),

    #[error("Spec validation error: {0}")]
    SpecValidationError(String),

    #[error("Mock API error: {0}")]
    MockApiError(String),

    #[error("HTTP client error: {0}")]
    HttpClientError(String),

    #[error("Template error: {0}")]
    TemplateError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ContractManagementError {
    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Invalid spec: {0}")]
    InvalidSpec(String),

    #[error("Contract creation error: {0}")]
    ContractCreationError(String),
}

impl From<ContractValidationError> for ContractManagementError {
    fn from(err: ContractValidationError) -> Self {
        ContractManagementError::ContractCreationError(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReportingError {
    #[error("Report generation error: {0}")]
    ReportGenerationError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Reporting error: {0}")]
    ReportingError(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),
}
