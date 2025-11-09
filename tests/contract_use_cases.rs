use apicentric::domain::contract::*;
use apicentric::domain::contract_testing::*;
use apicentric::domain::ports::contract::*;
use async_trait::async_trait;
use std::collections::HashMap;

// ----- Loader Test -----
struct MockRepo {
    contract: Option<Contract>,
}

#[async_trait]
impl ContractRepository for MockRepo {
    async fn save(&self, _contract: &Contract) -> Result<(), RepositoryError> {
        Ok(())
    }
    async fn get(&self, _id: &ContractId) -> Result<Option<Contract>, RepositoryError> {
        Ok(self.contract.clone())
    }
    async fn list(&self) -> Result<Vec<Contract>, RepositoryError> {
        Ok(vec![])
    }
    async fn delete(&self, _id: &ContractId) -> Result<(), RepositoryError> {
        Ok(())
    }
    async fn find_by_service(&self, _s: &str) -> Result<Vec<Contract>, RepositoryError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn loads_contract_from_repository() {
    let id = ContractId::new("1".into()).unwrap();
    let contract = Contract::new(id.clone(), "svc".into(), "spec.yml".into(), None).unwrap();
    let repo = MockRepo {
        contract: Some(contract.clone()),
    };
    let loader = ContractLoadingUseCase::new(repo);
    let loaded = loader.execute(&id).await.unwrap();
    assert_eq!(loaded.id, contract.id);
}

// ----- Validator Test -----
struct MockSpecLoader;

#[async_trait]
impl ServiceSpecLoader for MockSpecLoader {
    async fn load(&self, _path: &str) -> Result<ServiceSpec, SpecLoaderError> {
        Ok(ServiceSpec {
            name: "svc".into(),
            port: 0,
            base_path: "/".into(),
            fixtures: serde_json::Value::Null,
            endpoints: vec![],
        })
    }
    async fn validate(&self, _spec: &ServiceSpec) -> Result<(), SpecLoaderError> {
        Ok(())
    }
    fn extract_scenarios(
        &self,
        _spec: &ServiceSpec,
    ) -> Result<Vec<ValidationScenario>, SpecLoaderError> {
        let mut expected_headers = HashMap::new();
        expected_headers.insert("content-type".into(), "application/json".into());

        Ok(vec![ValidationScenario::new(
            "1".into(),
            "/ep".into(),
            HttpMethod::GET,
        )
        .with_expected_response(
            200,
            expected_headers,
            ResponseBody::Json(serde_json::json!({"ok": true})),
        )])
    }
}

#[tokio::test]
async fn validates_spec_and_extracts_scenarios() {
    let contract = Contract::new(
        ContractId::new("c1".into()).unwrap(),
        "svc".into(),
        "spec".into(),
        None,
    )
    .unwrap();
    let validator = SpecValidationUseCase::new(MockSpecLoader);
    let scenarios = validator.execute(&contract).await.unwrap();
    assert_eq!(scenarios.len(), 1);
}

// ----- Executor Test -----
struct MockHttpClient;

#[async_trait]
impl ContractHttpClient for MockHttpClient {
    async fn execute_request(
        &self,
        _base: &ApiUrl,
        _cfg: &RealApiConfig,
        _sc: &ValidationScenario,
    ) -> Result<ApiResponse, HttpClientError> {
        Ok(ApiResponse::new(
            200,
            HashMap::new(),
            ResponseBody::Json(serde_json::json!({"ok": true})),
            10,
        ))
    }
    async fn health_check(&self, _base_url: &ApiUrl) -> Result<bool, HttpClientError> {
        Ok(true)
    }
}

struct FailingHttpClient;

#[async_trait]
impl ContractHttpClient for FailingHttpClient {
    async fn execute_request(
        &self,
        _base: &ApiUrl,
        _cfg: &RealApiConfig,
        _sc: &ValidationScenario,
    ) -> Result<ApiResponse, HttpClientError> {
        Ok(ApiResponse::new(
            500,
            HashMap::new(),
            ResponseBody::Json(serde_json::json!({"error": true})),
            12,
        ))
    }

