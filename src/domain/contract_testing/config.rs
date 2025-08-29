use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::entities::ComplianceIssue;
use super::value_objects::{ApiUrl, ComplianceSeverity, RetryAttempts, TimeoutDuration};

/// Real API configuration for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealApiConfig {
    pub environment: String,
    pub base_url: ApiUrl,
    pub headers: HashMap<String, String>,
    pub auth_header: Option<String>,
    pub timeout: Option<TimeoutDuration>,
    pub retry_attempts: RetryAttempts,
}

impl RealApiConfig {
    pub fn new(
        environment: String,
        base_url: ApiUrl,
        timeout: Option<TimeoutDuration>,
        retry_attempts: RetryAttempts,
    ) -> Self {
        Self {
            environment,
            base_url,
            headers: HashMap::new(),
            auth_header: None,
            timeout,
            retry_attempts,
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_auth_header(mut self, auth_header: String) -> Self {
        self.auth_header = Some(auth_header);
        self
    }

    pub fn timeout(&self) -> Option<&TimeoutDuration> {
        self.timeout.as_ref()
    }

    pub fn retry_attempts(&self) -> &RetryAttempts {
        &self.retry_attempts
    }
}

/// Compatibility policy for determining contract compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityPolicy {
    pub name: String,
    pub strict_status_codes: bool,
    pub ignore_additional_fields: bool,
    pub tolerance_threshold: f64,
}

impl CompatibilityPolicy {
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            strict_status_codes: true,
            ignore_additional_fields: false,
            tolerance_threshold: 0.95,
        }
    }

    pub fn moderate() -> Self {
        Self {
            name: "moderate".to_string(),
            strict_status_codes: true,
            ignore_additional_fields: true,
            tolerance_threshold: 0.80,
        }
    }

    pub fn lenient() -> Self {
        Self {
            name: "lenient".to_string(),
            strict_status_codes: false,
            ignore_additional_fields: true,
            tolerance_threshold: 0.60,
        }
    }

    pub fn is_compatible(&self, issues: &[ComplianceIssue]) -> bool {
        let critical_issues = issues
            .iter()
            .filter(|issue| issue.severity == ComplianceSeverity::Critical)
            .count();

        if critical_issues > 0 {
            return false;
        }

        let total_issues = issues.len();
        if total_issues == 0 {
            return true;
        }

        // Calculate compliance based on issue severity
        let weighted_score = issues
            .iter()
            .map(|issue| match issue.severity {
                ComplianceSeverity::Low => 0.1,
                ComplianceSeverity::Medium => 0.3,
                ComplianceSeverity::High => 0.7,
                ComplianceSeverity::Critical => 1.0,
            })
            .sum::<f64>();

        let max_possible_score = total_issues as f64;
        let compliance_score = 1.0 - (weighted_score / max_possible_score);

        compliance_score >= self.tolerance_threshold
    }
}
