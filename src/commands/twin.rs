use apicentric::iot::adapters::modbus::ModbusAdapter;
use apicentric::iot::adapters::mqtt::MqttAdapter;
use apicentric::iot::args::TwinRunArgs;
use apicentric::iot::config::TwinConfig;
use apicentric::iot::model::DigitalTwin;
use apicentric::iot::physics::replay::ReplayStrategy;
use apicentric::iot::physics::scripting::RhaiScriptStrategy;
use apicentric::iot::physics::sine::SineWaveStrategy;
use apicentric::iot::traits::{ProtocolAdapter, SimulationStrategy};
use apicentric::errors::{ApicentricResult, ApicentricError};
use log::{error, info};
use std::path::Path;
use tokio::time::{sleep, Duration, Instant};

pub async fn run(args: TwinRunArgs) -> ApicentricResult<()> {
    info!("Starting Digital Twin simulation...");

    // Load device definition
    let device_path = Path::new(&args.device);
    let config_content = tokio::fs::read_to_string(device_path)
        .await
        .map_err(|e| ApicentricError::FileSystem {
            message: format!("Failed to read device file '{}': {}", args.device, e),
            suggestion: Some("Check if the file exists and is readable".to_string())
        })?;

    let config: TwinConfig = serde_yaml::from_str(&config_content).map_err(|e| ApicentricError::Configuration {
        message: format!("Failed to parse device config: {}", e),
        suggestion: Some("Check YAML syntax".to_string())
    })?;

    info!("Loaded twin definition: {}", config.twin.name);

    let mut twin = DigitalTwin::new(config.twin.name.clone());
    let mut adapters: Vec<Box<dyn ProtocolAdapter>> = Vec::new();
    let mut strategies: Vec<Box<dyn SimulationStrategy>> = Vec::new();

    // Initialize Adapters
    for transport in &config.twin.transports {
        let mut adapter: Box<dyn ProtocolAdapter> = match transport.adapter_type.as_str() {
            "mqtt" => Box::new(MqttAdapter::new()),
            "modbus" => Box::new(ModbusAdapter::new()),
            _ => {
                error!("Unknown transport type: {}", transport.adapter_type);
                continue;
            }
        };

        adapter.init(transport).await?;

        // Handle subscriptions
        if let Some(subs) = transport.params.get("subscriptions") {
            if let Some(subs_seq) = subs.as_sequence() {
                for sub in subs_seq {
                    if let Some(topic) = sub.as_str() {
                        adapter.subscribe(topic).await?;
                    }
                }
            }
        }

        adapters.push(adapter);
    }

    // Initialize Physics Strategies
    for physics in &config.twin.physics {
        match physics.strategy.as_str() {
            "script" => {
                if let Some(script_val) = physics.params.get("code") {
                    if let Some(script) = script_val.as_str() {
                        let strategy = RhaiScriptStrategy::new(script, physics.variable.clone())?;
                        strategies.push(Box::new(strategy));
                    }
                }
            }
            "sine" | "noise_sine" => {
                // For now, map noise_sine to sine for simple demo
                let min = physics
                    .params
                    .get("min")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let max = physics
                    .params
                    .get("max")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(100.0);
                let frequency = 0.1; // Default Hz

                let strategy = SineWaveStrategy::new(physics.variable.clone(), min, max, frequency);
                strategies.push(Box::new(strategy));
            }
            "replay" => {
                let file_name = physics
                    .params
                    .get("file")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ApicentricError::Configuration {
                            message: "Missing 'file' parameter for replay strategy".to_string(),
                            suggestion: Some("Add 'file: path/to/data.csv' to params".to_string())
                        }
                    })?;

                let column = physics
                    .params
                    .get("column")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let loop_data = physics
                    .params
                    .get("loop")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                // Resolve file path relative to the device config file
                let config_dir = device_path.parent().unwrap_or(Path::new("."));
                let csv_path = config_dir.join(file_name);

                let strategy =
                    ReplayStrategy::new(&csv_path, physics.variable.clone(), column, loop_data)?;
                strategies.push(Box::new(strategy));
            }
            _ => error!("Unknown physics strategy: {}", physics.strategy),
        }
    }

    // Main Simulation Loop
    info!("Entering simulation loop...");
    let tick_rate = Duration::from_millis(1000);
    let mut next_tick = Instant::now();

    loop {
        // 0. Poll Adapters for new data
        for adapter in &mut adapters {
            while let Some((key, value)) = adapter.poll().await {
                twin.state.variables.insert(key, value);
            }
        }

        // 1. Tick Physics
        for strategy in &mut strategies {
            if let Err(e) = strategy.tick(&mut twin.state).await {
                error!("Simulation tick error: {}", e);
            }
        }

        // 2. Publish State to Adapters
        for (key, value) in &twin.state.variables {
            for adapter in &adapters {
                if let Err(_e) = adapter.publish(key, value).await {
                    // Log but don't crash
                }
            }
        }

        // 3. Precise Timing
        next_tick += tick_rate;
        if let Some(wait_time) = next_tick.checked_duration_since(Instant::now()) {
            sleep(wait_time).await;
        } else {
            // We are lagging, catch up immediately
            next_tick = Instant::now();
        }
    }
}
