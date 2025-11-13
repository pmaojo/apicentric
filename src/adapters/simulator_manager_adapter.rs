//! An adapter to make the `ApiSimulatorManager` compatible with the `ContractMockApiRunner` trait.
use crate::{
    domain::{
        contract_testing::{ApiResponse, ResponseBody, ValidationScenario},
        ports::contract::{ContractMockApiRunner, MockApiError, MockApiHandle},
    },
    simulator::{ApiSimulatorManager, SimulatorConfig},
};
use async_trait::async_trait;
use tokio::fs;

pub struct SimulatorManagerAdapter {
    manager: ApiSimulatorManager,
}

impl SimulatorManagerAdapter {
    pub fn new() -> Self {
        let config = SimulatorConfig {
            enabled: true,
            ..Default::default()
        };
        Self {
            manager: ApiSimulatorManager::new(config),
        }
    }
}

#[async_trait]
impl ContractMockApiRunner for SimulatorManagerAdapter {
    async fn start(&self, path: &str) -> Result<MockApiHandle, MockApiError> {
        let yaml_content = fs::read_to_string(path)
            .await
            .map_err(|e| MockApiError::StartupError(format!("Failed to read service file: {}", e)))?;

        self.manager
            .apply_service_yaml(&yaml_content)
            .await
            .map_err(|e| {
                MockApiError::StartupError(format!("Failed to apply service definition: {}", e))
            })?;

        self.manager
            .start()
            .await
            .map_err(|e| MockApiError::StartupError(format!("Failed to start simulator: {}", e)))?;

        let status = self.manager.get_status().await;
        let service_info = status.active_services.first().ok_or_else(|| {
            MockApiError::StartupError("Simulator started, but no active services found".to_string())
        })?;

        Ok(MockApiHandle {
            port: service_info.port,
            base_url: format!("http://localhost:{}", service_info.port),
            process_id: None, // The manager runs in-process
        })
    }

    async fn stop(&self, _handle: MockApiHandle) -> Result<(), MockApiError> {
        self.manager
            .stop()
            .await
            .map_err(|e| MockApiError::StartupError(e.to_string()))
    }

    async fn execute_request(
        &self,
        handle: &MockApiHandle,
        sc: &ValidationScenario,
    ) -> Result<ApiResponse, MockApiError> {
        let url = format!("{}{}", handle.base_url, sc.path);
        let client = reqwest::Client::new();
        let method = sc.method.to_string().to_uppercase();
        let req_method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| MockApiError::StartupError("Invalid HTTP method".to_string()))?;

        let response = client
            .request(req_method, &url)
            .send()
            .await
            .map_err(|e| MockApiError::StartupError(e.to_string()))?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| MockApiError::StartupError(e.to_string()))?;

        let json_body: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
        let body = ResponseBody::Json(json_body);

        Ok(ApiResponse::new(status, headers, body, 0))
    }

    async fn render_response(
        &self,
        _sc: &ValidationScenario,
        _fx: &serde_json::Value,
    ) -> Result<ApiResponse, MockApiError> {
        unimplemented!("Response rendering is not supported in this adapter")
    }
}

impl Default for SimulatorManagerAdapter {
    fn default() -> Self {
        Self::new()
    }
}
