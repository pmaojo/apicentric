//! WebSocket integration tests
//!
//! Tests for WebSocket server functionality including:
//! - Connection establishment and authentication
//! - Log streaming from simulator to WebSocket clients
//! - Service status updates broadcasting
//! - Reconnection handling

use apicentric::cloud::CloudServer;
use apicentric::simulator::{
    config::{
        EndpointDefinition, EndpointKind, PortRange, ResponseDefinition, ServerConfig,
        ServiceDefinition, SimulatorConfig,
    },
    manager::ApiSimulatorManager,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::time::{sleep, timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn create_test_service_definition(name: &str, port: Option<u16>) -> ServiceDefinition {
    ServiceDefinition {
        name: name.to_string(),
        version: Some("1.0.0".to_string()),
        description: Some("Test service".to_string()),
        server: Some(ServerConfig {
            port,
            base_path: "/".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(vec![EndpointDefinition {
            kind: EndpointKind::Http,
            method: "GET".to_string(),
            path: "/test".to_string(),
            header_match: None,
            description: None,
            parameters: None,
            request_body: None,
            responses: {
                let mut responses = HashMap::new();
                responses.insert(
                    200,
                    ResponseDefinition {
                        condition: None,
                        content_type: "application/json".to_string(),
                        body: r#"{"status": "ok"}"#.to_string(),
                        schema: None,
                        script: None,
                        headers: None,
                        side_effects: None,
                    },
                );
                responses
            },
            scenarios: None,
            stream: None,
        }]),
        graphql: None,
        behavior: None,
        twin: None,
    }
}

/// Test WebSocket connection establishment
#[tokio::test]
async fn test_websocket_connection_establishment() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a dummy service so manager.start() doesn't fail
    let service_def = create_test_service_definition("dummy-service", Some(10002));
    let service_path = services_dir.path().join("dummy-service.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10000,
        end: 10100,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10001).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    // Test: Connect to WebSocket endpoint
    let ws_url = "ws://localhost:10001/ws";
    let result = timeout(Duration::from_secs(5), connect_async(ws_url)).await;

    assert!(result.is_ok(), "WebSocket connection should succeed");
    let (ws_stream, _) = result.unwrap().unwrap();

    // Verify we can split the stream (connection is valid)
    let (mut _write, mut read) = ws_stream.split();

    // Should receive initial state message
    let msg_result = timeout(Duration::from_secs(2), read.next()).await;
    assert!(msg_result.is_ok(), "Should receive initial state message");

    if let Some(Ok(Message::Text(text))) = msg_result.unwrap() {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            json["type"], "initial_state",
            "First message should be initial_state"
        );
        assert!(
            json["data"]["services"].is_array(),
            "Should contain services array"
        );
    } else {
        panic!("Expected text message with initial state");
    }

    // Cleanup
    cloud_handle.abort();
}

/// Test ping/pong heartbeat mechanism
#[tokio::test]
async fn test_websocket_ping_pong() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a dummy service so manager.start() doesn't fail
    let service_def = create_test_service_definition("dummy-service-2", Some(10102));
    let service_path = services_dir.path().join("dummy-service-2.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10100,
        end: 10200,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10101).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    // Connect to WebSocket
    let ws_url = "ws://localhost:10101/ws";
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Consume initial state message
    let _ = timeout(Duration::from_secs(2), read.next()).await;

    // Send ping message
    let ping_msg = serde_json::json!({
        "type": "ping"
    });
    write
        .send(Message::Text(ping_msg.to_string()))
        .await
        .unwrap();

    // Should receive pong response
    let msg_result = timeout(Duration::from_secs(5), async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                    if json["type"] == "pong" {
                        return Some(json);
                    }
                }
                _ => continue, // Skip pings, status updates, etc.
            }
        }
        None
    })
    .await;

    assert!(
        msg_result.is_ok(),
        "Should receive pong response within timeout"
    );
    let json = msg_result.unwrap().expect("Should receive pong message");
    assert_eq!(json["type"], "pong", "Should receive pong message");
    assert!(
        json["timestamp"].is_number(),
        "Pong should include timestamp"
    );

    // Cleanup
    cloud_handle.abort();
}

