//! Admin server for the API simulator.
use crate::simulator::registry::ServiceRegistry;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<ServiceRegistry>>,
    admin_token: Option<String>,
}

pub struct AdminServer {
    service_registry: Arc<RwLock<ServiceRegistry>>,
    server_handle: Option<JoinHandle<()>>,
    admin_token: Option<String>,
}

impl AdminServer {
    pub fn new(service_registry: Arc<RwLock<ServiceRegistry>>) -> Self {
        let admin_token = std::env::var("APICENTRIC_ADMIN_TOKEN").ok();
        Self {
            service_registry,
            server_handle: None,
            admin_token,
        }
    }

    pub async fn start(&mut self, port: u16) {
        let state = AppState {
            registry: self.service_registry.clone(),
            admin_token: self.admin_token.clone(),
        };

        let app = Router::new()
            .route("/apicentric-admin/logs", get(get_logs))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind admin server to {}: {}", addr, e);
                return;
            }
        };

        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Admin server error: {}", e);
            }
        });

        self.server_handle = Some(handle);
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

async fn get_logs(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // Auth check
    if let Some(token) = &state.admin_token {
        let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());

        match auth_header {
            Some(header_val) if header_val.starts_with("Bearer ") => {
                let provided_token = header_val.trim_start_matches("Bearer ");
                if provided_token != token {
                    return (StatusCode::FORBIDDEN, "Forbidden").into_response();
                }
            }
            _ => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
        }
    }

    // Step 1: Get list of services
    let services = {
        let registry = state.registry.read().await;
        registry.list_services().await
    };

    let mut all_logs = vec![];

    // Step 2: Iterate and get logs
    for service_info in services {
        let service_opt = {
            let registry = state.registry.read().await;
            registry.get_service(&service_info.name).cloned()
        };

        if let Some(service_arc) = service_opt {
            let service = service_arc.read().await;
            let logs = service.get_logs(100).await;
            all_logs.extend(logs);
        }
    }

    Json(all_logs).into_response()
}
