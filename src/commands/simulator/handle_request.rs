use apicentric::errors::{ApicentricError, ApicentricResult};
use apicentric::simulator::{ServiceDefinition, ServiceInstance};
use apicentric::storage::sqlite::SqliteStorage;
use apicentric::ExecutionContext;
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn execute(
    service_path: &str,
    method: &str,
    path: &str,
    headers_json: Option<&str>,
    _exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    // 1. Read body from stdin
    let mut buffer = String::new();
    // We use std::io::stdin() which blocks, but since we are in a CLI command execution context
    // and this is the primary input, blocking here is acceptable/expected before we go async.
    // However, to be async-friendly we could use tokio::io::stdin.
    // Given the previous implementation used simple strings, let's stick to reading all into string.
    // If the input is empty or piped, this will work.
    let _ = std::io::stdin().read_to_string(&mut buffer);
    let body = if buffer.is_empty() {
        None
    } else {
        Some(buffer.as_str())
    };

    // 2. Load service definition
    let content = tokio::fs::read_to_string(service_path).await.map_err(|e| {
        ApicentricError::config_error(
            format!("Failed to read service file: {}", e),
            Some(format!("Check if path '{}' exists", service_path)),
        )
    })?;

    let service_def: ServiceDefinition = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::validation_error(
            format!("Invalid service YAML: {}", e),
            None::<String>,
            None::<String>,
        )
    })?;

    // 3. Initialize ephemeral service instance
    let storage = Arc::new(SqliteStorage::init_db(":memory:").expect("Failed to init memory db"));
    let (log_sender, _) = broadcast::channel(10);

    // Use port 0 to bind to a random available port
    let mut service_instance = ServiceInstance::new(service_def, 0, storage, log_sender)?;
    service_instance.start().await?;

    // 4. Send request to the ephemeral server
    let port = service_instance.port();
    let url = format!("http://127.0.0.1:{}{}", port, path);

    let client = reqwest::Client::new();
    let mut req_builder = client.request(method.parse().unwrap_or(reqwest::Method::GET), url);

    if let Some(headers_str) = headers_json {
        if let Ok(headers_map) = serde_json::from_str::<HashMap<String, String>>(headers_str) {
            for (k, v) in headers_map {
                req_builder = req_builder.header(k, v);
            }
        }
    }

    if let Some(b) = body {
        req_builder = req_builder.body(b.to_string());
    }

    let resp = req_builder.send().await.map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to send request to ephemeral server: {}", e),
            None::<String>,
        )
    })?;

    // 5. Capture response and print to stdout
    let status = resp.status().as_u16();
    let resp_headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let resp_body = resp.text().await.unwrap_or_default();

    let output = serde_json::json!({
        "status": status,
        "headers": resp_headers,
        "body": resp_body
    });

    println!("{}", output);

    // 6. Cleanup
    service_instance.stop().await?;

    Ok(())
}