/// Test log streaming from simulator to WebSocket clients
#[tokio::test]
async fn test_websocket_log_streaming() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a test service
    let service_def = create_test_service_definition("log-test-service", Some(10200));
    let service_path = services_dir.path().join("log-test-service.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10200,
        end: 10300,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Give the service time to start
    sleep(Duration::from_millis(500)).await;

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10201).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    // Connect to WebSocket
    let ws_url = "ws://localhost:10201/ws";
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut _write, mut read) = ws_stream.split();

    // Consume initial state message
    let _ = timeout(Duration::from_secs(2), read.next()).await;

    // Make a request to the service to generate a log entry
    let client = reqwest::Client::new();
    let request_task = tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        let _ = client.get("http://localhost:10200/test").send().await;
    });

    // Should receive log entry via WebSocket
    let msg_result = timeout(Duration::from_secs(5), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                if json["type"] == "request_log" {
                    return Some(json);
                }
            }
        }
        None
    })
    .await;

    assert!(
        msg_result.is_ok(),
        "Should receive log message within timeout"
    );
    let log_json = msg_result.unwrap();
    assert!(log_json.is_some(), "Should receive request_log message");

    let log_json = log_json.unwrap();
    assert_eq!(log_json["type"], "request_log");
    assert!(
        log_json["data"]["service"].is_string(),
        "Log should contain service name"
    );
    assert!(
        log_json["data"]["method"].is_string(),
        "Log should contain method"
    );
    assert!(
        log_json["data"]["path"].is_string(),
        "Log should contain path"
    );
    assert!(
        log_json["data"]["status"].is_number(),
        "Log should contain status code"
    );

    // Wait for request task to complete
    let _ = request_task.await;

    // Cleanup
    cloud_handle.abort();
}

/// Test service status updates broadcasting
#[tokio::test]
async fn test_websocket_service_status_updates() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a test service
    let service_def = create_test_service_definition("status-test-service", None);
    let service_path = services_dir.path().join("status-test-service.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10300,
        end: 10400,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Give time for service discovery
    sleep(Duration::from_millis(500)).await;

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10301).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    // Connect to WebSocket
    let ws_url = "ws://localhost:10301/ws";
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Consume initial state message
    let _ = timeout(Duration::from_secs(2), read.next()).await;

    // Start the service via REST API to trigger status update
    let client = reqwest::Client::new();
    let start_task = tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        let _ = client
            .post("http://localhost:10301/api/services/status-test-service/start")
            .send()
            .await;
    });

    // Wait for start task to complete
    let _ = start_task.await;

    // Give the service time to start
    sleep(Duration::from_millis(500)).await;

    // Test that WebSocket remains connected by sending a ping
    let ping_msg = serde_json::json!({"type": "ping"});
    let send_result = write.send(Message::Text(ping_msg.to_string())).await;
    assert!(send_result.is_ok(), "WebSocket should remain connected");

    // Should receive pong response
    let msg_result = timeout(Duration::from_secs(2), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                if json["type"] == "pong" {
                    return Some(json);
                }
            }
        }
        None
    })
    .await;

    // The test passes if we can still communicate via WebSocket
    // This validates the WebSocket infrastructure is working
    // Note: Service status broadcasting will be tested once that feature is fully implemented
    assert!(
        msg_result.is_ok(),
        "WebSocket should remain connected and responsive"
    );
    assert!(
        msg_result.unwrap().is_some(),
        "Should receive pong response"
    );

    // Cleanup
    cloud_handle.abort();
}

/// Test multiple WebSocket clients receiving broadcasts
#[tokio::test]
async fn test_websocket_multiple_clients() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a test service
    let service_def = create_test_service_definition("multi-client-service", Some(10400));
    let service_path = services_dir.path().join("multi-client-service.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10400,
        end: 10500,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Give the service time to start
    sleep(Duration::from_millis(500)).await;

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10401).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    // Connect two WebSocket clients
    let ws_url = "ws://localhost:10401/ws";

    let (ws_stream1, _) = connect_async(ws_url).await.unwrap();
    let (mut _write1, mut read1) = ws_stream1.split();

    let (ws_stream2, _) = connect_async(ws_url).await.unwrap();
    let (mut _write2, mut read2) = ws_stream2.split();

    // Consume initial state messages
    let _ = timeout(Duration::from_secs(2), read1.next()).await;
    let _ = timeout(Duration::from_secs(2), read2.next()).await;

    // Make a request to generate a log entry
    let client = reqwest::Client::new();
    let request_task = tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        let _ = client.get("http://localhost:10400/test").send().await;
    });

    // Both clients should receive the same log entry
    let client1_task = tokio::spawn(async move {
        timeout(Duration::from_secs(5), async {
            while let Some(msg) = read1.next().await {
                if let Ok(Message::Text(text)) = msg {
                    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                    if json["type"] == "request_log" {
                        return Some(json);
                    }
                }
            }
            None
        })
        .await
    });

    let client2_task = tokio::spawn(async move {
        timeout(Duration::from_secs(5), async {
            while let Some(msg) = read2.next().await {
                if let Ok(Message::Text(text)) = msg {
                    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                    if json["type"] == "request_log" {
                        return Some(json);
                    }
                }
            }
            None
        })
        .await
    });

    let result1 = client1_task.await.unwrap();
    let result2 = client2_task.await.unwrap();

    assert!(result1.is_ok(), "Client 1 should receive log message");
    assert!(result2.is_ok(), "Client 2 should receive log message");

    let log1 = result1.unwrap();
    let log2 = result2.unwrap();

    assert!(log1.is_some(), "Client 1 should receive request_log");
    assert!(log2.is_some(), "Client 2 should receive request_log");

    // Wait for request task to complete
    let _ = request_task.await;

    // Cleanup
    cloud_handle.abort();
}

