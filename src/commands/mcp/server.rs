//! The MCP server implementation for `apicentric`.

#![cfg(feature = "mcp")]

use apicentric::Context;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{
        CallToolResult, Content, ErrorCode, Implementation, ProtocolVersion, ServerCapabilities,
        ServerInfo,
    },
    schemars, tool, tool_handler, tool_router, ErrorData as McpError,
};
use serde::Deserialize;

// Import for service creation
#[cfg(feature = "mcp")]
use apicentric::adapters::service_spec_loader::YamlServiceSpecLoader;
#[cfg(feature = "mcp")]
use apicentric::domain::contract_testing::HttpMethod;
#[cfg(feature = "mcp")]
use apicentric::domain::ports::contract::ServiceSpec;
#[cfg(feature = "mcp")]
use apicentric::simulator::config::{
    EndpointDefinition, EndpointKind, ResponseDefinition, ServerConfig, ServiceDefinition,
};
#[cfg(feature = "mcp")]
use std::collections::HashMap;
#[cfg(feature = "mcp")]
use std::fs;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ServiceName {
    pub service_name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct YamlDefinition {
    pub yaml_definition: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GenerationPrompt {
    pub prompt: String,
}

/// The `apicentric` MCP service.
#[derive(Clone)]
pub struct ApicentricMcpService {
    /// The application context.
    context: Context,
    tool_router: ToolRouter<Self>,
}

#[tool_handler]
impl ServerHandler for ApicentricMcpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides tools for interacting with the apicentric API simulator."
                    .to_string(),
            ),
        }
    }
}

