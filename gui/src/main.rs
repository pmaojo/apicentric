#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::State;
use pulse::simulator::{ApiSimulatorManager, SimulatorConfig, ServiceInfo, ServiceDefinition};
use std::path::PathBuf;

struct SimulatorState(Arc<ApiSimulatorManager>);

#[tauri::command]
async fn start_simulator(state: State<'_, SimulatorState>) -> Result<(), String> {
    state.0.start().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_simulator(state: State<'_, SimulatorState>) -> Result<(), String> {
    state.0.stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_services(state: State<'_, SimulatorState>) -> Result<Vec<ServiceInfo>, String> {
    Ok(state.0.get_status().await.active_services)
}

#[tauri::command]
fn load_service(path: String) -> Result<String, String> {
    let p = PathBuf::from(path);
    let file = std::fs::File::open(p).map_err(|e| e.to_string())?;
    let def: ServiceDefinition = serde_yaml::from_reader(file).map_err(|e| e.to_string())?;
    serde_yaml::to_string(&def).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_service(path: String, yaml: String) -> Result<(), String> {
    let def: ServiceDefinition = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    serde_yaml::to_writer(file, &def).map_err(|e| e.to_string())
}

fn main() {
    let cfg = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(cfg);
    tauri::Builder::default()
        .manage(SimulatorState(Arc::new(manager)))
        .invoke_handler(tauri::generate_handler![start_simulator, stop_simulator, list_services, load_service, save_service])
        .run(tauri::generate_context!())
        .expect("error while running Pulse GUI");
}
