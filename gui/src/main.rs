#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::{Manager, State};
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig, ServiceInfo, ServiceDefinition};
use apicentric::simulator::log::RequestLogEntry;
use apicentric::collab::share;
use libp2p::PeerId;
use std::path::PathBuf;
use std::str::FromStr;

struct SimulatorState(Arc<ApiSimulatorManager>);

#[tauri::command]
fn start_simulator(state: State<'_, SimulatorState>) -> Result<(), String> {
    tauri::async_runtime::block_on(state.0.start()).map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_simulator(state: State<'_, SimulatorState>) -> Result<(), String> {
    tauri::async_runtime::block_on(state.0.stop()).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_services(state: State<'_, SimulatorState>) -> Result<Vec<ServiceInfo>, String> {
    Ok(tauri::async_runtime::block_on(state.0.get_status()).active_services)
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
    apicentric::simulator::typescript::to_typescript(&def).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_logs(
    service: String,
    limit: Option<usize>,
    state: State<'_, SimulatorState>,
) -> Result<Vec<RequestLogEntry>, String> {
    tauri::async_runtime::block_on(async {
        let registry = state.0.service_registry().read().await;
        if let Some(instance) = registry.get_service(&service) {
            Ok(instance.read().await.get_logs(limit.unwrap_or(100)).await)
        } else {
            Err(format!("Service '{}' not found", service))
        }
    })
}

#[tauri::command]
fn share_service(
    service: String,
    state: State<'_, SimulatorState>,
) -> Result<(String, String), String> {
    tauri::async_runtime::block_on(async {
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
    })
}

#[tauri::command]
fn connect_service(
    peer: String,
    token: String,
    service: String,
    port: u16,
) -> Result<(), String> {
    tauri::async_runtime::block_on(async {
        let peer_id = PeerId::from_str(&peer).map_err(|e| e.to_string())?;
        share::connect_service(peer_id, token, service, port)
            .await
            .map_err(|e| e.to_string())
    })
}

fn main() {
    let cfg = SimulatorConfig::default_config();
    let manager = ApiSimulatorManager::new(cfg);
    tauri::Builder::default()
        .manage(SimulatorState(Arc::new(manager)))
        .setup(|app| {
            let handle = app.handle();
            let state = app.state::<SimulatorState>().0.clone();
            let mut rx = state.subscribe_logs();
            tauri::async_runtime::spawn(async move {
                while let Ok(entry) = rx.recv().await {
                    let _ = handle.emit_all("log", entry);
                }
            });
            Ok(())
        })
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
        .expect("error while running Apicentric GUI");
}
