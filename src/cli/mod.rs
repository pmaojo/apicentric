use crate::config::ExecutionMode;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A powerful CLI tool and API simulator platform for developers",
    long_about = "Apicentric is a Rust-based CLI tool for mocking APIs, testing contracts, and generating code.\n\n\
                  Use 'apicentric <COMMAND> --help' for more information about a specific command.\n\n\
                  Examples:\n  \
                  apicentric simulator start --services-dir ./services\n  \
                  apicentric tui\n  \
                  apicentric simulator validate --path services/my-api.yaml",
    after_help = "For more information, visit: https://github.com/pmaojo/apicentric"
)]
pub struct Cli {
    /// Path to the apicentric.json config file
    #[arg(short, long, default_value = "apicentric.json")]
    pub config: String,

    /// Execution mode (overrides config)
    #[arg(long, value_enum)]
    pub mode: Option<CliExecutionMode>,

    /// Enable dry-run mode (show what would be executed)
    #[arg(long)]
    pub dry_run: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Path to SQLite database for simulator storage
    #[arg(long, default_value = "apicentric.db")]
    pub db_path: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum CliExecutionMode {
    CI,
    Development,
    Debug,
}

impl From<CliExecutionMode> for ExecutionMode {
    fn from(cli_mode: CliExecutionMode) -> Self {
        match cli_mode {
            CliExecutionMode::CI => ExecutionMode::CI,
            CliExecutionMode::Development => ExecutionMode::Development,
            CliExecutionMode::Debug => ExecutionMode::Debug,
        }
    }
}

#[derive(Subcommand)]
pub enum SimulatorAction {
    /// Start the API simulator
    /// 
    /// Starts the API simulator and loads all service definitions from the specified directory.
    /// Services will be available on their configured ports.
    /// 
    /// Example: apicentric simulator start --services-dir ./services
    #[command(alias = "s")]
    Start {
        /// Path to directory containing service definition YAML files
        #[arg(short, long, default_value = "services")]
        services_dir: String,

        /// Force start even if services are already running
        #[arg(long)]
        force: bool,

        /// Enable peer-to-peer collaboration for service editing
        #[arg(long)]
        p2p: bool,
    },

    /// Stop the API simulator
    /// 
    /// Stops all running services and shuts down the simulator.
    /// 
    /// Example: apicentric simulator stop
    #[command(alias = "x")]
    Stop {
        /// Force stop all services immediately without graceful shutdown
        #[arg(long)]
        force: bool,
    },

    /// Show simulator and services status
    /// 
    /// Displays the current status of the simulator and all registered services,
    /// including port numbers, running state, and request counts.
    /// 
    /// Example: apicentric simulator status --detailed
    #[command(alias = "st")]
    Status {
        /// Show detailed service information including endpoints and configurations
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate service definition files
    /// 
    /// Validates YAML service definition files for syntax errors and schema compliance.
    /// Can validate a single file or all files in a directory.
    /// 
    /// Example: apicentric simulator validate --path services/my-api.yaml
    #[command(alias = "v")]
    Validate {
        /// Path to service definition YAML file or directory to validate
        #[arg(short, long, default_value = "services")]
        path: String,

        /// Validate all YAML files in subdirectories recursively
        #[arg(short, long)]
        recursive: bool,

        /// Show detailed validation output including warnings
        #[arg(long)]
        verbose: bool,
    },
    /// Set default scenario for all services
    SetScenario {
        /// Scenario name to activate
        scenario: String,
    },
    /// Show recent request logs for a service
    #[command(alias = "l")]
    Logs {
        /// Service name
        service: String,
        /// Number of log entries to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Monitor simulator status and logs
    #[command(alias = "m")]
    Monitor {
        /// Service name to monitor
        #[arg(long)]
        service: Option<String>,
        /// Output in JSON format for scripting
        #[arg(long)]
        json: bool,
        /// Refresh interval in seconds for continuous monitoring
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Record live API traffic into service definitions
    Record {
        /// Output directory for generated services
        #[arg(short, long, default_value = "services")]
        output: String,
        /// Target URL to proxy requests to (defaults to base_url in config)
        #[arg(long)]
        url: Option<String>,
    },
    /// Share a running service over libp2p
    Share {
        /// Service name to expose
        service: String,
    },
    /// Connect to a shared service and proxy locally
    Connect {
        /// Remote peer ID
        peer: String,
        /// Service name to access
        #[arg(long)]
        service: String,
        /// Local port to listen on
        #[arg(long)]
        port: u16,
        /// Authentication token issued by the peer
        #[arg(long)]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AiAction {
    /// Generate YAML from a natural language prompt
    Generate {
        /// The prompt to send to the AI provider
        prompt: String,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage API simulator and mock services
    /// 
    /// The simulator command provides operations for starting, stopping, and managing
    /// mock API services defined in YAML files.
    #[command(alias = "sim")]
    Simulator {
        #[command(subcommand)]
        action: SimulatorAction,
    },

    /// AI-assisted service generation
    /// 
    /// Use AI to generate service definitions from natural language descriptions.
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    /// Launch the graphical editor for mock services
    /// 
    /// Opens a GUI application for visually editing service definitions.
    /// Requires the GUI component to be installed.
    /// 
    /// Example: apicentric gui
    Gui,

    /// Launch the terminal dashboard
    /// 
    /// Opens an interactive terminal UI for managing services, viewing logs,
    /// and monitoring the simulator in real-time.
    /// 
    /// Example: apicentric tui
    Tui,
}

/// Parse command line arguments into a [`Cli`] instance.
pub fn parse() -> Cli {
    Cli::parse()
}
