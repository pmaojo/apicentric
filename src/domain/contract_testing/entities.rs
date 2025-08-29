use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use super::config::CompatibilityPolicy;
use super::value_objects::{
    ComplianceIssueType, ComplianceSeverity, ContractId, ContractValidationError, HttpMethod,
    RequestBody, ResponseBody,
};

/// Validation scenario for testing specific API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationScenario {
    pub id: String,
    pub path: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub request_body: Option<RequestBody>,
    pub query_params: HashMap<String, String>,
}

impl ValidationScenario {
    pub fn new(id: String, path: String, method: HttpMethod) -> Self {
        Self {
            id,
            path,
            method,
            headers: HashMap::new(),
            request_body: None,
            query_params: HashMap::new(),
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_request_body(mut self, body: RequestBody) -> Self {
        self.request_body = Some(body);
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn request_body(&self) -> Option<&RequestBody> {
        self.request_body.as_ref()
    }
}

/// API response from either mock or real API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: ResponseBody,
    pub duration_ms: u64,
}

impl ApiResponse {
    pub fn new(
        status: u16,
        headers: HashMap<String, String>,
        body: ResponseBody,
        duration_ms: u64,
    ) -> Self {
        Self {
            status_code: status,
            headers,
            body,
            duration_ms,
        }
    }

    pub fn status(&self) -> u16 {
        self.status_code
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &ResponseBody {
        &self.body
    }

    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
}

/// Compliance issue detected during validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    pub issue_type: ComplianceIssueType,
    pub severity: ComplianceSeverity,
    pub description: String,
    pub scenario_path: String,
    pub details: Option<serde_json::Value>,
}

/// Result of validating a single scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioValidationResult {
    pub scenario_id: String,
    pub mock_response: Option<ApiResponse>,
    pub real_response: Option<ApiResponse>,
    pub compliance_issue: Option<ComplianceIssue>,
    pub duration_ms: u64,
}

/// Overall result of contract validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    pub contract_id: ContractId,
    pub validation_timestamp: SystemTime,
    pub compliance_score: f64,
    pub is_compatible: bool,
    pub issues: Vec<ComplianceIssue>,
    pub scenario_results: Vec<ScenarioValidationResult>,
    pub environment: String,
}

/// Domain events for contract testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractEvent {
    ContractRegistered {
        contract_id: ContractId,
        service_name: String,
        timestamp: SystemTime,
    },
    ContractDeleted {
        contract_id: ContractId,
        timestamp: SystemTime,
    },
    ValidationCompleted {
        contract_id: ContractId,
        validation_result: ContractValidationResult,
        timestamp: SystemTime,
    },
}

