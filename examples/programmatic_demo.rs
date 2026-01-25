#[cfg(all(feature = "iot", feature = "simulator"))]
use apicentric::iot::{
    model::DigitalTwin, physics::sine::SineWaveStrategy, traits::SimulationStrategy,
};
#[cfg(all(feature = "iot", feature = "simulator"))]
use apicentric::simulator::config::{EndpointKind, PortRange, ServerConfig};
#[cfg(all(feature = "iot", feature = "simulator"))]
use apicentric::simulator::{
    ApiSimulatorManager, EndpointDefinition, ResponseDefinition, ServiceDefinition, SimulatorConfig,
};
#[cfg(all(feature = "iot", feature = "simulator"))]
use std::collections::HashMap;
#[cfg(all(feature = "iot", feature = "simulator"))]
use std::path::PathBuf;
#[cfg(all(feature = "iot", feature = "simulator"))]
use tokio::time::{sleep, Duration};

/// # Programmatic Demo of Apicentric
///
/// This example demonstrates how to use the Apicentric library as an SDK to:
/// 1. Programmatically define and start an HTTP API Simulator.
/// 2. Programmatically define and run an IoT Digital Twin.
/// 3. Simulate interaction between the Twin and the API (telemetry reporting).
///
/// ## Features Covered:
/// - **API Simulator**: Dynamic service creation, endpoint mocking, request handling.
/// - **IoT**: Digital Twin state management, Physics strategies (Sine Wave).
/// - **Integration**: Connecting IoT physics to API endpoints.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "iot", feature = "simulator"))]
    {
        // 1. Initialize Logging
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        env_logger::init();

        println!("🚀 Starting Apicentric Programmatic Demo...");

        // ==================================================================================
        // PART 1: API SIMULATOR (HTTP)
        // ==================================================================================
        println!("\n📦 [1/3] Setting up API Simulator...");

        // Configure the simulator
        // We use a temporary DB path to avoid conflicts
        let mut sim_config = SimulatorConfig::new(
            true,
            PathBuf::from("./services"), // Placeholder, we register manually
            PortRange {
                start: 9500,
                end: 9600,
            },
        );
        sim_config.db_path = PathBuf::from(":memory:"); // Use in-memory DB

        // Create the manager
        let manager = ApiSimulatorManager::new(sim_config);

        // Define a Service: "IoT Hub"
        // It will receive telemetry data from our digital twin.
        let mut responses_telemetry = HashMap::new();
        responses_telemetry.insert(
            200,
            ResponseDefinition {
                condition: None,
                content_type: "application/json".to_string(),
                // Use Handlebars to echo back the received data + timestamp
                body: r#"{"status": "received", "data": {{json request.body}}, "ts": "{{now}}"}"#
                    .to_string(),
                script: None,
                headers: None,
                side_effects: None,
                schema: None, // Added missing field
            },
        );

        let endpoint = EndpointDefinition {
            kind: EndpointKind::Http,
            method: "POST".to_string(),
            path: "/telemetry".to_string(),
            header_match: None,
            description: Some("Receive device telemetry".to_string()),
            parameters: None,
            request_body: None,
            responses: responses_telemetry,
            scenarios: None,
            stream: None,
        };

        let service_def = ServiceDefinition {
            name: "iot-hub".to_string(),
            version: Some("1.0".to_string()),
            description: Some("Mock IoT Hub API".to_string()),
            server: Some(ServerConfig {
                port: Some(9500),
                base_path: "/api/v1".to_string(),
                proxy_base_url: None,
                cors: None,
                record_unknown: false,
            }),
            models: None,
            fixtures: None,
            bucket: None,
            endpoints: Some(vec![endpoint]),
            graphql: None,
            behavior: None,
            #[cfg(feature = "iot")]
            twin: None,
        };

        // Register and Start the Service
        {
            let mut registry = manager.service_registry().write().await;
            registry.register_service(service_def).await?;
        }
        manager.start_service("iot-hub").await?;

        println!("✅ Service 'iot-hub' started at http://localhost:9500/api/v1");

        // ==================================================================================
        // PART 2: IOT DIGITAL TWIN
        // ==================================================================================
        println!("\n🤖 [2/3] Setting up IoT Digital Twin...");

        // Create a Digital Twin
        let mut twin = DigitalTwin::new("temp-sensor-01".to_string());

        // Add Physics Strategy: Sine Wave
        // Simulates a temperature fluctuating between 20.0 and 30.0
        // Note: In a real YAML config, this corresponds to `strategy: sine`
        let physics = SineWaveStrategy::new(
            "temperature".to_string(), // Variable name
            20.0,                      // Min
            30.0,                      // Max
            0.5,                       // Frequency
        );

        // ==================================================================================
        // PART 3: RUN AND INTERACT
        // ==================================================================================
        println!("\n🔄 [3/3] Running Simulation Loop (Twin -> API)...");

        // We'll run a custom loop here to demonstrate control
        let client = reqwest::Client::new();
        let steps = 5;

        for i in 1..=steps {
            // 1. Tick Physics
            // This updates the twin's state based on the strategy (time-based sine wave)
            physics.tick(&mut twin.state).await?;

            // 2. Read State
            if let Some(val) = twin.state.variables.get("temperature") {
                let temp: f64 = val.as_f64().unwrap_or(0.0);

                println!(
                    "   [Tick {}/{}] 🌡️  Device State: Temperature = {:.2}°C",
                    i, steps, temp
                );

                // 3. Report to API (Simulate Telemetry)
                let payload = serde_json::json!({
                    "device_id": twin.id,
                    "temperature": temp,
                    "unit": "celsius"
                });

                match client
                    .post("http://localhost:9500/api/v1/telemetry")
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();
                        let body = resp.text().await?;
                        println!(
                            "   [Tick {}/{}] ☁️  Cloud Response: {} - {}",
                            i, steps, status, body
                        );
                    }
                    Err(e) => println!("   [Tick {}/{}] ❌ Cloud Error: {}", i, steps, e),
                }
            }

            // Wait a bit
            sleep(Duration::from_secs(1)).await;
        }

        // Cleanup
        println!("\n🛑 Stopping services...");
        manager.stop_service("iot-hub").await?;

        println!("✨ Demo completed successfully!");
    }

    #[cfg(not(all(feature = "iot", feature = "simulator")))]
    {
        println!("This example requires 'iot' and 'simulator' features.");
    }

    Ok(())
}
