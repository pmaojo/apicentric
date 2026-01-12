use apicentric::iot::adapters::modbus::ModbusAdapter;
use apicentric::iot::adapters::mqtt::MqttAdapter;
use apicentric::iot::args::TwinRunArgs;
use apicentric::iot::config::TwinConfig;
use apicentric::iot::model::DigitalTwin;
use apicentric::iot::physics::scripting::RhaiScriptStrategy;
use apicentric::iot::physics::sine::SineWaveStrategy;
use apicentric::iot::traits::{ProtocolAdapter, SimulationStrategy};
use log::{error, info};
use std::path::Path;
use tokio::time::{sleep, Duration, Instant};

pub async fn run(args: TwinRunArgs) -> anyhow::Result<()> {
    info!("Starting Digital Twin simulation...");

    // Load device definition
    let device_path = Path::new(&args.device);
    let config_content = tokio::fs::read_to_string(device_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read device file '{}': {}", args.device, e))?;

    let config: TwinConfig = serde_yaml::from_str(&config_content)?;

    info!("Loaded twin definition: {}", config.twin.name);

    let mut twin = DigitalTwin::new(config.twin.name.clone());
    let mut adapters: Vec<Box<dyn ProtocolAdapter>> = Vec::new();
    let mut strategies: Vec<Box<dyn SimulationStrategy>> = Vec::new();

    // Initialize Adapters
    for transport in &config.twin.transports {
        match transport.adapter_type.as_str() {
            "mqtt" => {
                let mut adapter = MqttAdapter::new();
                adapter.init(transport).await?;
                adapters.push(Box::new(adapter));
            }
            "modbus" => {
                let mut adapter = ModbusAdapter::new();
                adapter.init(transport).await?;
                adapters.push(Box::new(adapter));
            }
            _ => error!("Unknown transport type: {}", transport.adapter_type),
        }
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
            _ => error!("Unknown physics strategy: {}", physics.strategy),
        }
    }

    // Main Simulation Loop
    info!("Entering simulation loop...");
    let tick_rate = Duration::from_millis(1000);
    let mut next_tick = Instant::now();

    loop {
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
