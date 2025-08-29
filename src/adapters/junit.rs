use crate::{PulseError, PulseResult};
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuites {
    #[serde(rename = "testsuite")]
    pub test_suites: Vec<TestSuite>,
    pub name: String,
    pub tests: usize,
    pub failures: usize,
    pub time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuite {
    #[serde(rename = "testcase", default)]
    pub test_cases: Vec<TestCase>,
    pub name: String,
    pub tests: usize,
    pub failures: usize,
    pub time: f64,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub classname: String,
    pub name: String,
    pub time: f64,
    #[serde(rename = "failure")]
    pub failures: Option<Vec<Failure>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Failure {
    pub message: String,
    #[serde(rename = "$value")]
    pub details: String,
}

#[derive(Debug, Serialize)]
pub struct ConsolidatedReport {
    pub suites: Vec<TestSuite>,
    pub total_tests: usize,
    pub total_failures: usize,
    pub total_time: f64,
    pub flaky_tests: Vec<String>,
}

impl ConsolidatedReport {
    pub fn total_tests(&self) -> usize {
        self.total_tests
    }

    pub fn failed_tests(&self) -> usize {
        self.total_failures
    }

    pub fn passed_tests(&self) -> usize {
        self.total_tests.saturating_sub(self.total_failures)
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests() as f64 / self.total_tests as f64) * 100.0
        }
    }
}

pub struct JUnitAdapter {
    reports_dir: String,
}

impl JUnitAdapter {
    pub fn new(reports_dir: String) -> Self {
        Self { reports_dir }
    }

    pub fn parse_reports(&self) -> PulseResult<ConsolidatedReport> {
        let mut consolidated = ConsolidatedReport {
            suites: Vec::new(),
            total_tests: 0,
            total_failures: 0,
            total_time: 0.0,
            flaky_tests: Vec::new(),
        };

        let reports_path = Path::new(&self.reports_dir);
        for entry in fs::read_dir(reports_path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot read reports directory: {}", e),
                Some("Check that the reports directory exists and is readable"),
            )
        })? {
            let entry = entry.map_err(|e| {
                PulseError::fs_error(
                    format!("Error reading directory entry: {}", e),
                    None::<String>,
                )
            })?;
            if entry.path().extension().map_or(false, |ext| ext == "xml") {
                let content = fs::read_to_string(entry.path()).map_err(|e| {
                    PulseError::fs_error(
                        format!("Cannot read report file: {}", e),
                        Some("Check file permissions and ensure the file is not corrupted"),
                    )
                })?;
                // Try to parse as testsuites first (Cypress format), then as single testsuite
                let content_trimmed = content.trim();
                if content_trimmed.is_empty() {
                    continue; // Skip empty files
                }

                if content_trimmed.starts_with("<?xml") && content_trimmed.contains("<testsuites") {
                    // Parse as testsuites (Cypress format)
                    match from_str::<TestSuites>(&content) {
                        Ok(test_suites) => {
                            for suite in test_suites.test_suites {
                                consolidated.total_tests += suite.tests;
                                consolidated.total_failures += suite.failures;
                                consolidated.total_time += suite.time;
                                consolidated.suites.push(suite);
                            }
                        }
                        Err(e) => {
                            println!("⚠️ Skipping malformed XML file {:?}: {}", entry.path(), e);
                            continue;
                        }
                    }
                } else if content_trimmed.starts_with("<?xml")
                    && content_trimmed.contains("<testsuite")
                {
                    // Parse as single testsuite
                    match from_str::<TestSuite>(&content) {
                        Ok(suite) => {
                            consolidated.total_tests += suite.tests;
                            consolidated.total_failures += suite.failures;
                            consolidated.total_time += suite.time;
                            consolidated.suites.push(suite);
                        }
                        Err(e) => {
                            println!("⚠️ Skipping malformed XML file {:?}: {}", entry.path(), e);
                            continue;
                        }
                    }
                } else {
                    // Skip non-XML files or malformed XML
                    println!("⚠️ Skipping non-XML file: {:?}", entry.path());
                    continue;
                }
            }
        }

        // Identificar tests flaky (fallan intermitentemente)
        self.identify_flaky_tests(&mut consolidated);

        Ok(consolidated)
    }

    fn identify_flaky_tests(&self, report: &mut ConsolidatedReport) {
        let mut test_failures: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for suite in &report.suites {
            for case in &suite.test_cases {
                if case.failures.is_some() {
                    let key = format!("{}::{}", case.classname, case.name);
                    *test_failures.entry(key.clone()).or_insert(0) += 1;
                }
            }
        }

        report.flaky_tests = test_failures
            .into_iter()
            .filter(|(_, failures)| *failures > 1)
            .map(|(test, _)| test)
            .collect();
    }

    pub fn save_consolidated_report(&self, report: &ConsolidatedReport) -> PulseResult<()> {
        let json = serde_json::to_string_pretty(report).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot serialize consolidated report: {}", e),
                None::<String>,
            )
        })?;
        let output_path = Path::new(&self.reports_dir).join("consolidated-report.json");
        let mut file = File::create(&output_path).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot create consolidated report file: {}", e),
                Some("Check write permissions for the reports directory"),
            )
        })?;
        file.write_all(json.as_bytes()).map_err(|e| {
            PulseError::fs_error(
                format!("Cannot write consolidated report: {}", e),
                Some("Check disk space and write permissions"),
            )
        })?;
        Ok(())
    }
}
