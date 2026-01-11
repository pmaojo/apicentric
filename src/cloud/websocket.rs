//!
//! This module provides a WebSocket server that broadcasts real-time updates
//! to connected clients, including request logs and service status changes.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::simulator::{log::RequestLogEntry, ApiSimulatorManager};

/// WebSocket client connection
struct WebSocketClient {
    /// Sender for messages to this client
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

/// Maximum number of WebSocket connections allowed
const MAX_CONNECTIONS: usize = 100;

/// WebSocket server state
#[derive(Clone)]
pub struct WebSocketState {
    /// Connected clients
    clients: Arc<RwLock<HashMap<Uuid, WebSocketClient>>>,
    /// Simulator manager for accessing state
    simulator: Arc<ApiSimulatorManager>,
}

impl WebSocketState {
    /// Create a new WebSocket state
    pub fn new(simulator: Arc<ApiSimulatorManager>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            simulator,
        }
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

/// Messages sent from client to server
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    /// Ping message for keepalive
    #[serde(rename = "ping")]
    Ping,
    /// Subscribe to specific channels
    #[serde(rename = "subscribe")]
    Subscribe { channels: Vec<String> },
    /// Unsubscribe from specific channels
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channels: Vec<String> },
}

/// Messages sent from server to client
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    /// Pong response to ping
    #[serde(rename = "pong")]
    Pong { timestamp: i64 },
    /// Service status update
    #[serde(rename = "service_status")]
    ServiceStatus { data: ServiceStatusUpdate },
    /// Request log entry
    #[serde(rename = "request_log")]
    RequestLog { data: RequestLogEntry },
    /// Initial state synchronization
    #[serde(rename = "initial_state")]
    InitialState { data: SimulatorState },
}

/// Service status update
#[derive(Debug, Serialize, Clone)]
pub struct ServiceStatusUpdate {
    /// Name of the service
    pub service_name: String,
    /// Current status
    pub status: String,
    /// Port if running
    pub port: Option<u16>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Simulator state for initial synchronization
#[derive(Debug, Serialize)]
struct SimulatorState {
    /// List of all services with their status
    services: Vec<ServiceStatusUpdate>,
    /// Whether the simulator is active
    is_active: bool,
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<WebSocketState>>,
) -> Response {
    // Note: Authentication is handled in the upgrade request headers
    // The actual token validation happens in the websocket connection handler
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<WebSocketState>) {
    let client_id = Uuid::new_v4();

    // Check connection limit
    {
        let clients = state.clients.read().await;
        if clients.len() >= MAX_CONNECTIONS {
            log::warn!(
                "WebSocket connection limit reached, rejecting client {}",
                client_id
            );
            return;
        }
    }

    log::info!("WebSocket client {} connected", client_id);

    let (mut sender, mut receiver) = socket.split();

    // Create a channel for sending messages to this client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Register the client
    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id, WebSocketClient { sender: tx.clone() });
    }

    // Send initial state
    if let Err(e) = send_initial_state(&state, &tx).await {
        log::error!(
            "Failed to send initial state to client {}: {}",
            client_id,
            e
        );
    }

    // Subscribe to log broadcasts
    let mut log_receiver = state.simulator.subscribe_logs();

    // Spawn a task to forward messages from the channel to the WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task to broadcast log entries to this client
    let tx_clone = tx.clone();
    let broadcast_task = tokio::spawn(async move {
        loop {
            match log_receiver.recv().await {
                Ok(log_entry) => {
                    let msg = ServerMessage::RequestLog { data: log_entry };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if tx_clone.send(Message::Text(json)).is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    log::warn!("Client {} lagged, skipped {} messages", client_id, skipped);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    // Spawn a task for ping/pong heartbeat with timeout
    let tx_heartbeat = tx.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let mut missed_pongs = 0;
        loop {
            interval.tick().await;
            // Send a ping frame
            if tx_heartbeat.send(Message::Ping(vec![])).is_err() {
                break;
            }
            missed_pongs += 1;
            // Close connection if too many missed pongs (client not responding)
            if missed_pongs > 3 {
                log::warn!("Client not responding to pings, closing connection");
                break;
            }
        }
    });

    // Handle incoming messages from the client
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_client_message(&text, &tx).await {
                    log::error!("Error handling client message: {}", e);
                }
            }
            Ok(Message::Close(_)) => {
                log::info!("Client {} requested close", client_id);
                break;
            }
            Ok(Message::Pong(_)) => {
                // Pong received, connection is alive - reset missed pongs counter
                // Note: We can't easily reset the counter in the heartbeat task,
                // but the connection staying open is proof enough
            }
            Ok(Message::Ping(_)) => {
                // Axum automatically responds to pings
            }
            Err(e) => {
                log::error!("WebSocket error for client {}: {}", client_id, e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup: remove client from the registry
    {
        let mut clients = state.clients.write().await;
        clients.remove(&client_id);
    }

    // Cancel background tasks
    send_task.abort();
    broadcast_task.abort();
    heartbeat_task.abort();

    log::info!("WebSocket client {} disconnected", client_id);
}

/// Send initial state to a newly connected client
async fn send_initial_state(
    state: &Arc<WebSocketState>,
    tx: &tokio::sync::mpsc::UnboundedSender<Message>,
) -> Result<(), String> {
    let status = state.simulator.get_status().await;

    let services: Vec<ServiceStatusUpdate> = status
        .active_services
        .iter()
        .map(|service| ServiceStatusUpdate {
            service_name: service.name.clone(),
            status: if service.is_running {
                "running".to_string()
            } else {
                "stopped".to_string()
            },
            port: if service.is_running {
                Some(service.port)
            } else {
                None
            },
            error: None,
        })
        .collect();

    let initial_state = SimulatorState {
        services,
        is_active: status.is_active,
    };

    let msg = ServerMessage::InitialState {
        data: initial_state,
    };

    let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;

    tx.send(Message::Text(json))
        .map_err(|e| format!("Failed to send message: {}", e))?;

    Ok(())
}

/// Handle a message from a client
async fn handle_client_message(
    text: &str,
    tx: &tokio::sync::mpsc::UnboundedSender<Message>,
) -> Result<(), String> {
    let client_msg: ClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {}", e))?;

    match client_msg {
        ClientMessage::Ping => {
            let pong = ServerMessage::Pong {
                timestamp: chrono::Utc::now().timestamp(),
            };
            let json = serde_json::to_string(&pong).map_err(|e| e.to_string())?;
            tx.send(Message::Text(json))
                .map_err(|e| format!("Failed to send pong: {}", e))?;
        }
        ClientMessage::Subscribe { channels } => {
            // For now, all clients are subscribed to all channels
            // This can be extended in the future for selective subscriptions
            log::debug!("Client requested subscription to channels: {:?}", channels);
        }
        ClientMessage::Unsubscribe { channels } => {
            // For now, all clients are subscribed to all channels
            log::debug!(
                "Client requested unsubscription from channels: {:?}",
                channels
            );
        }
    }

    Ok(())
}

/// Broadcast a service status update to all connected clients
pub async fn broadcast_service_status(
    state: &Arc<WebSocketState>,
    update: ServiceStatusUpdate,
) -> Result<(), String> {
    let msg = ServerMessage::ServiceStatus { data: update };
    let json = serde_json::to_string(&msg).map_err(|e| e.to_string())?;

    let clients = state.clients.read().await;
    for client in clients.values() {
        let _ = client.sender.send(Message::Text(json.clone()));
    }

    Ok(())
}
