use apicentric::cloud::CloudServer;
use apicentric::simulator::{config::SimulatorConfig, ApiSimulatorManager};
use apicentric::{ApicentricError, ApicentricResult};
use std::path::PathBuf;

/// Launch the Apicentric Cloud API Server
pub async fn cloud_command() -> ApicentricResult<()> {
    // Create services directory if it doesn't exist
    std::fs::create_dir_all("./services")?;

    // Create simulator config
    let config = SimulatorConfig {
        services_dir: PathBuf::from("./services"),
        db_path: PathBuf::from("apicentric.db"),
        ..Default::default()
    };

    // Create simulator manager
    let manager = ApiSimulatorManager::new(config);

    // Create and start cloud server
    let server = CloudServer::new(manager);

    println!("🚀 Starting Apicentric Cloud Server on port 8080...");
    println!("📝 API Documentation: http://localhost:8080/health");
    println!(
        "🔐 Authentication is optional (set APICENTRIC_PROTECT_SERVICES=true to require auth)"
    );

    if let Err(e) = server.start(8080).await {
        return Err(ApicentricError::runtime_error(
            format!("Failed to start cloud server: {}", e),
            Some("Check if port 8080 is already in use or if there are permission issues"),
        ));
    }

    Ok(())
}