    async fn health_check(&self, _base_url: &ApiUrl) -> Result<bool, HttpClientError> {
        Ok(true)
    }
}

struct MockMockApi;

#[async_trait]
impl ContractMockApiRunner for MockMockApi {
    async fn start(&self, _path: &str) -> Result<MockApiHandle, MockApiError> {
        Ok(MockApiHandle {
            port: 0,
            base_url: "mock".into(),
            process_id: None,
        })
    }
    async fn stop(&self, _handle: MockApiHandle) -> Result<(), MockApiError> {
        Ok(())
    }
    async fn execute_request(
        &self,
        _handle: &MockApiHandle,
        _sc: &ValidationScenario,
    ) -> Result<ApiResponse, MockApiError> {
        Ok(ApiResponse::new(
            200,
            HashMap::new(),
            ResponseBody::Json(serde_json::json!({"ok": true})),
            5,
        ))
    }
    async fn render_response(
        &self,
        _sc: &ValidationScenario,
        _fx: &serde_json::Value,
    ) -> Result<ApiResponse, MockApiError> {
        unimplemented!()
    }
}

struct NoOpMetrics;
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

struct NoOpTracer;
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

struct NoOpPublisher;
#[async_trait]
impl ContractEventPublisher for NoOpPublisher {
    async fn publish(&self, _event: ContractEvent) -> Result<(), EventError> {
        Ok(())
    }
}

#[tokio::test]
async fn executes_scenarios_and_returns_results() {
    let contract = Contract::new(
        ContractId::new("c1".into()).unwrap(),
        "svc".into(),
        "spec".into(),
        None,
    )
    .unwrap();
    let scenarios = vec![
        ValidationScenario::new("1".into(), "/ep".into(), HttpMethod::GET).with_expected_response(
            200,
            HashMap::new(),
            ResponseBody::Json(serde_json::json!({"ok": true})),
        ),
    ];
    let real = RealApiConfig::new(
        "dev".into(),
        ApiUrl::new("http://real".into()).unwrap(),
        None,
        RetryAttempts::new(0).unwrap(),
    );
    let policy = CompatibilityPolicy::strict();
    let executor = ScenarioExecutionUseCase::new(
        MockHttpClient,
        MockMockApi,
        Box::new(NoOpMetrics),
        Box::new(NoOpTracer),
        Box::new(NoOpPublisher),
    );
    let result = executor
        .execute(&contract, &real, &scenarios, &policy)
        .await
        .unwrap();
    assert_eq!(result.scenario_results.len(), 1);
    assert!(result.scenario_results[0].expected_response.is_some());
    assert!(result.scenario_results[0].compliance_issue.is_none());
    assert_eq!(
        result.scenario_results[0]
            .expected_response
            .as_ref()
            .unwrap()
            .status(),
        200
    );
}

#[tokio::test]
async fn assigns_scenario_path_to_issues() {
    let contract = Contract::new(
        ContractId::new("c1".into()).unwrap(),
        "svc".into(),
        "spec".into(),
        None,
    )
    .unwrap();
    let scenarios = vec![
        ValidationScenario::new("1".into(), "/ep".into(), HttpMethod::GET).with_expected_response(
            200,
            HashMap::new(),
            ResponseBody::Json(serde_json::json!({"ok": true})),
        ),
    ];
    let real = RealApiConfig::new(
        "dev".into(),
        ApiUrl::new("http://real".into()).unwrap(),
        None,
        RetryAttempts::new(0).unwrap(),
    );
    let policy = CompatibilityPolicy::strict();
    let executor = ScenarioExecutionUseCase::new(
        FailingHttpClient,
        MockMockApi,
        Box::new(NoOpMetrics),
        Box::new(NoOpTracer),
        Box::new(NoOpPublisher),
    );

    let result = executor
        .execute(&contract, &real, &scenarios, &policy)
        .await
        .unwrap();
    let issue = result.scenario_results[0]
        .compliance_issue
        .as_ref()
        .unwrap();
    assert_eq!(issue.scenario_path, "/ep");
}

// ----- Reporting Test -----
struct MockSink;

#[async_trait]
impl ContractReportSink for MockSink {
    async fn write_report(
        &self,
        _f: ReportFormat,
        _r: &ContractValidationResult,
    ) -> Result<Option<String>, ReportError> {
        Ok(None)
    }
}

struct MockNotifier;

#[async_trait]
impl ContractNotificationSender for MockNotifier {
    async fn send_slack_notification(
        &self,
        _url: &str,
        _r: &ContractValidationResult,
    ) -> Result<(), NotificationError> {
        Ok(())
    }
    async fn send_email_notification(
        &self,
        _r: &[String],
        _res: &ContractValidationResult,
    ) -> Result<(), NotificationError> {
        Ok(())
    }
}

#[tokio::test]
async fn publishes_reports_in_multiple_formats() {
    let reporting = ReportingUseCase::new(
        MockSink,
        Box::new(MockNotifier),
        ReportingConfig {
            output_dir: ".".into(),
            generate_html: false,
            generate_badges: false,
            retention_days: 0,
            template_path: None,
        },
    );
    let result = ContractValidationResult {
        contract_id: ContractId::new("c1".into()).unwrap(),
        validation_timestamp: std::time::SystemTime::now(),
        compliance_score: 1.0,
        is_compatible: true,
        issues: vec![],
        scenario_results: vec![],
        environment: "dev".into(),
    };
    let pub_result = reporting.publish_validation_report(&result).await.unwrap();
    assert!(pub_result.published_formats.contains(&"JSON".into()));
    assert!(pub_result.published_formats.contains(&"JUnit".into()));
}