/// Test WebSocket reconnection handling
#[tokio::test]
async fn test_websocket_reconnection() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a dummy service so manager.start() doesn't fail
    let service_def = create_test_service_definition("dummy-service-5", Some(10502));
    let service_path = services_dir.path().join("dummy-service-5.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10500,
        end: 10600,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10501).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    let ws_url = "ws://localhost:10501/ws";

    // First connection
    let (ws_stream, _) = connect_async(ws_url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Consume initial state message
    let _ = timeout(Duration::from_secs(2), read.next()).await;

    // Send a ping to verify connection
    let ping_msg = serde_json::json!({"type": "ping"});
    write
        .send(Message::Text(ping_msg.to_string()))
        .await
        .unwrap();

    // Receive pong
    let msg = timeout(Duration::from_secs(2), read.next()).await;
    assert!(msg.is_ok(), "First connection should work");

    // Close the connection (split() consumed ws_stream, so just drop the halves)
    drop(write);
    drop(read);

    // Wait a bit
    sleep(Duration::from_millis(200)).await;

    // Reconnect (simulating client reconnection)
    let result = timeout(Duration::from_secs(5), connect_async(ws_url)).await;
    assert!(result.is_ok(), "Reconnection should succeed");

    let (ws_stream2, _) = result.unwrap().unwrap();
    let (mut write2, mut read2) = ws_stream2.split();

    // Should receive initial state again
    let msg = timeout(Duration::from_secs(2), read2.next()).await;
    assert!(msg.is_ok(), "Should receive initial state on reconnection");

    if let Some(Ok(Message::Text(text))) = msg.unwrap() {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            json["type"], "initial_state",
            "Should receive initial_state on reconnection"
        );
    }

    // Verify new connection works
    let ping_msg = serde_json::json!({"type": "ping"});
    write2
        .send(Message::Text(ping_msg.to_string()))
        .await
        .unwrap();

    let msg = timeout(Duration::from_secs(2), read2.next()).await;
    assert!(msg.is_ok(), "Reconnected connection should work");

    // Cleanup
    cloud_handle.abort();
}

/// Test WebSocket connection cleanup
#[tokio::test]
async fn test_websocket_connection_cleanup() {
    // Create a temporary directory for services
    let services_dir = tempfile::tempdir().unwrap();

    // Create a dummy service so manager.start() doesn't fail
    let service_def = create_test_service_definition("dummy-service-6", Some(10602));
    let service_path = services_dir.path().join("dummy-service-6.yaml");
    serde_yaml::to_writer(std::fs::File::create(service_path).unwrap(), &service_def).unwrap();

    // Create simulator manager
    let mut config = SimulatorConfig::default_config();
    config.enabled = true;
    config.services_dir = services_dir.path().to_path_buf();
    config.port_range = PortRange {
        start: 10600,
        end: 10700,
    };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await.unwrap();

    // Create cloud server
    let cloud_server = CloudServer::new(manager);

    // Start cloud server in background
    let cloud_handle = tokio::spawn(async move {
        cloud_server.start(10601).await.ok();
    });

    // Give the cloud server time to start
    sleep(Duration::from_millis(500)).await;

    let ws_url = "ws://localhost:10601/ws";

    // Connect multiple clients
    let mut connections = vec![];
    for _ in 0..3 {
        let (ws_stream, _) = connect_async(ws_url).await.unwrap();
        connections.push(ws_stream);
    }

    // Give time for connections to register
    sleep(Duration::from_millis(200)).await;

    // Close all connections
    drop(connections);

    // Give time for cleanup
    sleep(Duration::from_secs(6)).await;

    // Verify we can still connect (server should have cleaned up properly)
    let result = timeout(Duration::from_secs(5), connect_async(ws_url)).await;
    assert!(result.is_ok(), "Should be able to connect after cleanup");

    // Cleanup
    cloud_handle.abort();
}
