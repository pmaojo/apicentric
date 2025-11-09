use apicentric::adapters::report_sink::FileReportSink;
use apicentric::domain::contract_testing::*;
use apicentric::domain::ports::contract::{ContractReportSink, ReportFormat};
use std::collections::HashMap;
use tempfile::tempdir;

fn sample_result() -> ContractValidationResult {
    let contract_id = ContractId::new("contract-1".into()).unwrap();
    let scenario_result = ScenarioValidationResult {
        scenario_id: "s1".into(),
        mock_response: None,
        real_response: Some(ApiResponse::new(
            500,
            HashMap::new(),
            ResponseBody::Text("error".into()),
            15,
        )),
        expected_response: Some(ApiResponse::new(
            200,
            HashMap::new(),
            ResponseBody::Text("ok".into()),
            0,
        )),
        compliance_issue: Some(ComplianceIssue {
            issue_type: ComplianceIssueType::StatusCodeMismatch,
            severity: ComplianceSeverity::High,
            description: "Status code mismatch".into(),
            scenario_path: "/endpoint".into(),
            details: None,
        }),
        duration_ms: 15,
    };

    ContractValidationResult {
        contract_id,
        validation_timestamp: std::time::SystemTime::now(),
        compliance_score: 0.0,
        is_compatible: false,
        issues: vec![scenario_result.compliance_issue.clone().unwrap()],
        scenario_results: vec![scenario_result],
        environment: "dev".into(),
    }
}

#[tokio::test]
async fn writes_junit_report_with_failures() {
    let dir = tempdir().unwrap();
    let sink = FileReportSink::new(dir.path().into());
    let result = sample_result();

    sink.write_report(ReportFormat::JUnit, &result)
        .await
        .unwrap();

    let junit_path = dir.path().join("report.junit.xml");
    let contents = tokio::fs::read_to_string(&junit_path).await.unwrap();
    assert!(contents.contains("<testsuite"));
    assert!(contents.contains("failure"));
}

#[tokio::test]
async fn writes_html_report_with_summary() {
    let dir = tempdir().unwrap();
    let sink = FileReportSink::new(dir.path().into());
    let result = sample_result();

    sink.write_report(ReportFormat::Html, &result)
        .await
        .unwrap();

    let html_path = dir.path().join("report.html");
    let contents = tokio::fs::read_to_string(&html_path).await.unwrap();
    assert!(contents.contains("Contract Validation Report"));
    assert!(contents.contains("/endpoint"));
}
