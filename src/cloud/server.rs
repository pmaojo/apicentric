//! The cloud server.
//!
//! This module provides a `CloudServer` that can be used to serve the cloud API.

use std::sync::Arc;
use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::simulator::ApiSimulatorManager;
use super::handlers;
use crate::auth::{handlers as auth_handlers, handlers::AuthState, AuthUser};
use std::env;
use crate::auth::jwt::JwtKeys;
use crate::auth::repository::AuthRepository;

/// The cloud server.
pub struct CloudServer {
    simulator_manager: Arc<ApiSimulatorManager>,
    auth_state: Arc<AuthState>,
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
        let secret = env::var("APICENTRIC_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
        let keys = JwtKeys::from_secret(&secret);
        let auth_state = AuthState { repo: Arc::new(repo), keys };
        let protect_services = env::var("APICENTRIC_PROTECT_SERVICES").map(|v| v == "true" || v == "1").unwrap_or(false);
        Self {
            simulator_manager: Arc::new(simulator_manager),
            auth_state: Arc::new(auth_state),
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
        
        axum::serve(listener, app)
            .await?;
            
        Ok(())
    }

    fn create_router(&self) -> Router {
        let mut base = Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            // Auth endpoints
            .route("/api/auth/register", post(auth_handlers::register))
            .route("/api/auth/login", post(auth_handlers::login))
            .route("/api/auth/me", get(auth_handlers::me))
            
            // API routes
            .route("/api/services", get(handlers::list_services))
            .route("/api/services/load", post(handlers::load_service))
            .route("/api/services/save", post(handlers::save_service))
            
            // Serve static files (the React frontend)
            .nest_service("/", ServeDir::new("gui/dist"))
            
            // Middleware
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive()) // For development
                    .into_inner(),
            )
            
            // Share the simulator manager state across all handlers
            .with_state(Arc::clone(&self.simulator_manager))
            .layer(axum::Extension(Arc::clone(&self.auth_state)));

        if self.protect_services {
            use axum::middleware::from_fn;
            use axum::{http::StatusCode, body::Body};
            async fn require_auth(req: axum::http::Request<Body>, next: axum::middleware::Next) -> Result<axum::response::Response, StatusCode> {
                let headers = req.headers();
                let auth = headers.get(axum::http::header::AUTHORIZATION).and_then(|h| h.to_str().ok()).ok_or(StatusCode::UNAUTHORIZED)?;
                if !auth.starts_with("Bearer ") { return Err(StatusCode::UNAUTHORIZED); }
                Ok(next.run(req).await)
            }
            base = base.route_layer(from_fn(require_auth));
        }
        base
    }
}

/// The health check endpoint.
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "apicentric-cloud",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
