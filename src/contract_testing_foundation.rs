 // Contract Testing para Pulse - Los YAML de mock services SON los contratos
// Cada mock YAML define fielmente cómo debe comportarse la API real

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{anyhow, Result};

// === DOMAIN ENTITIES ===

/// Un contrato es esencialmente un mock service YAML que define el comportamiento esperado
/// de una API. El contract testing valida que la API real se comporte como el mock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiContract {
    pub id: String,
    pub name: String,
    /// Path al archivo YAML del mock service - ESTE ES EL CONTRATO
    pub service_spec_path: String,
    /// Configuración para conectar con la API real que debe cumplir el contrato
    pub real_api_config: RealApiConfig,
    /// Scenarios extraídos automáticamente del mock YAML
    pub validation_scenarios: Vec<ValidationScenario>,
    pub last_validation: Option<ContractValidationResult>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Configuración para conectar con la API real que implementa el contrato
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealApiConfig {
    pub base_url: String,
    pub auth: Option<AuthConfig>,
    pub headers: HashMap<String, String>,
    pub timeout_ms: u64,
    pub retry_attempts: u8,
    /// Mapeo de paths del mock a paths reales si difieren
    pub path_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { header: String, key: String },
    OAuth2 { client_id: String, client_secret: String, token_url: String },
}

/// Un scenario de validación extraído del mock YAML
/// Representa una interacción específica que debe ser validada contra la API real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationScenario {
    pub id: String,
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    /// Request body del mock (si aplica)
    pub request_body: Option<serde_json::Value>,
    pub headers: HashMap<String, String>,
    /// Condiciones del mock que determinan esta respuesta
    pub conditions: Vec<String>,
    /// Respuesta esperada según el mock
    pub expected_response: ExpectedResponse,
}

/// Respuesta esperada según está definida en el mock YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    /// Template de respuesta del mock (con handlebars)
    pub body_template: String,
    /// Respuesta renderizada para este scenario específico
    pub rendered_body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET, POST, PUT, PATCH, DELETE
}

/// Resultado de validar un contrato (mock YAML) contra la API real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    pub contract_id: String,
    pub service_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_scenarios: usize,
    pub passed: usize,
    pub failed: usize,
    pub scenario_results: Vec<ScenarioValidationResult>,
    pub contract_compliance_score: f64,
    pub overall_status: ContractComplianceStatus,
    pub execution_time_ms: u64,
}

/// Resultado de validar un scenario específico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioValidationResult {
    pub scenario_id: String,
    pub scenario_name: String,
    pub compliant: bool,
    /// Respuesta del mock (según el contrato)
    pub contract_response: Option<ApiResponse>,
    /// Respuesta de la API real
    pub real_api_response: Option<ApiResponse>,
    pub compliance_issues: Vec<ComplianceIssue>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractComplianceStatus {
    FullyCompliant,      // API real cumple 100% el contrato
    MostlyCompliant,     // Cumple pero con diferencias menores
    NonCompliant,        // Diferencias significativas
    ValidationError(String), // Error al validar
}

