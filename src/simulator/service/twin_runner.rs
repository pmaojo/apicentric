use crate::errors::ApicentricResult;
use crate::iot::adapters::modbus::ModbusAdapter;
use crate::iot::adapters::mqtt::MqttAdapter;
use crate::iot::config::TwinDefinition;
use crate::iot::model::DigitalTwin;
use crate::iot::physics::scripting::RhaiScriptStrategy;
use crate::iot::physics::sine::SineWaveStrategy;
use crate::iot::traits::{ProtocolAdapter, SimulationStrategy};
use crate::simulator::service::ServiceInstance;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info};

impl ServiceInstance {
    /// Start the digital twin simulation loop
    pub async fn start_twin_runner(&mut self, twin_def: TwinDefinition) -> ApicentricResult<()> {
        let service_name = twin_def.name.clone();
        let state = Arc::clone(&self.state);

        let twin_runner_handle = tokio::spawn(async move {
            info!(target: "simulator", service = %service_name, "🚀 Starting Digital Twin runner");

            let mut twin = DigitalTwin::new(service_name.clone());
            let mut adapters: Vec<Box<dyn ProtocolAdapter>> = Vec::new();
            let mut strategies: Vec<Box<dyn SimulationStrategy>> = Vec::new();

            // Initialize Adapters
            for transport in &twin_def.transports {
                match transport.adapter_type.as_str() {
                    "mqtt" => {
                        let mut adapter = MqttAdapter::new();
                        if let Err(e) = adapter.init(transport).await {
                            error!(target: "simulator", "Failed to init MQTT adapter: {}", e);
                            continue;
                        }
                        adapters.push(Box::new(adapter));
                    }
                    "modbus" => {
                        let mut adapter = ModbusAdapter::new();
                        if let Err(e) = adapter.init(transport).await {
                            error!(target: "simulator", "Failed to init Modbus adapter: {}", e);
                            continue;
                        }
                        adapters.push(Box::new(adapter));
                    }
                    _ => {
                        error!(target: "simulator", "Unknown transport type: {}", transport.adapter_type)
                    }
                }
            }

            // Initialize Physics Strategies
            for physics in &twin_def.physics {
                match physics.strategy.as_str() {
                    "script" => {
                        if let Some(script_val) = physics.params.get("code") {
                            if let Some(script) = script_val.as_str() {
                                match RhaiScriptStrategy::new(script, physics.variable.clone()) {
                                    Ok(strategy) => strategies.push(Box::new(strategy)),
                                    Err(e) => {
                                        error!(target: "simulator", "Failed to init Rhai strategy: {}", e)
                                    }
                                }
                            }
                        }
                    }
                    "sine" | "noise_sine" => {
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
                        let frequency = 0.1;
                        let strategy =
                            SineWaveStrategy::new(physics.variable.clone(), min, max, frequency);
                        strategies.push(Box::new(strategy));
                    }
                    _ => {
                        error!(target: "simulator", "Unknown physics strategy: {}", physics.strategy)
                    }
                }
            }

            // Main Simulation Loop
            let tick_rate = Duration::from_millis(1000);
            let mut next_tick = Instant::now();

            loop {
                // 1. Tick Physics
                for strategy in &mut strategies {
                    if let Err(e) = strategy.tick(&mut twin.state).await {
                        error!(target: "simulator", "Simulation tick error: {}", e);
                    }
                }

                // 2. Publish State to Adapters
                for (key, value) in &twin.state.variables {
                    for adapter in &adapters {
                        let _ = adapter.publish(key, value).await;
                    }
                }

                // 3. Emit Telemetry Log (so dashboard sees activity)
                let payload = serde_json::to_string(&twin.state.variables).ok();

                Self::record_log(
                    &state,
                    &service_name,
                    None,
                    "TICK",
                    "/telemetry",
                    200,
                    payload,
                )
                .await;

                // 4. Precise Timing
                next_tick += tick_rate;
                let now = Instant::now();
                if next_tick > now {
                    sleep(next_tick - now).await;
                } else {
                    next_tick = now;
                }
            }
        });

        self.twin_handle = Some(twin_runner_handle);
        self.is_running = true;
        Ok(())
    }
}
