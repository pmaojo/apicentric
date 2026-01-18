use apicentric::simulator::config::{
    EndpointDefinition, EndpointKind, PortRange, ResponseDefinition, ServerConfig,
    ServiceDefinition, SimulatorConfig,
};
use apicentric::simulator::ApiSimulatorManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

/// This example demonstrates how to embed Apicentric as a library within a CLI application.
/// It programmatically defines an API service (without needing external YAML files),
/// starts the server, and then consumes it locally.
///
/// This pattern allows `apicentric` to act as the protocol layer for your own tools.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Logging (optional, but good for visibility)
    env_logger::init();

    println!("🚀 initializing embedded API server...");

    // 2. Configure the Server
    // We point 'services_dir' to a temp location since we are creating the service in-memory.
    let config = SimulatorConfig::new(
        true,
        PathBuf::from("./temp_services"), // Just a placeholder, we won't load from here
        PortRange {
            start: 9500,
            end: 9600,
        },
    );

    // 3. Create the Manager
    let manager = ApiSimulatorManager::new(config);

    // 4. Programmatically Define a Service
    // This represents the "Real App" API capabilities you want to expose.
    let mut endpoints = Vec::new();

    // Define a GET endpoint
    let mut responses_get = HashMap::new();
    responses_get.insert(
        200,
        ResponseDefinition {
            condition: None,
            content_type: "application/json".to_string(),
            body: "{\"status\": \"active\", \"message\": \"Hello from embedded apicentric!\"}"
                .to_string(),
            script: None,
            headers: None,
            side_effects: None,
        },
    );

    endpoints.push(EndpointDefinition {
        kind: EndpointKind::Http,
        method: "GET".to_string(),
        path: "/status".to_string(),
        header_match: None,
        description: Some("Check system status".to_string()),
        parameters: None,
        request_body: None,
        responses: responses_get,
        scenarios: None,
        stream: None,
    });

    // Define a POST endpoint that echoes data
    let mut responses_post = HashMap::new();
    responses_post.insert(
        201,
        ResponseDefinition {
            condition: None,
            content_type: "application/json".to_string(),
            body: "{\"received\": {{json request.body}}, \"timestamp\": \"{{now}}\"}".to_string(), // Uses Handlebars
            script: None,
            headers: None,
            side_effects: None,
        },
    );

    endpoints.push(EndpointDefinition {
        kind: EndpointKind::Http,
        method: "POST".to_string(),
        path: "/data".to_string(),
        header_match: None,
        description: Some("Submit data".to_string()),
        parameters: None,
        request_body: None,
        responses: responses_post,
        scenarios: None,
        stream: None,
    });

    let service_def = ServiceDefinition {
        name: "embedded-cli-service".to_string(),
        version: Some("1.0.0".to_string()),
        description: Some("An API provided by the CLI tool itself".to_string()),
        server: Some(ServerConfig {
            port: Some(9500),
            base_path: "/api".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(endpoints),
        graphql: None,
        behavior: None,
        twin: None,
    };

    // 5. Register and Start the Service
    // In a real app, you might do this via the Manager's internal registry directly if exposed,
    // or use the helper method `apply_service_definition` (if available via P2P or generally).
    // Here we use the public `apply_service_definition` (which might be gated by features, so checking availability).
    // Note: `apply_service_definition` is available on the manager.

    // We need to use `apply_service_definition` but `ApiSimulatorManager` in `src/simulator/manager.rs`
    // has it. Let's try to register it directly or use `apply_service_yaml`.
    // Since we constructed the struct, let's look for a way to register it.
    // The `service_registry` is exposed via `service_registry()`, so we can write to it.

    {
        let mut registry = manager.service_registry().write().await;
        registry.register_service(service_def.clone()).await?;
    }

    // Now start it
    manager.start_service("embedded-cli-service").await?;

    println!("✅ Service 'embedded-cli-service' started on port 9500");

    // 6. Consume the API (Internal Protocol Usage)
    // The CLI tool can now use standard HTTP clients to talk to its own embedded server,
    // or expose this port for other local tools to use.

    println!("⏳ Waiting for server to be fully ready...");
    sleep(Duration::from_millis(500)).await;

    println!("📡 Sending request to embedded API...");
    let client = reqwest::Client::new();

    // Test GET
    let resp = client.get("http://localhost:9500/api/status")
        .send()
        .await?
        .text()
        .await?;
    println!("📥 GET /api/status response: {}", resp);

    // Test POST
    let resp = client.post("http://localhost:9500/api/data")
        .json(&serde_json::json!({"task": "processing", "id": 123}))
        .send()
        .await?
        .text()
        .await?;
    println!("📥 POST /api/data response: {}", resp);

    // 7. Cleanup
    manager.stop_service("embedded-cli-service").await?;
    println!("🛑 Service stopped. Exiting.");

    Ok(())
}
