use crate::domain::contract_testing::{
    ApiUrl, ComplianceIssue, ComplianceIssueType, ComplianceSeverity, RealApiConfig,
    ScenarioValidationResult, ValidationScenario,
};
use crate::domain::ports::contract::{ContractHttpClient, ContractMockApiRunner, MockApiHandle};

/// Executes contract validation scenarios against mock and real APIs.
pub struct ContractExecutor<H, M>
where
    H: ContractHttpClient,
    M: ContractMockApiRunner,
{
    http_client: H,
    mock_runner: M,
}

impl<H, M> ContractExecutor<H, M>
where
    H: ContractHttpClient,
    M: ContractMockApiRunner,
{
    /// Create a new executor with the given ports.
    pub fn new(http_client: H, mock_runner: M) -> Self {
        Self {
            http_client,
            mock_runner,
        }
    }

    /// Execute a scenario and return its validation result.
    pub async fn execute(
        &self,
        base_url: &ApiUrl,
        config: &RealApiConfig,
        scenario: &ValidationScenario,
        mock_handle: &MockApiHandle,
    ) -> ScenarioValidationResult {
        let mock_response = self
            .mock_runner
            .execute_request(mock_handle, scenario)
            .await
            .ok();
        let real_response = self
            .http_client
            .execute_request(base_url, config, scenario)
            .await
            .ok();

        let compliance_issue = match (&mock_response, &real_response) {
            (Some(m), Some(r)) if m.status() != r.status() => Some(ComplianceIssue {
                issue_type: ComplianceIssueType::StatusCodeMismatch,
                severity: ComplianceSeverity::Medium,
                description: format!(
                    "Status code mismatch: expected {} got {}",
                    m.status(),
                    r.status()
                ),
                scenario_path: scenario.path().to_string(),
                details: None,
            }),
            _ => None,
        };

        let expected_response = mock_response.clone();

        ScenarioValidationResult {
            scenario_id: scenario.id.clone(),
            mock_response,
            real_response,
            expected_response,
            compliance_issue,
            duration_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;

    use crate::domain::contract_testing::{
        value_objects::RetryAttempts, ApiResponse, HttpMethod, ResponseBody,
    };
    use crate::domain::ports::contract::{HttpClientError, MockApiError};

    struct FakeHttpClient;
    #[async_trait]
    impl ContractHttpClient for FakeHttpClient {
        async fn execute_request(
            &self,
            _base_url: &ApiUrl,
            _config: &RealApiConfig,
            _scenario: &ValidationScenario,
        ) -> Result<ApiResponse, HttpClientError> {
            Ok(ApiResponse::new(
                500,
                HashMap::new(),
                ResponseBody::Text("error".into()),
                5,
            ))
        }

        async fn health_check(&self, _base_url: &ApiUrl) -> Result<bool, HttpClientError> {
            Ok(true)
        }
    }

    struct FakeMockRunner;
    #[async_trait]
    impl ContractMockApiRunner for FakeMockRunner {
        async fn start(&self, _service_spec_path: &str) -> Result<MockApiHandle, MockApiError> {
            Ok(MockApiHandle {
                port: 0,
                base_url: String::new(),
                process_id: None,
            })
        }

        async fn stop(&self, _handle: MockApiHandle) -> Result<(), MockApiError> {
            Ok(())
        }

        async fn execute_request(
            &self,
            _handle: &MockApiHandle,
            _scenario: &ValidationScenario,
        ) -> Result<ApiResponse, MockApiError> {
            Ok(ApiResponse::new(
                200,
                HashMap::new(),
                ResponseBody::Text("ok".into()),
                1,
            ))
        }

        async fn render_response(
            &self,
            _scenario: &ValidationScenario,
            _fixtures: &serde_json::Value,
        ) -> Result<ApiResponse, MockApiError> {
            Ok(ApiResponse::new(
                200,
                HashMap::new(),
                ResponseBody::Text(String::new()),
                1,
            ))
        }
    }

    #[tokio::test]
    async fn detects_status_code_mismatch() {
        let executor = ContractExecutor::new(FakeHttpClient, FakeMockRunner);
        let scenario = ValidationScenario::new("s1".into(), "/ping".into(), HttpMethod::GET);
        let base_url = ApiUrl::new("http://example.com".into()).unwrap();
        let config = RealApiConfig::new(
            "test".into(),
            base_url.clone(),
            None,
            RetryAttempts::new(0).unwrap(),
        );
        let handle = MockApiHandle {
            port: 0,
            base_url: String::new(),
            process_id: None,
        };

        let result = executor
            .execute(&base_url, &config, &scenario, &handle)
            .await;
        assert!(result.compliance_issue.is_some());
    }
}
