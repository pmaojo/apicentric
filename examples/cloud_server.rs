#[cfg(feature = "webui")]
use apicentric::cloud::CloudServer;
#[cfg(feature = "webui")]
use apicentric::simulator::{config::SimulatorConfig, ApiSimulatorManager};
#[cfg(feature = "webui")]
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "webui")]
    {
        // Initialize logging
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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

        server.start(8080).await?;
    }

    #[cfg(not(feature = "webui"))]
    {
        println!("This example requires the 'webui' feature.");
    }

    Ok(())
}
