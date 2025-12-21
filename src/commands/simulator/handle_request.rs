use apicentric::errors::{ApicentricError, ApicentricResult};
use apicentric::simulator::{ServiceDefinition, ServiceInstance};
use apicentric::storage::sqlite::SqliteStorage;
use apicentric::ExecutionContext;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn execute(
    service_path: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
    headers_json: Option<&str>,
    _exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    // 1. Load service definition
    let content = tokio::fs::read_to_string(service_path).await.map_err(|e| {
        ApicentricError::config_error(
            format!("Failed to read service file: {}", e),
            Some(format!("Check if path '{}' exists", service_path)),
        )
    })?;

    let service_def: ServiceDefinition = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::validation_error(format!("Invalid service YAML: {}", e), None::<String>, None::<String>)
    })?;

    // 2. Initialize ephemeral service instance
    // Use in-memory storage for this transient request
    let storage = Arc::new(SqliteStorage::init_db(":memory:").expect("Failed to init memory db"));
    let (log_sender, _) = broadcast::channel(10); // We won't consume logs here

    // We can use a dummy port since we won't start the TCP listener
    let mut service_instance = ServiceInstance::new(service_def, 0, storage, log_sender)?;

    // We need to set running=true to bypass the check in `handle_request`, but `start()` spawns a server.
    // Instead of calling `start()`, we can construct the request and call `handle_request` which checks `is_running`.
    // Wait, `ServiceInstance` struct has `is_running` field.
    // I need to bypass the `is_running` check or manually set it.
    // Since `is_running` is private and checked in `handle_request`, I might need to call `start()` but that binds a port.
    // A workaround is to modify `ServiceInstance` to allow handling requests without starting the server,
    // or call the internal static method if possible.
    // `ServiceInstance::handle_request_static` is private.

    // Let's rely on the fact that `ServiceInstance` exposes `handle_request` which checks `is_running`.
    // I can modify `ServiceInstance` in `src/simulator/service/mod.rs` to allow testing/CLI usage without binding port.
    // OR, I can just bind to port 0 (random ephemeral), start it, handle request, then stop.
    // That seems safest without modifying core logic too much.

    service_instance.start().await?;

    // 3. Construct the Hyper Request
    let mut builder = Request::builder().method(method).uri(path);

    if let Some(headers_str) = headers_json {
        if let Ok(headers_map) = serde_json::from_str::<HashMap<String, String>>(headers_str) {
            for (k, v) in headers_map {
                builder = builder.header(k, v);
            }
        }
    }

    let req_body = if let Some(b) = body {
        Full::new(Bytes::from(b.to_string()))
    } else {
        Full::new(Bytes::new())
    };

    let _req = builder.body(req_body).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to build request: {}", e), None::<String>)
    })?;

    // Convert Full<Bytes> to Incoming is hard because Incoming is internal.
    // `ServiceInstance::handle_request` takes `Request<hyper::body::Incoming>`.
    // Wait, `Incoming` is for the server side.
    // I need to change `ServiceInstance::handle_request` to take generic Body or something I can construct.

    // Let's check `src/simulator/service/mod.rs` again.
    // `pub async fn handle_request(&self, req: Request<hyper::body::Incoming>)`
    // `Incoming` is `hyper::body::Incoming`. I cannot construct it easily.

    // I should modify `ServiceInstance::handle_request` to accept `Request<B>` where `B: Body`.
    // But `handle_request_static` uses `body.collect().await`, so it needs `Body` trait.

    // Alternative: Make the CLI command send a real HTTP request to the bound localhost port.
    // Since we started the service on port 0, we can query `service_instance.port()` to find the actual port.
    let port = service_instance.port();
    let url = format!("http://127.0.0.1:{}{}", port, path);

    let client = reqwest::Client::new();
    let mut req_builder = client.request(
        method.parse().unwrap_or(reqwest::Method::GET),
        url,
    );

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

    // 4. Capture response and print to stdout
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

    // 5. Cleanup
    service_instance.stop().await?;

    Ok(())
}
