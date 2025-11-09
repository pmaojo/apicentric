//! A simple file-based implementation of the `ContractReportSink` port.
//!
//! This module provides a `FileReportSink` that writes contract validation
//! reports to the file system.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::fs;

use crate::domain::contract_testing::ContractValidationResult;
use crate::domain::ports::contract::{ContractReportSink, ReportError, ReportFormat};

/// A simple file-based implementation of `ContractReportSink`.
///
/// The adapter writes reports under the provided output directory. For
/// demonstration purposes only JSON output is fully implemented; other
/// formats are treated as no-ops. When a badge is requested a placeholder
/// file is written and its path returned.
pub struct FileReportSink {
    output_dir: PathBuf,
}

impl FileReportSink {
    /// Creates a new `FileReportSink`.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - The directory where reports will be written.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

#[async_trait]
impl ContractReportSink for FileReportSink {
    /// Writes a contract validation report to the file system.
    ///
    /// # Arguments
    ///
    /// * `format` - The format of the report to write.
    /// * `result` - The contract validation result to write.
    async fn write_report(
        &self,
        format: ReportFormat,
        result: &ContractValidationResult,
    ) -> Result<Option<String>, ReportError> {
        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|e| ReportError::FileSystemError(e.to_string()))?;

        match format {
            ReportFormat::Json => {
                let path = self.output_dir.join("report.json");
                let data = serde_json::to_string_pretty(result)
                    .map_err(|e| ReportError::FormatError(e.to_string()))?;
                fs::write(&path, data)
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(None)
            }
            ReportFormat::JUnit => {
                let path = self.output_dir.join("report.junit.xml");
                let xml = self.render_junit_report(result)?;
                fs::write(&path, xml)
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(None)
            }
            ReportFormat::Html => {
                let path = self.output_dir.join("report.html");
                let html = self.render_html_report(result)?;
                fs::write(&path, html)
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(None)
            }
            ReportFormat::Badge => {
                let path = self.output_dir.join("badge.svg");
                fs::write(&path, "<svg></svg>")
                    .await
                    .map_err(|e| ReportError::WriteError(e.to_string()))?;
                Ok(Some(path.to_string_lossy().to_string()))
            }
        }
    }
}

impl FileReportSink {
    fn render_junit_report(
        &self,
        result: &ContractValidationResult,
    ) -> Result<String, ReportError> {
        let tests = result.scenario_results.len();
        let failures = result
            .scenario_results
            .iter()
            .filter(|scenario| scenario.compliance_issue.is_some())
            .count();
        let timestamp: DateTime<Utc> = result.validation_timestamp.into();
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<testsuite name=\"ContractValidation\" tests=\"{}\" failures=\"{}\" timestamp=\"{}\">\n",
            tests,
            failures,
            timestamp.to_rfc3339()
        ));

        for scenario in &result.scenario_results {
            xml.push_str(&format!(
                "  <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\">\n",
                Self::escape_markup(&scenario.scenario_id),
                Self::escape_markup(&result.contract_id.to_string()),
                scenario.duration_ms as f64 / 1000.0
            ));

            if let Some(issue) = &scenario.compliance_issue {
                let details = issue
                    .details
                    .as_ref()
                    .map(|d| serde_json::to_string_pretty(d).unwrap_or_default())
                    .unwrap_or_default();
                xml.push_str(&format!(
                    "    <failure message=\"{}\" type=\"{:?}\">{}</failure>\n",
                    Self::escape_markup(&issue.description),
                    issue.issue_type,
                    Self::escape_markup(&details)
                ));
            }

            xml.push_str("  </testcase>\n");
        }

        xml.push_str("</testsuite>\n");
        Ok(xml)
    }

    fn render_html_report(&self, result: &ContractValidationResult) -> Result<String, ReportError> {
        let tests = result.scenario_results.len();
        let failures = result
            .scenario_results
            .iter()
            .filter(|scenario| scenario.compliance_issue.is_some())
            .count();
        let timestamp: DateTime<Utc> = result.validation_timestamp.into();
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
        html.push_str("<title>Contract Validation Report</title>");
        html.push_str("<style>body{font-family:Arial,Helvetica,sans-serif;margin:2rem;}table{border-collapse:collapse;width:100%;}th,td{border:1px solid #ddd;padding:0.75rem;text-align:left;}th{background:#f5f5f5;}tr:nth-child(even){background:#fafafa;} .pass{color:#1b5e20;} .fail{color:#c62828;font-weight:bold;}</style>");
        html.push_str("</head><body>");
        html.push_str(&format!(
            "<h1>Contract Validation Report</h1><p><strong>Contract:</strong> {}</p><p><strong>Environment:</strong> {}</p><p><strong>Validated at:</strong> {}</p><p><strong>Compliance score:</strong> {:.2}</p><p><strong>Scenarios:</strong> {} &mdash; <strong>Failures:</strong> {}</p>",
            Self::escape_markup(&result.contract_id.to_string()),
            Self::escape_markup(&result.environment),
            timestamp.to_rfc3339(),
            result.compliance_score,
            tests,
            failures
        ));

        html.push_str(
            "<table><thead><tr><th>Scenario</th><th>Path</th><th>Status</th><th>Expected vs Real</th><th>Issue</th></tr></thead><tbody>",
        );

        for scenario in &result.scenario_results {
            let status_class = if scenario.compliance_issue.is_some() {
                "fail"
            } else {
                "pass"
            };
            let status_text = if scenario.compliance_issue.is_some() {
                "FAIL"
            } else {
                "PASS"
            };

            let expected_status = scenario
                .expected_response
                .as_ref()
                .map(|resp| resp.status().to_string())
                .unwrap_or_else(|| "-".into());
            let real_status = scenario
                .real_response
                .as_ref()
                .map(|resp| resp.status().to_string())
                .unwrap_or_else(|| "-".into());
            let comparison = format!("expected {} &ndash; real {}", expected_status, real_status);

            let scenario_path = scenario
                .compliance_issue
                .as_ref()
                .map(|issue| issue.scenario_path.clone())
                .unwrap_or_else(|| scenario.scenario_id.clone());

            let issue_description = scenario
                .compliance_issue
                .as_ref()
                .map(|issue| Self::escape_markup(&issue.description))
                .unwrap_or_else(|| "Compatible".into());
            let issue_details = scenario
                .compliance_issue
                .as_ref()
                .and_then(|issue| issue.details.as_ref())
                .map(|details| {
                    Self::escape_markup(&serde_json::to_string_pretty(details).unwrap_or_default())
                })
                .unwrap_or_else(String::new);

            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td>{}<br><small>{}</small></td></tr>",
                Self::escape_markup(&scenario.scenario_id),
                Self::escape_markup(&scenario_path),
                status_class,
                status_text,
                comparison,
                issue_description,
                issue_details
            ));
        }

        html.push_str("</tbody></table></body></html>");
        Ok(html)
    }

    fn escape_markup(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