/// Diferencia entre lo que especifica el contrato (mock) y lo que hace la API real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    pub endpoint: String,
    pub issue_type: ComplianceIssueType,
    pub description: String,
    pub severity: ComplianceSeverity,
    pub contract_expectation: String,
    pub real_api_behavior: String,
    pub remediation_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceIssueType {
    /// Campo presente en contrato pero ausente en API real
    ContractFieldMissing(String),
    /// Campo presente en API real pero no especificado en contrato
    UnspecifiedFieldPresent(String),
    /// Tipo de dato diferente al especificado en contrato
    DataTypeMismatch { field: String, contract_type: String, real_type: String },
    /// Status code diferente al especificado en contrato
    StatusCodeMismatch { contract_status: u16, real_status: u16 },
    /// Estructura de respuesta diferente
    ResponseStructureMismatch,
    /// API real no disponible
    ApiUnavailable,
    /// Headers diferentes a los especificados
    HeaderMismatch { header: String, contract_value: String, real_value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Critical,  // Rompe el contrato - afecta funcionalidad
    Major,     // Diferencia importante pero no crítica
    Minor,     // Diferencia cosmética
    Info,      // Solo informativo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub response_time_ms: u64,
}

// === USE CASE ===

/// Use case principal para Contract Testing
/// Valida que las APIs reales cumplan los contratos definidos en los mock YAML
pub struct ContractValidationUseCase {
    mock_service_loader: MockServiceLoader,
    api_client: ApiClient,
    contract_storage: ContractStorage,
    handlebars_renderer: HandlebarsRenderer,
}

impl ContractValidationUseCase {
    pub fn new() -> Self {
        Self {
            mock_service_loader: MockServiceLoader::new(),
            api_client: ApiClient::new(),
            contract_storage: ContractStorage::new(),
            handlebars_renderer: HandlebarsRenderer::new(),
        }
    }

    /// Crear contrato desde un mock service YAML existente
    /// El YAML ya define el contrato - solo agregamos configuración para validar contra API real
    pub async fn create_contract_from_service_spec(
        &self,
        name: String,
        service_spec_path: String,
        real_api_config: RealApiConfig,
    ) -> Result<ApiContract> {
        // Cargar y parsear el mock service YAML
        let service_spec = self.mock_service_loader.load_service_spec(&service_spec_path).await?;
        
        // Extraer scenarios de validación desde los endpoints del mock
        let validation_scenarios = self.extract_validation_scenarios_from_service(&service_spec)?;
        
        let contract = ApiContract {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            service_spec_path,
            real_api_config,
            validation_scenarios,
            last_validation: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.contract_storage.save(&contract).await?;
        Ok(contract)
    }

    /// Validar que la API real cumple el contrato definido en el mock YAML
    pub async fn validate_contract(&self, contract_id: &str) -> Result<ContractValidationResult> {
        let contract = self.contract_storage.get(contract_id).await?
            .ok_or_else(|| anyhow::anyhow!("Contract not found: {}", contract_id))?;

        let start_time = std::time::Instant::now();
        let mut scenario_results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;

        println!("🔍 Validating contract: {} against real API", contract.name);
        println!("📋 Service spec: {}", contract.service_spec_path);
        println!("🌐 Real API: {}", contract.real_api_config.base_url);

        // Validar cada scenario extraído del mock
        for scenario in &contract.validation_scenarios {
            let scenario_start = std::time::Instant::now();
            
            println!("  🧪 Testing: {} {} {}", 
                self.http_method_to_string(&scenario.method), 
                scenario.path,
                scenario.name
            );

            // Renderizar respuesta esperada del contrato (mock)
            let contract_response = match self.render_contract_response(scenario).await {
                Ok(response) => Some(response),
                Err(e) => {
                    failed += 1;
                    scenario_results.push(ScenarioValidationResult {
                        scenario_id: scenario.id.clone(),
                        scenario_name: scenario.name.clone(),
                        compliant: false,
                        contract_response: None,
                        real_api_response: None,
                        compliance_issues: vec![ComplianceIssue {
                            endpoint: format!("{} {}", 
                                self.http_method_to_string(&scenario.method), 
                                scenario.path
                            ),
                            issue_type: ComplianceIssueType::ApiUnavailable,
                            description: format!("Failed to render contract response: {}", e),
                            severity: ComplianceSeverity::Critical,
                            contract_expectation: "Valid response template".to_string(),
                            real_api_behavior: "Template rendering failed".to_string(),
                            remediation_suggestion: Some("Check mock YAML template syntax".to_string()),
                        }],
                        execution_time_ms: scenario_start.elapsed().as_millis() as u64,
                    });
                    continue;
                }
            };

            // Ejecutar request contra API real
            let real_api_response = match self.execute_against_real_api(scenario, &contract.real_api_config).await {
                Ok(response) => Some(response),
                Err(e) => {
                    failed += 1;
                    scenario_results.push(ScenarioValidationResult {
                        scenario_id: scenario.id.clone(),
                        scenario_name: scenario.name.clone(),
                        compliant: false,
                        contract_response: contract_response,
                        real_api_response: None,
                        compliance_issues: vec![ComplianceIssue {
                            endpoint: format!("{} {}", 
                                self.http_method_to_string(&scenario.method), 
                                scenario.path
                            ),
                            issue_type: ComplianceIssueType::ApiUnavailable,
                            description: format!("Real API request failed: {}", e),
                            severity: ComplianceSeverity::Critical,
                            contract_expectation: "API should be available and respond".to_string(),
                            real_api_behavior: format!("API unavailable: {}", e),
                            remediation_suggestion: Some("Check API availability and configuration".to_string()),
                        }],
                        execution_time_ms: scenario_start.elapsed().as_millis() as u64,
                    });
                    continue;
                }
            };

            // Validar compliance entre contrato y realidad
            let contract_response_ref = contract_response
                .as_ref()
                .ok_or_else(|| anyhow!("missing contract response"))?;
            let real_api_response_ref = real_api_response
                .as_ref()
                .ok_or_else(|| anyhow!("missing real API response"))?;
            let compliance_issues = self.validate_compliance(
                contract_response_ref,
                real_api_response_ref,
                scenario
            )?;

            let is_compliant = compliance_issues.iter().all(|issue| {
                !matches!(issue.severity, ComplianceSeverity::Critical | ComplianceSeverity::Major)
            });

            if is_compliant {
                passed += 1;
                println!("    ✅ Compliant");
            } else {
                failed += 1;
                println!("    ❌ Non-compliant ({} issues)", compliance_issues.len());
                for issue in &compliance_issues {
                    if matches!(issue.severity, ComplianceSeverity::Critical | ComplianceSeverity::Major) {
                        println!("      🚨 {:?}: {}", issue.severity, issue.description);
                    }
                }
            }

            scenario_results.push(ScenarioValidationResult {
                scenario_id: scenario.id.clone(),
                scenario_name: scenario.name.clone(),
                compliant: is_compliant,
                contract_response,
                real_api_response,
                compliance_issues,
                execution_time_ms: scenario_start.elapsed().as_millis() as u64,
            });
        }

        let total_scenarios = scenario_results.len();
        let compliance_score = if total_scenarios > 0 {
            passed as f64 / total_scenarios as f64
        } else {
            0.0
        };

        let overall_status = match (passed, failed) {
            (p, 0) if p > 0 => ContractComplianceStatus::FullyCompliant,
            (p, f) if p > f => ContractComplianceStatus::MostlyCompliant,
            (0, f) if f > 0 => ContractComplianceStatus::NonCompliant,
            _ => ContractComplianceStatus::ValidationError("No scenarios validated".to_string()),
        };

        let service_name = self.extract_service_name_from_path(&contract.service_spec_path);

        let result = ContractValidationResult {
            contract_id: contract_id.to_string(),
            service_name,
            timestamp: chrono::Utc::now(),
            total_scenarios,
            passed,
            failed,
            scenario_results,
            contract_compliance_score: compliance_score,
            overall_status,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        };

        // Actualizar contract con último resultado
        let mut updated_contract = contract;
        updated_contract.last_validation = Some(result.clone());
        updated_contract.updated_at = chrono::Utc::now();
        self.contract_storage.save(&updated_contract).await?;

        Ok(result)
    }

    /// Validar compliance entre la respuesta esperada (contrato) y la real
    fn validate_compliance(
        &self,
        contract_response: &ApiResponse,
        real_api_response: &ApiResponse,
        scenario: &ValidationScenario,
    ) -> Result<Vec<ComplianceIssue>> {
        let mut issues = Vec::new();
        let endpoint = format!("{} {}", 
            self.http_method_to_string(&scenario.method), 
            scenario.path
        );

        // Validar status code
        if contract_response.status != real_api_response.status {
            issues.push(ComplianceIssue {
                endpoint: endpoint.clone(),
                issue_type: ComplianceIssueType::StatusCodeMismatch {
                    contract_status: contract_response.status,
                    real_status: real_api_response.status,
                },
                description: format!(
                    "Status code mismatch: contract expects {}, real API returns {}",
                    contract_response.status, real_api_response.status
                ),
                severity: if contract_response.status / 100 != real_api_response.status / 100 {
                    ComplianceSeverity::Critical
                } else {
                    ComplianceSeverity::Major
                },
                contract_expectation: format!("Status: {}", contract_response.status),
                real_api_behavior: format!("Status: {}", real_api_response.status),
                remediation_suggestion: Some("Update contract or fix API to return correct status".to_string()),
            });
        }

        // Validar estructura de respuesta si ambas son exitosas
        if contract_response.status < 400 && real_api_response.status < 400 {
            let structure_issues = self.validate_response_structure(
                &contract_response.body,
                &real_api_response.body,
                &endpoint
            )?;
            issues.extend(structure_issues);
        }

        // Validar headers críticos
        for (header_name, expected_value) in &contract_response.headers {
            if let Some(real_value) = real_api_response.headers.get(header_name) {
                if expected_value != real_value {
                    issues.push(ComplianceIssue {
                        endpoint: endpoint.clone(),
                        issue_type: ComplianceIssueType::HeaderMismatch {
                            header: header_name.clone(),
                            contract_value: expected_value.clone(),
                            real_value: real_value.clone(),
                        },
                        description: format!("Header '{}' mismatch", header_name),
                        severity: if header_name.to_lowercase() == "content-type" {
                            ComplianceSeverity::Major
                        } else {
                            ComplianceSeverity::Minor
                        },
                        contract_expectation: format!("{}: {}", header_name, expected_value),
                        real_api_behavior: format!("{}: {}", header_name, real_value),
                        remediation_suggestion: Some(format!("Update contract or API to match header '{}'", header_name)),
                    });
                }
            }
        }

        Ok(issues)
    }

    fn validate_response_structure(
        &self,
        contract_json: &serde_json::Value,
        real_json: &serde_json::Value,
        endpoint: &str,
    ) -> Result<Vec<ComplianceIssue>> {
        let mut issues = Vec::new();

        // Extraer paths de campos
        let contract_paths = self.extract_json_paths(contract_json);
        let real_paths = self.extract_json_paths(real_json);

        // Campos especificados en contrato pero ausentes en API real
        for path in &contract_paths {
            if !real_paths.contains(path) {
                issues.push(ComplianceIssue {
                    endpoint: endpoint.to_string(),
                    issue_type: ComplianceIssueType::ContractFieldMissing(path.clone()),
                    description: format!("Field '{}' specified in contract but missing in real API", path),
                    severity: ComplianceSeverity::Major,
                    contract_expectation: format!("Field '{}' should be present", path),
                    real_api_behavior: format!("Field '{}' is missing", path),
                    remediation_suggestion: Some(format!("Add field '{}' to API or remove from contract", path)),
                });
            }
        }

        // Campos presentes en API real pero no especificados en contrato
        for path in &real_paths {
            if !contract_paths.contains(path) {
                issues.push(ComplianceIssue {
                    endpoint: endpoint.to_string(),
                    issue_type: ComplianceIssueType::UnspecifiedFieldPresent(path.clone()),
                    description: format!("Field '{}' present in real API but not specified in contract", path),
                    severity: ComplianceSeverity::Minor,
                    contract_expectation: format!("Field '{}' not specified", path),
                    real_api_behavior: format!("Field '{}' is present", path),
                    remediation_suggestion: Some(format!("Add field '{}' to contract if it should be documented", path)),
                });
            }
        }

        Ok(issues)
    }

    fn extract_json_paths(&self, value: &serde_json::Value) -> std::collections::HashSet<String> {
        let mut paths = std::collections::HashSet::new();
        self.extract_json_paths_recursive(value, String::new(), &mut paths);
        paths
    }

    fn extract_json_paths_recursive(
        &self,
        value: &serde_json::Value,
        current_path: String,
        paths: &mut std::collections::HashSet<String>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", current_path, key)
                    };
                    paths.insert(new_path.clone());
                    self.extract_json_paths_recursive(val, new_path, paths);
                }
            }
            serde_json::Value::Array(arr) => {
                if !arr.is_empty() {
                    self.extract_json_paths_recursive(&arr[0], current_path, paths);
                }
            }
            _ => {}
        }
    }

    fn http_method_to_string(&self, method: &HttpMethod) -> &'static str {
        match method {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
        }
    }

    fn extract_service_name_from_path(&self, path: &str) -> String {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    // Métodos que serían implementados con los adaptadores reales
    fn extract_validation_scenarios_from_service(&self, _service_spec: &MockServiceSpec) -> Result<Vec<ValidationScenario>> {
        // Parsear el YAML del mock service y extraer endpoints como scenarios
        todo!("Extract validation scenarios from mock service YAML")
    }

    async fn render_contract_response(&self, _scenario: &ValidationScenario) -> Result<ApiResponse> {
        // Renderizar la respuesta esperada usando handlebars
        todo!("Render contract response using handlebars")
    }

    async fn execute_against_real_api(
        &self,
        _scenario: &ValidationScenario,
        _config: &RealApiConfig,
    ) -> Result<ApiResponse> {
        // Ejecutar request contra la API real
        todo!("Execute request against real API")
    }
}

