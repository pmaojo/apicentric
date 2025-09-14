use crate::config::ExecutionMode;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the mockforge.json config file
    #[arg(short, long, default_value = "mockforge.json")]
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
    #[arg(long, default_value = "pulse.db")]
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
    Start {
        /// Services directory path
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
    Stop {
        /// Force stop all services
        #[arg(long)]
        force: bool,
    },

    /// Show simulator and services status
    Status {
        /// Show detailed service information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate service definition files
    Validate {
        /// Path to service definition file or directory
        #[arg(short, long, default_value = "services")]
        path: String,

        /// Validate all files in directory recursively
        #[arg(short, long)]
        recursive: bool,

        /// Show detailed validation output
        #[arg(long)]
        verbose: bool,
    },
    /// Set default scenario for all services
    SetScenario {
        /// Scenario name to activate
        scenario: String,
    },
    /// Show recent request logs for a service
    Logs {
        /// Service name
        service: String,
        /// Number of log entries to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// AI-assisted generation
    Ai {
        #[command(subcommand)]
        action: AiAction,
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
    /// API Simulator operations for managing service definitions
    Simulator {
        #[command(subcommand)]
        action: SimulatorAction,
    },

    /// Launch the graphical editor for mock services
    Gui,
}

/// Parse command line arguments into a [`Cli`] instance.
pub fn parse() -> Cli {
    Cli::parse()
}
