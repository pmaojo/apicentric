pub mod allure;
pub mod prometheus;
pub mod sentry;

pub use allure::AllureAdapter;
pub use prometheus::PrometheusAdapter;
pub use sentry::SentryAdapter;

use crate::PulseResult;
use std::time::Duration;

/// Unified metrics manager that coordinates all metrics adapters
pub struct MetricsManager {
    pub allure: Option<AllureAdapter>,
    pub prometheus: Option<PrometheusAdapter>,
    pub sentry: Option<SentryAdapter>,
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            allure: None,
            prometheus: None,
            sentry: None,
        }
    }

    pub fn with_allure(mut self, adapter: AllureAdapter) -> Self {
        self.allure = Some(adapter);
        self
    }

    pub fn with_prometheus(mut self, adapter: PrometheusAdapter) -> Self {
        self.prometheus = Some(adapter);
        self
    }

    pub fn with_sentry(mut self, adapter: SentryAdapter) -> Self {
        self.sentry = Some(adapter);
        self
    }

    pub fn start_test(&mut self, name: &str, full_name: &str) {
        if let Some(ref mut allure) = self.allure {
            allure.start_test(name, full_name);
        }

        if let Some(ref sentry) = self.sentry {
            sentry.add_breadcrumb(&format!("Starting test: {}", name), "test");
        }
    }

    pub fn end_test(
        &mut self,
        name: &str,
        passed: bool,
        error: Option<String>,
        duration: Duration,
    ) -> PulseResult<()> {
        // Record in Allure
        if let Some(ref mut allure) = self.allure {
            let status = if passed {
                allure::AllureStatus::Passed
            } else {
                allure::AllureStatus::Failed
            };
            allure.end_test(status, error.clone(), duration)?;
        }

        // Record in Prometheus
        if let Some(ref prometheus) = self.prometheus {
            prometheus.record_test_execution(duration, passed);
        }

        // Report to Sentry if failed
        if !passed {
            if let Some(ref sentry) = self.sentry {
                let error_msg =
                    error.unwrap_or_else(|| "Test failed without specific error".to_string());
                sentry.report_test_failure(name, &error_msg, duration);
            }
        }

        Ok(())
    }

    pub fn record_test_suite_completion(
        &self,
        total_tests: usize,
        failed_tests: usize,
        duration: Duration,
    ) {
        if let Some(ref prometheus) = self.prometheus {
            prometheus.record_test_suite_execution();
        }

        if let Some(ref sentry) = self.sentry {
            sentry.report_test_suite_completion(total_tests, failed_tests, duration);
        }
    }

    pub fn report_flaky_tests(&self, flaky_tests: &[String]) {
        if let Some(ref prometheus) = self.prometheus {
            prometheus.update_flaky_tests(flaky_tests.len() as i64);
        }

        if let Some(ref sentry) = self.sentry {
            for test in flaky_tests {
                sentry.report_flaky_test(test, 2, 3); // Assuming 2 failures out of 3 runs for flaky detection
            }
        }
    }

    pub fn generate_reports(&mut self) -> PulseResult<()> {
        if let Some(ref allure) = self.allure {
            allure.generate_environment_info()?;
        }

        if let Some(ref sentry) = self.sentry {
            sentry.flush()?;
        }

        Ok(())
    }

    pub fn get_summary(&self) -> String {
        let mut summary = String::new();

        if let Some(ref prometheus) = self.prometheus {
            summary.push_str(&format!("📊 {}\n", prometheus.get_metrics_summary()));
        }

        if self.allure.is_some() {
            summary.push_str("📋 Allure reports generated\n");
        }

        if self.sentry.is_some() {
            summary.push_str("🔍 Sentry monitoring active\n");
        }

        summary
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}