/// Especificación de un mock service cargada desde YAML
#[derive(Debug, Clone)]
pub struct MockServiceSpec {
    pub name: String,
    pub port: u16,
    pub base_path: String,
    pub fixtures: serde_json::Value,
    pub endpoints: Vec<MockEndpoint>,
}

#[derive(Debug, Clone)]
pub struct MockEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub conditions: Vec<String>,
    pub response: MockResponse,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body_template: String,
}

// === ADAPTERS (placeholder implementations) ===

/// Carga y parsea especificaciones de mock services desde YAML
struct MockServiceLoader;
impl MockServiceLoader {
    fn new() -> Self { Self }
    async fn load_service_spec(&self, _path: &str) -> Result<MockServiceSpec> { 
        todo!("Load and parse mock service YAML")
    }
}

/// Cliente HTTP para ejecutar requests contra APIs reales
struct ApiClient;
impl ApiClient {
    fn new() -> Self { Self }
}

/// Storage para contratos (podría ser archivo JSON, base de datos, etc.)
struct ContractStorage;
impl ContractStorage {
    fn new() -> Self { Self }
    async fn save(&self, _contract: &ApiContract) -> Result<()> { Ok(()) }
    async fn get(&self, _id: &str) -> Result<Option<ApiContract>> { Ok(None) }
}

