//! The command-line interface for `apicentric`.
//!
//! This module provides the command-line interface for `apicentric`, including
//! the main `Cli` struct and the `Commands` enum.

use crate::config::ExecutionMode;
use clap::{Parser, Subcommand, ValueEnum};

/// The command-line interface for `apicentric`.
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
    /// The path to the `apicentric.json` config file.
    #[arg(short, long, default_value = "apicentric.json")]
    pub config: String,

    /// The execution mode (overrides config).
    #[arg(long, value_enum)]
    pub mode: Option<CliExecutionMode>,

    /// Enables dry-run mode (shows what would be executed).
    #[arg(long)]
    pub dry_run: bool,

    /// Enables verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// The path to the SQLite database for simulator storage.
    #[arg(long, default_value = "apicentric.db")]
    pub db_path: String,

    #[command(subcommand)]
    pub command: Commands,
}

/// The execution mode for the CLI.
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

/// The actions available for the simulator.
#[derive(Subcommand)]
pub enum SimulatorAction {
    /// Starts the API simulator.
    ///
    /// Starts the API simulator and loads all service definitions from the specified directory.
    /// Services will be available on their configured ports.
    ///
    /// Example: apicentric simulator start --services-dir ./services
    #[command(alias = "s")]
    Start {
        /// The path to the directory containing service definition YAML files.
        #[arg(short, long, default_value = "services")]
        services_dir: String,

        /// Forces the simulator to start even if services are already running.
        #[arg(long)]
        force: bool,

        /// Enables peer-to-peer collaboration for service editing.
        #[arg(long)]
        p2p: bool,
    },

    /// Stops the API simulator.
    ///
    /// Stops all running services and shuts down the simulator.
    ///
    /// Example: apicentric simulator stop
    #[command(alias = "x")]
    Stop {
        /// Force stops all services immediately without graceful shutdown.
        #[arg(long)]
        force: bool,
    },

    /// Shows the simulator and services status.
    ///
    /// Displays the current status of the simulator and all registered services,
    /// including port numbers, running state, and request counts.
    ///
    /// Example: apicentric simulator status --detailed
    #[command(alias = "st")]
    Status {
        /// Shows detailed service information including endpoints and configurations.
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validates service definition files.
    ///
    /// Validates YAML service definition files for syntax errors and schema compliance.
    /// Can validate a single file or all files in a directory.
    ///
    /// Example: apicentric simulator validate --path services/my-api.yaml
    #[command(alias = "v")]
    Validate {
        /// The path to the service definition YAML file or directory to validate.
        #[arg(short, long, default_value = "services")]
        path: String,

        /// Validates all YAML files in subdirectories recursively.
        #[arg(short, long)]
        recursive: bool,

        /// Shows detailed validation output including warnings.
        #[arg(long)]
        verbose: bool,
    },
    /// Sets the default scenario for all services.
    SetScenario {
        /// The scenario name to activate.
        scenario: String,
    },
    /// Shows recent request logs for a service.
    #[command(alias = "l")]
    Logs {
        /// The service name.
        service: String,
        /// The number of log entries to display.
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Monitors the simulator status and logs.
    #[command(alias = "m")]
    Monitor {
        /// The service name to monitor.
        #[arg(long)]
        service: Option<String>,
        /// The output in JSON format for scripting.
        #[arg(long)]
        json: bool,
        /// The refresh interval in seconds for continuous monitoring.
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Records live API traffic into service definitions.
    Record {
        /// The output directory for generated services.
        #[arg(short, long, default_value = "services")]
        output: String,
        /// The target URL to proxy requests to (defaults to base_url in config).
        #[arg(long)]
        url: Option<String>,
    },
    /// Shares a running service over libp2p.
    Share {
        /// The service name to expose.
        service: String,
    },
    /// Imports WireMock stub mappings into a service definition.
    #[command(name = "import-wiremock")]
    ImportWiremock {
        /// The path to the WireMock mapping JSON file or directory export.
        #[arg(short, long)]
        input: String,
        /// The output path for the generated service YAML definition.
        #[arg(short, long)]
        output: String,
    },
    /// Connects to a shared service and proxy locally.
    Connect {
        /// The remote peer ID.
        peer: String,
        /// The service name to access.
        #[arg(long)]
        service: String,
        /// The local port to listen on.
        #[arg(long)]
        port: u16,
        /// The authentication token issued by the peer.
        #[arg(long)]
        token: Option<String>,
    },

    /// Creates a Docker image for a service.
    Dockerize {
        /// The path to the service definition YAML file(s).
        #[arg(short, long, required = true)]
        services: Vec<String>,
        /// The output directory for the Dockerfile and service files.
        #[arg(short, long, default_value = ".")]
        output: String,
    },
}

/// The actions available for the AI.
#[derive(Subcommand)]
pub enum AiAction {
    /// Generates YAML from a natural language prompt.
    Generate {
        /// The prompt to send to the AI provider.
        prompt: String,
    },
}

/// The commands available in the `apicentric` CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Manages the API simulator and mock services.
    ///
    /// The simulator command provides operations for starting, stopping, and managing
    /// mock API services defined in YAML files.
    #[command(alias = "sim")]
    Simulator {
        #[command(subcommand)]
        action: SimulatorAction,
    },

    /// AI-assisted service generation.
    ///
    /// Use AI to generate service definitions from natural language descriptions.
    Ai {
        #[command(subcommand)]
        action: AiAction,
    },

    /// Launches the graphical editor for mock services.
    ///
    /// Opens a GUI application for visually editing service definitions.
    /// Requires the GUI component to be installed.
    ///
    /// Example: apicentric gui
    Gui,

    /// Launches the terminal dashboard.
    ///
    /// Opens an interactive terminal UI for managing services, viewing logs,
    /// and monitoring the simulator in real-time.
    ///
    /// Example: apicentric tui
    Tui,

    /// Launches the Apicentric Cloud API server.
    ///
    /// Starts the cloud server, which provides a web API for managing services.
    /// This is the intended backend for the WebUI.
    ///
    /// Example: apicentric cloud
    Cloud,
}

/// Parses the command-line arguments into a `Cli` instance.
pub fn parse() -> Cli {
    Cli::parse()
}
