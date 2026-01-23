use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig, config::PortRange};
use apicentric::ai::{AiProvider, OpenAiProvider};
use std::sync::Arc;
use tokio::sync::RwLock;
use lazy_static::lazy_static;
use std::path::PathBuf;
use flutter_rust_bridge::frb;

// Types to expose to Dart
pub struct ServiceStatus {
    pub name: String,
    pub port: u16,
    pub is_active: bool,
}

lazy_static! {
    static ref SIMULATOR: RwLock<Option<Arc<ApiSimulatorManager>>> = RwLock::new(None);
}

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub async fn start_simulator(services_dir: String, db_path: String) -> anyhow::Result<()> {
    let mut config = SimulatorConfig::default();
    config.enabled = true;
    config.services_dir = PathBuf::from(services_dir);
    config.db_path = PathBuf::from(db_path);
    config.port_range = PortRange { start: 9000, end: 9999 };

    let manager = ApiSimulatorManager::new(config);
    manager.start().await?;

    let mut sim_lock = SIMULATOR.write().await;
    *sim_lock = Some(Arc::new(manager));
    Ok(())
}

pub async fn stop_simulator() -> anyhow::Result<()> {
    let mut sim_lock = SIMULATOR.write().await;
    if let Some(manager) = sim_lock.take() {
        manager.stop().await?;
    }
    Ok(())
}

pub async fn get_active_services() -> anyhow::Result<Vec<ServiceStatus>> {
    let sim_lock = SIMULATOR.read().await;
    if let Some(manager) = sim_lock.as_ref() {
        let status = manager.get_status().await;
        Ok(status.active_services.iter().map(|s| ServiceStatus {
            name: s.name.clone(),
            port: s.port,
            is_active: s.is_running,
        }).collect())
    } else {
        Ok(vec![])
    }
}

pub async fn generate_service_ai(api_key: String, prompt: String, model: String) -> anyhow::Result<String> {
     let provider = OpenAiProvider::new(api_key, model);
     let yaml = provider.generate_yaml(&prompt).await?;
     Ok(yaml)
}

pub async fn save_service_definition(path: String, content: String) -> anyhow::Result<()> {
    tokio::fs::write(path, content).await?;
    Ok(())
}

// Simple stream that emits logs
// In a real app, we would hook this into the simulator's log broadcast
pub fn create_log_stream(sink: flutter_rust_bridge::StreamSink<String>) -> anyhow::Result<()> {
    // We spawn a task to poll logs
    tokio::spawn(async move {
        // We need to wait for simulator to be available
        // Polling loop
        let mut attempts = 0;
        let rx_option = loop {
             {
                 let sim_lock = SIMULATOR.read().await;
                 if let Some(manager) = sim_lock.as_ref() {
                     break Some(manager.subscribe_logs());
                 }
             }
             tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
             attempts += 1;
             if attempts > 20 { break None; } // Give up after 10s if simulator not started
        };

        if let Some(mut rx) = rx_option {
            while let Ok(entry) = rx.recv().await {
                 let log_line = format!("[{}] {} {} -> {}", entry.timestamp, entry.method, entry.path, entry.status);
                 if sink.add(log_line).is_err() {
                     break;
                 }
            }
        }
    });
    Ok(())
}
