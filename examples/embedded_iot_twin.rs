use apicentric::iot::config::{AdapterConfig, PhysicsConfig, TwinDefinition};
use apicentric::simulator::config::{PortRange, ServiceDefinition, SimulatorConfig};
use apicentric::simulator::ApiSimulatorManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

/// This example demonstrates how to embed Apicentric as a library to run an IoT Digital Twin.
/// It programmatically defines a twin with physics simulation (Sine Wave) and MQTT transport.
///
/// This shows Apicentric's capability as a protocol-agnostic server SDK, handling not just HTTP
/// but also MQTT, Modbus, and other IoT protocols in a unified way.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Logging
    env_logger::init();

    println!("🚀 initializing embedded IoT Twin...");

    // 2. Configure the Simulator
    let config = SimulatorConfig::new(
        true,
        PathBuf::from("./temp_services"),
        PortRange {
            start: 9600,
            end: 9700,
        },
    );

    // 3. Create the Manager
    let manager = ApiSimulatorManager::new(config);

    // 4. Programmatically Define a Digital Twin
    // This twin represents a "Temperature Sensor" with physics behavior.

    // Define Physics: A sine wave simulating temperature fluctuation
    let mut physics_params = HashMap::new();
    physics_params.insert("min".to_string(), serde_yaml::to_value(20.0)?);
    physics_params.insert("max".to_string(), serde_yaml::to_value(30.0)?);

    let physics = PhysicsConfig {
        variable: "temperature".to_string(),
        strategy: "sine".to_string(), // Built-in sine wave strategy
        params: physics_params,
    };

    // Define Transport: MQTT (simulated publishing)
    // Note: This requires an MQTT broker to be running on localhost:1883 to actually connect.
    // The simulator will attempt to connect but handle failures gracefully.
    let mut transport_params = HashMap::new();
    transport_params.insert("broker_url".to_string(), serde_yaml::to_value("localhost")?);
    transport_params.insert("port".to_string(), serde_yaml::to_value(1883)?);
    transport_params.insert("topic_prefix".to_string(), serde_yaml::to_value("sensors/temp01")?);
    transport_params.insert("client_id".to_string(), serde_yaml::to_value("embedded_twin_01")?);

    let transport = AdapterConfig {
        adapter_type: "mqtt".to_string(),
        params: transport_params,
    };

    let twin_def = TwinDefinition {
        name: "embedded-temp-sensor".to_string(),
        physics: vec![physics],
        transports: vec![transport],
    };

    // Wrap in ServiceDefinition
    // Note: 'twin' is an optional field in ServiceDefinition. When present, the simulator
    // runs it as an IoT actor instead of a standard HTTP server.
    let service_def = ServiceDefinition {
        name: "embedded-temp-sensor".to_string(),
        version: Some("1.0.0".to_string()),
        description: Some("An embedded IoT Digital Twin".to_string()),
        server: None, // No HTTP server config needed for pure Twin
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: None,
        graphql: None,
        behavior: None,
        twin: Some(twin_def),
    };

    // 5. Register and Start the Twin
    {
        let mut registry = manager.service_registry().write().await;
        registry.register_service(service_def.clone()).await?;
    }

    manager.start_service("embedded-temp-sensor").await?;

    println!("✅ Digital Twin 'embedded-temp-sensor' started");
    println!("   Physics: Sine wave (20.0 - 30.0)");
    println!("   Transport: MQTT (localhost:1883)");
    println!("   (Make sure you have an MQTT broker running to see messages, e.g., 'mosquitto')");

    // 6. Monitor the Simulation (Logs)
    // The manager broadcasts logs which we can subscribe to.
    let mut log_rx = manager.subscribe_logs();

    println!("📡 Listening for telemetry logs (simulating for 5 seconds)...");

    let timeout = sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            Ok(log) = log_rx.recv() => {
                if log.service == "embedded-temp-sensor" && log.method == "TICK" {
                    println!("📊 Telemetry: {}", log.payload.unwrap_or_default());
                }
            }
            _ = &mut timeout => {
                break;
            }
        }
    }

    // 7. Cleanup
    manager.stop_service("embedded-temp-sensor").await?;
    println!("🛑 Twin stopped. Exiting.");

    Ok(())
}