/// Main Contract aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: ContractId,
    pub service_name: String,
    pub spec_path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contract {
    pub fn new(
        id: ContractId,
        service_name: String,
        spec_path: String,
        description: Option<String>,
    ) -> Result<Self, ContractValidationError> {
        // Business rule validation
        if service_name.is_empty() {
            return Err(ContractValidationError::EmptyServiceName);
        }

        if spec_path.is_empty() {
            return Err(ContractValidationError::EmptySpecPath);
        }

        let now = Utc::now();

        Ok(Contract {
            id,
            service_name,
            spec_path,
            description,
            created_at: now,
            updated_at: now,
        })
    }

    // Getters
    pub fn id(&self) -> &ContractId {
        &self.id
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn spec_path(&self) -> &str {
        &self.spec_path
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Update contract details
    pub fn update(
        &mut self,
        service_name: Option<String>,
        spec_path: Option<String>,
        description: Option<String>,
    ) -> Result<(), ContractValidationError> {
        if let Some(name) = service_name {
            if name.is_empty() {
                return Err(ContractValidationError::EmptyServiceName);
            }
            self.service_name = name;
        }

        if let Some(path) = spec_path {
            if path.is_empty() {
                return Err(ContractValidationError::EmptySpecPath);
            }
            self.spec_path = path;
        }

        if let Some(desc) = description {
            self.description = Some(desc);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate compliance score based on validation issues
    pub fn calculate_compliance_score(&self, issues: &[ComplianceIssue]) -> f64 {
        if issues.is_empty() {
            return 1.0;
        }

        let total_weight = issues
            .iter()
            .map(|issue| match issue.severity {
                ComplianceSeverity::Low => 1.0,
                ComplianceSeverity::Medium => 3.0,
                ComplianceSeverity::High => 7.0,
                ComplianceSeverity::Critical => 10.0,
            })
            .sum::<f64>();

        let max_weight = issues.len() as f64 * 10.0; // All critical
        (max_weight - total_weight) / max_weight
    }

    /// Validate response compatibility between mock and real API
    pub fn validate_response_compatibility(
        &self,
        mock_response: &ApiResponse,
        real_response: &ApiResponse,
        policy: &CompatibilityPolicy,
    ) -> Option<ComplianceIssue> {
        // Check status code compatibility
        if policy.strict_status_codes && mock_response.status() != real_response.status() {
            let severity = if mock_response.status() / 100 != real_response.status() / 100 {
                ComplianceSeverity::High
            } else {
                ComplianceSeverity::Medium
            };

            return Some(ComplianceIssue {
                issue_type: ComplianceIssueType::StatusCodeMismatch,
                severity,
                description: format!(
                    "Status code mismatch: mock returned {}, real API returned {}",
                    mock_response.status(),
                    real_response.status()
                ),
                scenario_path: "".to_string(), // Will be filled by caller
                details: Some(serde_json::json!({
                    "mock_status": mock_response.status(),
                    "real_status": real_response.status()
                })),
            });
        }

        // Check response body compatibility (basic JSON structure comparison)
        if let (ResponseBody::Json(mock_json), ResponseBody::Json(real_json)) =
            (mock_response.body(), real_response.body())
        {
            if !self.json_structures_compatible(mock_json, real_json, policy) {
                return Some(ComplianceIssue {
                    issue_type: ComplianceIssueType::ResponseSchemaMismatch,
                    severity: ComplianceSeverity::Medium,
                    description: "Response JSON structure differs between mock and real API"
                        .to_string(),
                    scenario_path: "".to_string(),
                    details: Some(serde_json::json!({
                        "mock_structure": self.json_structure_summary(mock_json),
                        "real_structure": self.json_structure_summary(real_json)
                    })),
                });
            }
        }

        None
    }

    fn json_structures_compatible(
        &self,
        mock_json: &serde_json::Value,
        real_json: &serde_json::Value,
        policy: &CompatibilityPolicy,
    ) -> bool {
        use serde_json::Value;

        match (mock_json, real_json) {
            (Value::Object(mock_obj), Value::Object(real_obj)) => {
                // Check that all mock fields exist in real response
                for (key, mock_value) in mock_obj {
                    if let Some(real_value) = real_obj.get(key) {
                        if !self.json_structures_compatible(mock_value, real_value, policy) {
                            return false;
                        }
                    } else {
                        // Missing field in real response
                        return false;
                    }
                }

                // If policy is strict, check for additional fields
                if !policy.ignore_additional_fields {
                    for key in real_obj.keys() {
                        if !mock_obj.contains_key(key) {
                            return false;
                        }
                    }
                }

                true
            }
            (Value::Array(mock_arr), Value::Array(real_arr)) => {
                // For arrays, check that types are compatible
                if mock_arr.is_empty() || real_arr.is_empty() {
                    return true; // Empty arrays are compatible
                }

                // Check first element structure compatibility
                self.json_structures_compatible(&mock_arr[0], &real_arr[0], policy)
            }
            _ => {
                // For primitive types, check type compatibility
                std::mem::discriminant(mock_json) == std::mem::discriminant(real_json)
            }
        }
    }

    fn json_structure_summary(&self, json: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match json {
            Value::Object(obj) => {
                let mut summary = serde_json::Map::new();
                for (key, value) in obj {
                    summary.insert(key.clone(), self.json_structure_summary(value));
                }
                Value::Object(summary)
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    Value::String("array<empty>".to_string())
                } else {
                    Value::String(format!("array<{}>", self.json_type_name(&arr[0])))
                }
            }
            _ => Value::String(self.json_type_name(json).to_string()),
        }
    }

    fn json_type_name(&self, json: &serde_json::Value) -> &'static str {
        use serde_json::Value;

        match json {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}
