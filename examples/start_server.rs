use apicentric::cloud::CloudServer;
use apicentric::simulator::config::{PortRange, SimulatorConfig};
use apicentric::simulator::ApiSimulatorManager;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create services directory if it doesn't exist
    std::fs::create_dir_all("./services")?;

    // Create simulator manager with a SimulatorConfig
    let services_dir = PathBuf::from("./services");
    let config = SimulatorConfig::new(
        true,
        services_dir.clone(),
        PortRange {
            start: 9000,
            end: 9099,
        },
    );
    let manager = ApiSimulatorManager::new(config);

    // Create and start cloud server
    let server = CloudServer::new(manager);

    println!("🚀 Starting Apicentric Cloud Server on port 8080...");
    server.start(8080).await?;

    Ok(())
}