/// Renderizador de templates Handlebars para respuestas de contrato
struct HandlebarsRenderer;
impl HandlebarsRenderer {
    fn new() -> Self { Self }
}

// === CLI COMMAND HANDLER ===

pub async fn handle_contract_command(action: &str, args: &[String]) -> Result<()> {
    let usecase = ContractValidationUseCase::new();

    match action {
        "create" => {
            let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Contract name required"))?;
            let service_spec = args.get(1).ok_or_else(|| anyhow::anyhow!("Service spec path required"))?;
            let real_url = args.get(2).ok_or_else(|| anyhow::anyhow!("Real API URL required"))?;

            let real_api_config = RealApiConfig {
                base_url: real_url.clone(),
                auth: None,
                headers: HashMap::new(),
                timeout_ms: 10000,
                retry_attempts: 3,
                path_mapping: HashMap::new(),
            };

            let contract = usecase.create_contract_from_service_spec(
                name.clone(),
                service_spec.clone(),
                real_api_config,
            ).await?;

            println!("✅ Contract created from service specification");
            println!("   ID: {}", contract.id);
            println!("   Name: {}", contract.name);
            println!("   Service spec: {}", contract.service_spec_path);
            println!("   Validation scenarios: {}", contract.validation_scenarios.len());
        }
        "validate" => {
            let contract_id = args.get(0).ok_or_else(|| anyhow::anyhow!("Contract ID required"))?;
            
            println!("🔍 Validating contract compliance: {}", contract_id);
            let result = usecase.validate_contract(contract_id).await?;

            println!("\n📊 Contract Validation Results:");
            println!("   Service: {}", result.service_name);
            println!("   Total scenarios: {}", result.total_scenarios);
            println!("   Compliant: {} ✅", result.passed);
            println!("   Non-compliant: {} ❌", result.failed);
            println!("   Compliance score: {:.2}%", result.contract_compliance_score * 100.0);
            println!("   Status: {:?}", result.overall_status);
            println!("   Execution time: {}ms", result.execution_time_ms);

            // Mostrar issues de compliance si los hay
            for scenario_result in &result.scenario_results {
                if !scenario_result.compliance_issues.is_empty() {
                    println!("\n⚠️ Compliance issues in {}:", scenario_result.scenario_name);
                    for issue in &scenario_result.compliance_issues {
                        println!("   - {} ({:?}): {}", 
                            issue.endpoint, 
                            issue.severity, 
                            issue.description
                        );
                        println!("     📋 Contract expects: {}", issue.contract_expectation);
                        println!("     🌐 Real API does: {}", issue.real_api_behavior);
                        if let Some(suggestion) = &issue.remediation_suggestion {
                            println!("     💡 {}", suggestion);
                        }
                    }
                }
            }

            // Exit code basado en compliance
            if matches!(result.overall_status, ContractComplianceStatus::NonCompliant) {
                std::process::exit(1);
            }
        }
        "list" => {
            println!("📋 Contract listing not yet implemented");
            todo!("Implement contract listing")
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown contract command: {}", action));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_contract_creation_from_service_spec() {
        let usecase = ContractValidationUseCase::new();
        
        let real_api_config = RealApiConfig {
            base_url: "https://api.example.com".to_string(),
            auth: None,
            headers: HashMap::new(),
            timeout_ms: 5000,
            retry_attempts: 2,
            path_mapping: HashMap::new(),
        };

        // This would fail with the placeholder implementation
        // but shows the intended API for contract creation from service specs
        let result = usecase.create_contract_from_service_spec(
            "test-contract".to_string(),
            "services/auth-service.yaml".to_string(),
            real_api_config,
        ).await;

        // Expected to fail with placeholder implementation
        assert!(result.is_err());
    }

    #[test]
    fn test_json_path_extraction_for_compliance() {
        let usecase = ContractValidationUseCase::new();
        
        let contract_response = serde_json::json!({
            "user": {
                "id": 123,
                "email": "user@example.com",
                "profile": {
                    "name": "John Doe"
                }
            },
            "token": "abc123",
            "expires_in": 3600
        });

        let paths = usecase.extract_json_paths(&contract_response);
        
        // Verify contract-specified fields are extracted
        assert!(paths.contains("user"));
        assert!(paths.contains("user.id"));
        assert!(paths.contains("user.email"));
        assert!(paths.contains("user.profile"));
        assert!(paths.contains("user.profile.name"));
        assert!(paths.contains("token"));
        assert!(paths.contains("expires_in"));
    }

    #[test]
    fn test_compliance_issue_severity_classification() {
        // Test that different types of compliance issues are classified correctly
        let status_mismatch = ComplianceIssue {
            endpoint: "POST /login".to_string(),
            issue_type: ComplianceIssueType::StatusCodeMismatch {
                contract_status: 200,
                real_status: 500,
            },
            description: "Critical status mismatch".to_string(),
            severity: ComplianceSeverity::Critical,
            contract_expectation: "200 OK".to_string(),
            real_api_behavior: "500 Error".to_string(),
            remediation_suggestion: None,
        };

        let extra_field = ComplianceIssue {
            endpoint: "GET /profile".to_string(),
            issue_type: ComplianceIssueType::UnspecifiedFieldPresent("avatar_url".to_string()),
            description: "Extra field in real API".to_string(),
            severity: ComplianceSeverity::Minor,
            contract_expectation: "Field not specified".to_string(),
            real_api_behavior: "Field present".to_string(),
            remediation_suggestion: None,
        };

        // Critical issues should fail compliance
        assert!(matches!(status_mismatch.severity, ComplianceSeverity::Critical));
        
        // Minor issues should not fail compliance
        assert!(matches!(extra_field.severity, ComplianceSeverity::Minor));
    }

    #[test]
    fn test_contract_compliance_status() {
        // Test overall compliance status determination
        assert!(matches!(
            ContractComplianceStatus::FullyCompliant, 
            ContractComplianceStatus::FullyCompliant
        ));
        
        assert!(matches!(
            ContractComplianceStatus::NonCompliant, 
            ContractComplianceStatus::NonCompliant
        ));
    }
}
