use std::time::SystemTime;

use super::ContractUseCaseError;
use crate::domain::contract_testing::{
    ApiResponse, CompatibilityPolicy, ComplianceIssue, ComplianceIssueType, ComplianceSeverity,
    Contract, ContractEvent, ContractValidationResult, RealApiConfig, ResponseBody,
    ScenarioValidationResult, ValidationScenario,
};
use crate::domain::ports::contract::{
    ContractEventPublisher, ContractHttpClient, ContractMetricsCollector, ContractMockApiRunner,
    ContractTracingCollector, MockApiHandle,
};
use tracing::{debug, error, warn};

pub struct ScenarioExecutionUseCase<H, M>
where
    H: ContractHttpClient,
    M: ContractMockApiRunner,
{
    http_client: H,
    mock_api: M,
    metrics: Box<dyn ContractMetricsCollector>,
    tracer: Box<dyn ContractTracingCollector>,
    event_publisher: Box<dyn ContractEventPublisher>,
}

impl<H, M> ScenarioExecutionUseCase<H, M>
where
    H: ContractHttpClient,
    M: ContractMockApiRunner,
{
    pub fn new(
        http_client: H,
        mock_api: M,
        metrics: Box<dyn ContractMetricsCollector>,
        tracer: Box<dyn ContractTracingCollector>,
        event_publisher: Box<dyn ContractEventPublisher>,
    ) -> Self {
        Self {
            http_client,
            mock_api,
            metrics,
            tracer,
            event_publisher,
        }
    }

    pub async fn execute(
        &self,
        contract: &Contract,
        real_api_config: &RealApiConfig,
        scenarios: &[ValidationScenario],
        policy: &CompatibilityPolicy,
    ) -> Result<ContractValidationResult, ContractUseCaseError> {
        let span = self.tracer.start_span("validate_contract", &contract.id);
        self.tracer
            .add_span_attribute(&span, "contract_id", &contract.id.to_string());

        let start_time = std::time::Instant::now();
        self.metrics.record_validation_started(&contract.id);

        let requires_mock = scenarios
            .iter()
            .any(|scenario| scenario.expected_status.is_none() || scenario.expected_body.is_none());

        let mock_handle = if requires_mock {
            Some(
                self.mock_api
                    .start(&contract.spec_path)
                    .await
                    .map_err(|e| ContractUseCaseError::MockApiError(e.to_string()))?,
            )
        } else {
            None
        };

        let validation_result = match self
            .validate_scenarios(
                contract,
                real_api_config,
                mock_handle.as_ref(),
                scenarios,
                policy,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                if let Some(handle) = mock_handle {
                    let _ = self.mock_api.stop(handle).await;
                }
                return Err(e);
            }
        };

        if let Some(handle) = mock_handle {
            let _ = self.mock_api.stop(handle).await;
        }

        let duration = start_time.elapsed();
        self.metrics.record_validation_completed(
            &contract.id,
            duration.as_millis() as u64,
            validation_result.compliance_score,
        );

        self.publish_completion_event(&contract.id, &validation_result)
            .await;
        self.tracer.finish_span(span);

        Ok(validation_result)
    }

    async fn publish_completion_event(
        &self,
        contract_id: &crate::domain::contract_testing::ContractId,
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
        mock_handle: Option<&MockApiHandle>,
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
                        expected_response: None,
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
        mock_handle: Option<&MockApiHandle>,
        scenario: &ValidationScenario,
        policy: &CompatibilityPolicy,
    ) -> Result<ScenarioValidationResult, ContractUseCaseError> {
        let start_time = std::time::Instant::now();

        let mock_response = if let Some(handle) = mock_handle {
            Some(
                self.mock_api
                    .execute_request(handle, scenario)
                    .await
                    .map_err(|e| ContractUseCaseError::MockApiError(e.to_string()))?,
            )
        } else {
            None
        };

        let scenario_expected =
            self.build_expected_response(scenario, mock_response.as_ref(), policy);

        let real_response = self
            .http_client
            .execute_request(&real_api_config.base_url, real_api_config, scenario)
            .await
            .map_err(|e| ContractUseCaseError::HttpClientError(e.to_string()))?;

        self.metrics.record_api_response_time(
            &scenario.path,
            real_response.duration_ms,
            real_response.status_code,
        );

        let compliance_issue = scenario_expected
            .as_ref()
            .and_then(|expected| {
                contract.validate_response_compatibility(expected, &real_response, policy)
            })
            .map(|mut issue| {
                issue.scenario_path = scenario.path.clone();
                issue
            });

        let duration = start_time.elapsed();

        let expected_response = scenario_expected;

        Ok(ScenarioValidationResult {
            scenario_id: scenario.id.clone(),
            mock_response,
            real_response: Some(real_response.clone()),
            expected_response,
            compliance_issue,
            duration_ms: duration.as_millis() as u64,
        })
    }

    fn build_expected_response(
        &self,
        scenario: &ValidationScenario,
        mock_response: Option<&ApiResponse>,
        _policy: &CompatibilityPolicy,
    ) -> Option<ApiResponse> {
        let mut expected = mock_response.cloned();

        if let Some(status) = scenario.expected_status {
            let mut response = expected.unwrap_or_else(|| {
                ApiResponse::new(
                    status,
                    scenario.expected_headers.clone(),
                    ResponseBody::Text("".into()),
                    0,
                )
            });
            response.status_code = status;

            if !scenario.expected_headers.is_empty() {
                response.headers = scenario.expected_headers.clone();
            }

            if let Some(body) = scenario.expected_body.clone() {
                response.body = body;
            }
            expected = Some(response);
        }

        expected
    }
}