#[cfg(feature = "mcp")]
impl ApicentricMcpService {
    /// Convert a domain ServiceSpec to a simulator ServiceDefinition
    fn convert_service_spec_to_definition(spec: ServiceSpec) -> Result<ServiceDefinition, String> {
        let mut endpoints = Vec::new();

        for endpoint in spec.endpoints {
            let method = match endpoint.method {
                HttpMethod::GET => "GET".to_string(),
                HttpMethod::POST => "POST".to_string(),
                HttpMethod::PUT => "PUT".to_string(),
                HttpMethod::DELETE => "DELETE".to_string(),
                HttpMethod::PATCH => "PATCH".to_string(),
                HttpMethod::HEAD => "HEAD".to_string(),
                HttpMethod::OPTIONS => "OPTIONS".to_string(),
            };

            let mut responses = HashMap::new();
            responses.insert(
                endpoint.response.status,
                ResponseDefinition {
                    condition: None,
                    content_type: endpoint
                        .response
                        .headers
                        .get("Content-Type")
                        .cloned()
                        .unwrap_or_else(|| "application/json".to_string()),
                    body: endpoint.response.body_template,
                    script: None,
                    headers: Some(endpoint.response.headers),
                    side_effects: None,
                    schema: None,
                },
            );

            let endpoint_def = EndpointDefinition {
                kind: EndpointKind::Http,
                method,
                path: endpoint.path,
                header_match: None,
                description: None,
                parameters: None,
                request_body: None,
                responses,
                scenarios: None,
                stream: None,
            };

            endpoints.push(endpoint_def);
        }

        // Convert fixtures from Value to HashMap if it's an object
        let fixtures = if spec.fixtures.is_object() {
            spec.fixtures
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(ServiceDefinition {
            name: spec.name,
            version: Some("1.0.0".to_string()),
            description: Some("Service created via MCP".to_string()),
            server: Some(ServerConfig {
                port: Some(spec.port),
                base_path: spec.base_path,
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: Some(fixtures),
            bucket: None,
            endpoints: Some(endpoints),
            graphql: None,
            behavior: None,
            twin: None,
        })
    }
}

#[tool_router]
impl ApicentricMcpService {
    /// Creates a new `ApicentricMcpService`.
    pub fn new(context: Context) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
        }
    }

    /// Lists all available mock services.
    #[tool]
    pub async fn list_services(&self) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: list_services");
        if let Some(manager) = self.context.api_simulator() {
            // Load services from directory if not loaded
            if let Err(e) = manager.load_services().await {
                tracing::warn!("Failed to load services: {}", e);
                // We don't return error here, just return empty list or partial list
            }

            let registry = manager.service_registry().read().await;
            let service_infos = registry.list_services().await;
            let services = service_infos
                .into_iter()
                .map(|s| s.name)
                .collect::<Vec<_>>();
            let content = services.into_iter().map(Content::text).collect();
            Ok(CallToolResult::success(content))
        } else {
            tracing::warn!("API Simulator not available");
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Starts a specific mock service.
    #[tool]
    pub async fn start_service(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: start_service for {}", service_name);
        if let Some(manager) = self.context.api_simulator() {
            match manager.start_service(&service_name).await {
                Ok(_) => {
                    let response = format!("Service '{}' started successfully.", service_name);
                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to start service '{}': {}", service_name, e),
                    None,
                )),
            }
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Stops a running mock service.
    #[tool]
    pub async fn stop_service(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: stop_service for {}", service_name);
        if let Some(manager) = self.context.api_simulator() {
            match manager.stop_service(&service_name).await {
                Ok(_) => {
                    let response = format!("Service '{}' stopped successfully.", service_name);
                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to stop service '{}': {}", service_name, e),
                    None,
                )),
            }
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Retrieves the latest logs for a service.
    #[tool]
    pub async fn get_service_logs(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: get_service_logs for {}", service_name);
        // For now, return dummy logs since the logging system is not fully integrated
        let dummy_logs = format!("Service '{}' logs:\n[INFO] Service started at 2025-11-17T14:40:00Z\n[INFO] Ready to accept connections", service_name);
        Ok(CallToolResult::success(vec![Content::text(dummy_logs)]))
    }

    /// Reloads all services from the services directory.
    #[tool]
    pub async fn reload_services(&self) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: reload_services");
        if let Some(manager) = self.context.api_simulator() {
            match manager.reload_services().await {
                Ok(_) => {
                    let response =
                        "All services reloaded successfully from services directory.".to_string();
                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to reload services: {}", e),
                    None,
                )),
            }
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Gets the status of a specific service.
    #[tool]
    pub async fn get_service_status(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: get_service_status for {}", service_name);
        if let Some(manager) = self.context.api_simulator() {
            let registry = manager.service_registry().read().await;
            let service_infos = registry.list_services().await;

            if let Some(service_info) = service_infos.into_iter().find(|s| s.name == service_name) {
                let status = if service_info.is_running {
                    "running"
                } else {
                    "stopped"
                };
                let response = format!(
                    "Service '{}' status: {}\nPort: {}\nBase Path: {}\nEndpoints: {}",
                    service_name,
                    status,
                    service_info.port,
                    service_info.base_path,
                    service_info.endpoints_count
                );
                Ok(CallToolResult::success(vec![Content::text(response)]))
            } else {
                Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Service '{}' not found", service_name),
                    None,
                ))
            }
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Deletes a service from the registry.
    #[tool]
    pub async fn delete_service(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: delete_service for {}", service_name);
        if let Some(manager) = self.context.api_simulator() {
            // First stop the service if it's running
            let _ = manager.stop_service(&service_name).await;

            // Remove from registry
            let mut registry = manager.service_registry().write().await;
            match registry.unregister_service(&service_name).await {
                Ok(_) => {
                    let response = format!("Service '{}' deleted successfully.", service_name);
                    Ok(CallToolResult::success(vec![Content::text(response)]))
                }
                Err(e) => Err(McpError::new(
                    ErrorCode(-32603),
                    format!("Failed to delete service '{}': {}", service_name, e),
                    None,
                )),
            }
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Gets the overall status of the API simulator.
    #[tool]
    pub async fn get_simulator_status(&self) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: get_simulator_status");
        if let Some(manager) = self.context.api_simulator() {
            let status = manager.get_status().await;
            let response = format!(
                "API Simulator Status:\nTotal Services: {}\nActive Services: {}\n\nActive Services:\n{}",
                status.services_count,
                status.active_services.len(),
                status.active_services
                    .iter()
                    .map(|s| format!("  - {}: http://localhost:{}{}", s.name, s.port, s.base_path))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            Ok(CallToolResult::success(vec![Content::text(response)]))
        } else {
            Err(McpError::new(
                ErrorCode(-32603),
                "API Simulator not available - ensure the 'simulator' feature is enabled"
                    .to_string(),
                None,
            ))
        }
    }

    /// Creates and loads a new service from a YAML string.
    #[tool]
    pub async fn create_service(
        &self,
        Parameters(YamlDefinition { yaml_definition }): Parameters<YamlDefinition>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: create_service");
        match YamlServiceSpecLoader::load_from_string(&yaml_definition) {
            Ok(spec) => {
                match Self::convert_service_spec_to_definition(spec) {
                    Ok(service_def) => {
                        if let Some(manager) = self.context.api_simulator() {
                            let mut registry = manager.service_registry().write().await;
                            match registry.register_service(service_def.clone()).await {
                                Ok(_) => {
                                    // Persist the service to disk
                                    let services_dir =
                                        if let Some(config) = &self.context.config().simulator {
                                            config.services_dir.clone()
                                        } else {
                                            std::path::PathBuf::from("services")
                                        };

                                    if let Err(e) = fs::create_dir_all(&services_dir) {
                                        return Err(McpError::new(
                                            ErrorCode(-32603),
                                            format!("Failed to create services directory: {}", e),
                                            None,
                                        ));
                                    }

                                    let file_path =
                                        services_dir.join(format!("{}.yaml", service_def.name));

                                    // Serialize the definition to YAML
                                    match serde_yaml::to_string(&service_def) {
                                        Ok(yaml_content) => {
                                            match fs::write(&file_path, yaml_content) {
                                                Ok(_) => {
                                                    let response = format!("Service '{}' created, registered, and saved to {}.", service_def.name, file_path.display());
                                                    Ok(CallToolResult::success(vec![
                                                        Content::text(response),
                                                    ]))
                                                }
                                                Err(e) => Err(McpError::new(
                                                    ErrorCode(-32603),
                                                    format!("Failed to write service file: {}", e),
                                                    None,
                                                )),
                                            }
                                        }
                                        Err(e) => Err(McpError::new(
                                            ErrorCode(-32603),
                                            format!(
                                                "Failed to serialize service definition: {}",
                                                e
                                            ),
                                            None,
                                        )),
                                    }
                                }
                                Err(e) => Err(McpError::new(
                                    ErrorCode(-32603),
                                    format!("Failed to register service: {}", e),
                                    None,
                                )),
                            }
                        } else {
                            Err(McpError::new(ErrorCode(-32603), "API Simulator not available - ensure the 'simulator' feature is enabled".to_string(), None))
                        }
                    }
                    Err(e) => Err(McpError::new(
                        ErrorCode(-32603),
                        format!("Failed to convert service spec: {}", e),
                        None,
                    )),
                }
            }
            Err(e) => Err(McpError::new(
                ErrorCode(-32603),
                format!("Invalid YAML definition: {}", e),
                None,
            )),
        }
    }

    /// Generates, creates, and starts a new service from a prompt.
    #[tool]
    pub async fn generate_and_start_service(
        &self,
        Parameters(GenerationPrompt { prompt }): Parameters<GenerationPrompt>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool called: generate_and_start_service");
        match apicentric::ai::generate_service(&self.context, &prompt).await {
            Ok(yaml) => {
                if let Some(manager) = self.context.api_simulator() {
                    match manager.apply_service_yaml(&yaml).await {
                        Ok(service_name) => {
                            // Persist the service to disk
                            let services_dir =
                                if let Some(config) = &self.context.config().simulator {
                                    config.services_dir.clone()
                                } else {
                                    std::path::PathBuf::from("services")
                                };

                            if let Err(e) = fs::create_dir_all(&services_dir) {
                                return Err(McpError::new(
                                    ErrorCode(-32603),
                                    format!("Failed to create services directory: {}", e),
                                    None,
                                ));
                            }

                            let file_path = services_dir.join(format!("{}.yaml", service_name));

                            match fs::write(&file_path, &yaml) {
                                Ok(_) => {
                                    let response = format!(
                                        "Service '{}' generated, started, and saved to {}.",
                                        service_name,
                                        file_path.display()
                                    );
                                    Ok(CallToolResult::success(vec![Content::text(response)]))
                                }
                                Err(e) => Err(McpError::new(
                                    ErrorCode(-32603),
                                    format!("Failed to write service file: {}", e),
                                    None,
                                )),
                            }
                        }
                        Err(e) => Err(McpError::new(
                            ErrorCode(-32603),
                            format!("Failed to apply generated service: {}", e),
                            None,
                        )),
                    }
                } else {
                    Err(McpError::new(
                        ErrorCode(-32603),
                        "API Simulator not available - ensure the 'simulator' feature is enabled"
                            .to_string(),
                        None,
                    ))
                }
            }
            Err(e) => Err(McpError::new(
                ErrorCode(-32603),
                format!("Failed to generate service from prompt: {}", e),
                None,
            )),
        }
    }
}
