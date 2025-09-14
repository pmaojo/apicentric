#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::State;
use mockforge::simulator::{ApiSimulatorManager, SimulatorConfig, ServiceInfo, ServiceDefinition};
use mockforge::simulator::log::RequestLogEntry;
use mockforge::collab::share;
use libp2p::PeerId;
use std::path::PathBuf;
use std::str::FromStr;

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

#[tauri::command]
fn export_types(path: String) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let def: ServiceDefinition = serde_yaml::from_reader(file).map_err(|e| e.to_string())?;
    mockforge::simulator::typescript::to_typescript(&def).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_logs(
    service: String,
    limit: Option<usize>,
    state: State<'_, SimulatorState>,
) -> Result<Vec<RequestLogEntry>, String> {
    let registry = state.0.service_registry().read().await;
    if let Some(instance) = registry.get_service(&service) {
        Ok(instance.read().await.get_logs(limit.unwrap_or(100)))
    } else {
        Err(format!("Service '{}' not found", service))
    }
}

#[tauri::command]
async fn share_service(
    service: String,
    state: State<'_, SimulatorState>,
) -> Result<(String, String), String> {
    let registry = state.0.service_registry().read().await;
    if let Some(instance) = registry.get_service(&service) {
        let port = instance.read().await.port();
        drop(registry);
        share::share_service(port)
            .await
            .map(|(peer, token)| (peer.to_string(), token))
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Service '{}' not found", service))
    }
}

#[tauri::command]
async fn connect_service(
    peer: String,
    token: String,
    service: String,
    port: u16,
) -> Result<(), String> {
    let peer_id = PeerId::from_str(&peer).map_err(|e| e.to_string())?;
    share::connect_service(peer_id, token, service, port)
        .await
        .map_err(|e| e.to_string())
}

fn main() {
    let cfg = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(cfg);
    tauri::Builder::default()
        .manage(SimulatorState(Arc::new(manager)))
        .invoke_handler(tauri::generate_handler![
            start_simulator,
            stop_simulator,
            list_services,
            load_service,
            save_service,
            export_types,
            get_logs,
            share_service,
            connect_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running MockForge GUI");
}
