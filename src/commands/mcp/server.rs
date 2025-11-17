//! The MCP server implementation for `apicentric`.

#![cfg(feature = "mcp")]

use apicentric::Context;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router, ErrorData as McpError,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ServiceName {
    pub service_name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct YamlDefinition {
    pub yaml_definition: String,
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
        let services = vec!["service1".to_string(), "service2".to_string()];
        let content = services.into_iter().map(Content::text).collect();
        Ok(CallToolResult::success(content))
    }

    /// Starts a specific mock service.
    #[tool]
    pub async fn start_service(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        let response = format!("Service '{}' started.", service_name);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    /// Stops a running mock service.
    #[tool]
    pub async fn stop_service(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        let response = format!("Service '{}' stopped.", service_name);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    /// Retrieves the latest logs for a service.
    #[tool]
    pub async fn get_service_logs(
        &self,
        Parameters(ServiceName { service_name }): Parameters<ServiceName>,
    ) -> Result<CallToolResult, McpError> {
        let response = format!("Logs for service '{}'.", service_name);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    /// Creates and loads a new service from a YAML string.
    #[tool]
    pub async fn create_service(
        &self,
        Parameters(YamlDefinition { yaml_definition }): Parameters<YamlDefinition>,
    ) -> Result<CallToolResult, McpError> {
        let response = format!("Service created from YAML: {}", yaml_definition);
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}
