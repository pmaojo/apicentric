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

pub struct CloudServer {
    simulator_manager: Arc<ApiSimulatorManager>,
}

impl CloudServer {
    pub fn new(simulator_manager: ApiSimulatorManager) -> Self {
        Self {
            simulator_manager: Arc::new(simulator_manager),
        }
    }

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
        Router::new()
            // Health check endpoint
            .route("/health", get(health_check))
            
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
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "apicentric-cloud",
        "version": env!("CARGO_PKG_VERSION")
    }))
}