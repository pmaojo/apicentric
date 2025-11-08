// Service Definition Loader Adapter - Infrastructure layer implementation
// Implements ServiceSpecLoader port for loading and parsing YAML service definitions

use crate::domain::contract_testing::*;
use crate::domain::ports::contract::{EndpointSpec, ResponseSpec, ServiceSpec, ServiceSpecLoader, SpecLoaderError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use crate::utils::{FileReader, TokioFileReader};
use tracing::{debug, info};

/// YAML-based implementation of ServiceSpecLoader
/// Loads service definitions from YAML files
pub struct YamlServiceSpecLoader {
    base_path: Option<String>,
    file_reader: Arc<dyn FileReader>,
}

impl YamlServiceSpecLoader {
    pub fn new() -> Self {
        Self { base_path: None, file_reader: Arc::new(TokioFileReader) }
    }

    pub fn with_base_path<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: Some(base_path.as_ref().to_string_lossy().to_string()),
            file_reader: Arc::new(TokioFileReader),
        }
    }

    pub fn with_file_reader(reader: Arc<dyn FileReader>) -> Self {
        Self { base_path: None, file_reader: reader }
    }

    pub fn with_base_path_and_reader<P: AsRef<Path>>(base_path: P, reader: Arc<dyn FileReader>) -> Self {
        Self {
            base_path: Some(base_path.as_ref().to_string_lossy().to_string()),
            file_reader: reader,
        }
    }

    fn resolve_path(&self, path: &str) -> String {
        if let Some(base) = &self.base_path {
            if Path::new(path).is_relative() {
                return format!("{}/{}", base, path);
            }
        }
        path.to_string()
    }

    fn extract_validation_scenarios_from_spec(
        &self,
        spec: &ServiceSpec,
    ) -> Result<Vec<ValidationScenario>, SpecLoaderError> {
        let mut scenarios = Vec::new();

        for (endpoint_index, endpoint) in spec.endpoints.iter().enumerate() {
            // Create base scenario from endpoint
            let scenario_id = format!(
                "{}_{}_endpoint_{}",
                spec.name,
                endpoint.method.to_string().to_lowercase(),
                endpoint_index
            );

            let mut headers = HashMap::new();

            // Add common headers
            headers.insert("Accept".to_string(), "application/json".to_string());
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            // Extract headers from response spec if available
            for (key, value) in &endpoint.response.headers {
                if key.to_lowercase().starts_with("x-")
                    || key.to_lowercase() == "authorization"
                    || key.to_lowercase() == "user-agent"
                {
                    headers.insert(key.clone(), value.clone());
                }
            }

            // Create basic validation scenario
            let scenario = ValidationScenario {
                id: scenario_id,
                path: endpoint.path.clone(),
                method: endpoint.method.clone(),
                headers,
                query_params: HashMap::new(),
                request_body: None,
            };

            scenarios.push(scenario);

            // Generate additional scenarios based on conditions
            for (condition_index, condition) in endpoint.conditions.iter().enumerate() {
                if let Ok(additional_scenario) =
                    self.parse_condition_scenario(spec, endpoint, condition, condition_index)
                {
                    scenarios.push(additional_scenario);
                }
            }
        }

        debug!(
            "Extracted {} validation scenarios from spec: {}",
            scenarios.len(),
            spec.name
        );
        Ok(scenarios)
    }

    fn parse_condition_scenario(
        &self,
        spec: &ServiceSpec,
        endpoint: &EndpointSpec,
        condition: &str,
        index: usize,
    ) -> Result<ValidationScenario, SpecLoaderError> {
        let scenario_id = format!(
            "{}_{}_condition_{}",
            spec.name,
            endpoint.method.to_string().to_lowercase(),
            index
        );

        let mut headers = HashMap::new();
        let mut query_params = HashMap::new();
        let mut request_body = None;

        // Parse condition string for additional parameters
        // Expected format: "param=value&header=X-Custom:value&body={...}"
        if condition.contains('&') {
            for part in condition.split('&') {
                if let Some((key, value)) = part.split_once('=') {
                    match key {
                        "header" => {
                            if let Some((header_name, header_value)) = value.split_once(':') {
                                headers.insert(header_name.to_string(), header_value.to_string());
                            }
                        }
                        "query" => {
                            if let Some((param_name, param_value)) = value.split_once(':') {
                                query_params
                                    .insert(param_name.to_string(), param_value.to_string());
                            }
                        }
                        "body" => {
                            if value.starts_with('{') && value.ends_with('}') {
                                match serde_json::from_str::<serde_json::Value>(value) {
                                    Ok(json) => request_body = Some(RequestBody::Json(json)),
                                    Err(_) => {
                                        request_body = Some(RequestBody::Text(value.to_string()))
                                    }
                                }
                            } else {
                                request_body = Some(RequestBody::Text(value.to_string()));
                            }
                        }
                        _ => {
                            query_params.insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }

        // Add default headers
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(ValidationScenario {
            id: scenario_id,
            path: endpoint.path.clone(),
            method: endpoint.method.clone(),
            headers,
            query_params,
            request_body,
        })
    }
}

#[async_trait]
impl ServiceSpecLoader for YamlServiceSpecLoader {
    async fn load(&self, path: &str) -> Result<ServiceSpec, SpecLoaderError> {
        let resolved_path = self.resolve_path(path);
        debug!("Loading service spec from: {}", resolved_path);

        // Read YAML file
        let content = self.file_reader.read_to_string(Path::new(&resolved_path)).await.map_err(|e| {
            SpecLoaderError::FileNotFound(format!("Failed to read {}: {}", resolved_path, e))
        })?;

        // Parse YAML
        let yaml_spec: YamlServiceSpec = serde_yaml::from_str(&content).map_err(|e| {
            SpecLoaderError::InvalidYaml(format!(
                "Failed to parse YAML from {}: {}",
                resolved_path, e
            ))
        })?;

        // Convert to domain ServiceSpec
        let service_spec = self.convert_yaml_to_service_spec(yaml_spec)?;

        info!(
            "Successfully loaded service spec: {} with {} endpoints",
            service_spec.name,
            service_spec.endpoints.len()
        );

        Ok(service_spec)
    }

    async fn validate(&self, spec: &ServiceSpec) -> Result<(), SpecLoaderError> {
        debug!("Validating service spec: {}", spec.name);

        // Validate required fields
        if spec.name.is_empty() {
            return Err(SpecLoaderError::MissingField("name".to_string()));
        }

        if spec.port == 0 {
            return Err(SpecLoaderError::ValidationError(
                "port must be greater than 0".to_string(),
            ));
        }

        if spec.endpoints.is_empty() {
            return Err(SpecLoaderError::ValidationError(
                "spec must have at least one endpoint".to_string(),
            ));
        }

        // Validate endpoints
        for (index, endpoint) in spec.endpoints.iter().enumerate() {
            if endpoint.path.is_empty() {
                return Err(SpecLoaderError::MissingField(format!(
                    "endpoint[{}].path",
                    index
                )));
            }

            if !endpoint.path.starts_with('/') {
                return Err(SpecLoaderError::ValidationError(format!(
                    "endpoint[{}].path must start with '/'",
                    index
                )));
            }

            // Validate response spec
            if endpoint.response.status == 0 {
                return Err(SpecLoaderError::ValidationError(format!(
                    "endpoint[{}].response.status must be a valid HTTP status code",
                    index
                )));
            }
        }

        // Validate fixtures is valid JSON
        if !spec.fixtures.is_object() && !spec.fixtures.is_array() {
            return Err(SpecLoaderError::ValidationError(
                "fixtures must be a valid JSON object or array".to_string(),
            ));
        }

        debug!("Service spec validation successful: {}", spec.name);
        Ok(())
    }

    fn extract_scenarios(
        &self,
        spec: &ServiceSpec,
    ) -> Result<Vec<ValidationScenario>, SpecLoaderError> {
        self.extract_validation_scenarios_from_spec(spec)
    }
}

impl YamlServiceSpecLoader {
    fn convert_yaml_to_service_spec(
        &self,
        yaml_spec: YamlServiceSpec,
    ) -> Result<ServiceSpec, SpecLoaderError> {
        let mut endpoints = Vec::new();

        // Convert YAML endpoints to domain endpoints
        for yaml_endpoint in yaml_spec.endpoints {
            let method = self.parse_http_method(&yaml_endpoint.method)?;

            let endpoint = EndpointSpec {
                path: yaml_endpoint.path,
                method,
                conditions: yaml_endpoint.conditions.unwrap_or_default(),
                response: ResponseSpec {
                    status: yaml_endpoint.response.status,
                    headers: yaml_endpoint.response.headers.unwrap_or_default(),
                    body_template: yaml_endpoint.response.body.unwrap_or_default(),
                },
            };

            endpoints.push(endpoint);
        }

        // Parse fixtures
        let fixtures = if let Some(fixtures_yaml) = yaml_spec.fixtures {
            serde_json::to_value(fixtures_yaml).map_err(|e| {
                SpecLoaderError::ValidationError(format!(
                    "Failed to convert fixtures to JSON: {}",
                    e
                ))
            })?
        } else {
            serde_json::json!({})
        };

        Ok(ServiceSpec {
            name: yaml_spec.name,
            port: yaml_spec.port,
            base_path: yaml_spec.base_path.unwrap_or_default(),
            fixtures,
            endpoints,
        })
    }

    fn parse_http_method(&self, method_str: &str) -> Result<HttpMethod, SpecLoaderError> {
        match method_str.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(SpecLoaderError::ValidationError(format!(
                "Unsupported HTTP method: {}",
                method_str
            ))),
        }
    }
}

impl Default for YamlServiceSpecLoader {
    fn default() -> Self {
        Self::new()
    }
}

// === YAML PARSING STRUCTURES ===

#[derive(Debug, Deserialize, Serialize)]
struct YamlServiceSpec {
    name: String,
    port: u16,
    #[serde(rename = "basePath")]
    base_path: Option<String>,
    fixtures: Option<serde_yaml::Value>,
    endpoints: Vec<YamlEndpoint>,
}

#[derive(Debug, Deserialize, Serialize)]
struct YamlEndpoint {
    path: String,
    method: String,
    conditions: Option<Vec<String>>,
    response: YamlResponse,
}

#[derive(Debug, Deserialize, Serialize)]
struct YamlResponse {
    status: u16,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

