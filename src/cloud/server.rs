//! The cloud server.
//!
//! This module provides a `CloudServer` that can be used to serve the cloud API.

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

use super::handlers;
#[cfg(feature = "iot")]
use super::iot_handlers;
use crate::auth::jwt::JwtKeys;
use crate::auth::repository::AuthRepository;
use crate::auth::{handlers as auth_handlers, handlers::AuthState};
use crate::cloud::cors::create_cors_layer;
use crate::cloud::recording_session::RecordingSessionManager;
use crate::cloud::websocket::{ws_handler, WebSocketState};
use crate::simulator::ApiSimulatorManager;
use std::env;

/// The cloud server.
pub struct CloudServer {
    simulator_manager: Arc<ApiSimulatorManager>,
    auth_state: Arc<AuthState>,
    recording_manager: Arc<RecordingSessionManager>,
    websocket_state: Arc<WebSocketState>,
    protect_services: bool,
}

impl CloudServer {
    /// Creates a new `CloudServer`.
    ///
    /// # Arguments
    ///
    /// * `simulator_manager` - The API simulator manager.
    pub fn new(simulator_manager: ApiSimulatorManager) -> Self {
        // Initialize auth state (temporary simple sqlite file for users)
        let db_path = env::var("APICENTRIC_AUTH_DB").unwrap_or_else(|_| "data/auth.db".to_string());
        std::fs::create_dir_all("data").ok();
        let repo = AuthRepository::new(&db_path).expect("Failed to init auth repository");
        let secret =
            env::var("APICENTRIC_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
        let keys = JwtKeys::from_secret(&secret);
        let blacklist = crate::auth::blacklist::TokenBlacklist::new();
        let auth_state = Arc::new(AuthState {
            repo: Arc::new(repo),
            keys,
            blacklist,
        });
        let recording_manager = Arc::new(RecordingSessionManager::new());
        let simulator_manager_arc = Arc::new(simulator_manager);
        let websocket_state = Arc::new(WebSocketState::new(Arc::clone(&simulator_manager_arc)));
        let protect_services = env::var("APICENTRIC_PROTECT_SERVICES")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        Self {
            simulator_manager: simulator_manager_arc,
            auth_state,
            recording_manager,
            websocket_state,
            protect_services,
        }
    }

    /// Starts the cloud server.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on.
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_router();

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;

        println!("🚀 Apicentric Cloud Server listening on http://{}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    fn create_router(&self) -> Router {
        // Public routes (no authentication required)
        let public_routes = Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            // Legacy simulator status endpoints (for backward compatibility)
            .route("/status", get(handlers::get_simulator_status))
            .route("/start", post(handlers::start_simulator))
            .route("/stop", post(handlers::stop_simulator))
            // Auth endpoints (public by nature)
            .route("/api/auth/register", post(auth_handlers::register))
            .route("/api/auth/login", post(auth_handlers::login));

        // Protected routes (may require authentication based on config)
        let mut protected_routes = Router::new()
            // Auth endpoints that require existing token
            .route("/api/auth/me", get(auth_handlers::me))
            .route("/api/auth/refresh", post(auth_handlers::refresh))
            .route("/api/auth/logout", post(auth_handlers::logout))
            // WebSocket endpoint for real-time updates
            .route("/ws", get(ws_handler))
            // API routes
            .route(
                "/api/services",
                get(handlers::list_services).post(handlers::create_service),
            )
            // Marketplace and Import routes
            .route("/api/marketplace", get(handlers::marketplace_list))
            .route("/api/import/url", post(handlers::import_from_url))
            .route("/api/services/load", post(handlers::load_service))
            .route("/api/services/save", post(handlers::save_service))
            .route("/api/services/reload", post(handlers::reload_services))
            .route(
                "/api/services/:name",
                get(handlers::get_service)
                    .put(handlers::update_service)
                    .delete(handlers::delete_service),
            )
            .route("/api/services/:name/start", post(handlers::start_service))
            .route("/api/services/:name/stop", post(handlers::stop_service))
            .route(
                "/api/services/:name/status",
                get(handlers::get_service_status),
            )
            // Log routes
            .route(
                "/api/logs",
                get(handlers::query_logs).delete(handlers::clear_logs),
            )
            .route("/api/logs/export", get(handlers::export_logs))
            // Recording routes
            .route("/api/recording/start", post(handlers::start_recording))
            .route("/api/recording/stop", post(handlers::stop_recording))
            .route("/api/recording/status", get(handlers::get_recording_status))
            .route(
                "/api/recording/generate",
                post(handlers::generate_service_from_recording),
            )
            // AI generation routes
            .route("/api/ai/generate", post(handlers::ai_generate))
            .route("/api/ai/validate", post(handlers::ai_validate))
            .route("/api/ai/config", get(handlers::ai_config_status))
            // Code generation routes
            .route(
                "/api/codegen/typescript",
                post(handlers::generate_typescript),
            )
            .route(
                "/api/codegen/react-query",
                post(handlers::generate_react_query),
            )
            .route("/api/codegen/axios", post(handlers::generate_axios))
            // Configuration management routes
            .route(
                "/api/config",
                get(handlers::get_config).put(handlers::update_config),
            )
            .route("/api/config/validate", post(handlers::validate_config))
            // Monitoring and metrics routes
            .route("/api/metrics", get(handlers::get_metrics));

        // Add IoT routes if feature enabled
        #[cfg(feature = "iot")]
        {
            protected_routes = protected_routes.route("/api/v1/iot/graph", get(iot_handlers::get_iot_graph));
        }

        // Apply authentication middleware to protected routes if enabled
        let protected_routes = if self.protect_services {
            let auth_state = Arc::clone(&self.auth_state);
            protected_routes.layer(axum::middleware::from_fn(move |req, next| {
                let auth_state = Arc::clone(&auth_state);
                async move { crate::auth::middleware::require_auth(auth_state, req, next).await }
            }))
        } else {
            protected_routes
        };

        // Combine routes
        let mut router = Router::new().merge(public_routes).merge(protected_routes);

        // Serve static files (the Next.js frontend) if available
        // Try multiple possible locations for the frontend build
        let frontend_paths = vec![
            "webui/.next/standalone", // Docker build
            "webui/out",              // Static export
            "webui/dist",             // Alternative build
        ];

        for path in frontend_paths {
            if std::path::Path::new(path).exists() {
                println!("📁 Serving frontend from: {}", path);
                router = router.nest_service("/", ServeDir::new(path));
                break;
            }
        }

        router
            // Middleware with environment-based CORS configuration
            .layer(
                ServiceBuilder::new()
                    .layer(create_cors_layer())
                    .into_inner(),
            )
            // Share the simulator manager state across all handlers
            .with_state(Arc::clone(&self.simulator_manager))
            .layer(axum::Extension(Arc::clone(&self.auth_state)))
            .layer(axum::Extension(Arc::clone(&self.recording_manager)))
            .layer(axum::Extension(Arc::clone(&self.websocket_state)))
    }
}

/// The health check endpoint.
async fn health_check() -> Json<serde_json::Value> {
    use std::time::SystemTime;

    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Json(serde_json::json!({
        "status": "healthy",
        "service": "apicentric-cloud",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": uptime,
    }))
}
