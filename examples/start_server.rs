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

        // Create simulator config
        let mut config = SimulatorConfig {
            services_dir: PathBuf::from("./services"),
            db_path: PathBuf::from("apicentric.db"),
            ..Default::default()
        };
        config.enabled = true;

        // Create simulator manager
        let manager = ApiSimulatorManager::new(config);

        // Start simulator (loads services)
        manager.start().await?;

        // Create and start cloud server
        let server = CloudServer::new(manager);

        println!("🚀 Starting Apicentric Server on port 8080...");
        server.start(8080).await?;
    }

    #[cfg(not(feature = "webui"))]
    {
        println!("This example requires the 'webui' feature.");
    }

    Ok(())
}
