use apicentric::cloud::CloudServer;
use apicentric::simulator::{ApiSimulatorManager, SimulatorConfig};
use clap::{Arg, Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    let matches = Command::new("apicentric-cloud")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Apicentric Cloud Server - API mocking and simulation in the cloud")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to listen on")
                .default_value("8080"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Host to bind to")
                .default_value("0.0.0.0"),
        )
        .get_matches();

    let port: u16 = matches
        .get_one::<String>("port")
        .unwrap()
        .parse()
        .expect("Invalid port number");

    println!("🌟 Starting Apicentric Cloud Server...");
    println!("📊 Version: {}", env!("CARGO_PKG_VERSION"));
    
    // Initialize the simulator manager with default config
    let config = SimulatorConfig::default_config();
    let simulator_manager = ApiSimulatorManager::new(config);
    
    // Create and start the cloud server
    let server = CloudServer::new(simulator_manager);
    
    if let Err(e) = server.start(port).await {
        eprintln!("❌ Failed to start server: {}", e);
        std::process::exit(1);
    }

    Ok(())
}