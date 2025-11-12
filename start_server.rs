use apicentric::simulator::ApiSimulatorManager;
use apicentric::cloud::CloudServer;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create services directory if it doesn't exist
    std::fs::create_dir_all("./services")?;
    
    // Create simulator manager
    let services_dir = PathBuf::from("./services");
    let manager = ApiSimulatorManager::new(services_dir);
    
    // Create and start cloud server
    let server = CloudServer::new(manager);
    
    println!("🚀 Starting Apicentric Cloud Server on port 8080...");
    server.start(8080).await?;
    
    Ok(())
}
