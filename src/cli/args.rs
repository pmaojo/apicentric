//! CLI Argument Definitions
//!
//! This module defines the data structures for the command-line interface.
//! It replaces the usage of `clap` with plain Rust structs and enums.

use crate::config::ExecutionMode;

/// The main CLI structure
#[derive(Debug, Clone)]
pub struct Cli {
    /// The path to the `apicentric.json` config file.
    /// Default: "apicentric.json"
    pub config: String,

    /// The execution mode (overrides config).
    pub mode: Option<CliExecutionMode>,

    /// Enables dry-run mode (shows what would be executed).
    pub dry_run: bool,

    /// Enables verbose output.
    pub verbose: bool,

    /// The path to the SQLite database for simulator storage.
    /// Default: "apicentric.db"
    pub db_path: String,

    /// The subcommand to execute.
    pub command: Commands,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            config: "apicentric.json".to_string(),
            mode: None,
            dry_run: false,
            verbose: false,
            db_path: "apicentric.db".to_string(),
            command: Commands::Doctor, // Default fallthrough if parsing fails or help is needed? No, parsing handles defaults.
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub enum Commands {
    Simulator {
        action: Option<SimulatorAction>,
    },
    Ai {
        action: AiAction,
    },
    #[cfg(feature = "tui")]
    Tui,
    #[cfg(feature = "gui")]
    Gui,
    #[cfg(feature = "webui")]
    Cloud,
    New {
        name: String,
        template: Option<String>,
    },
    #[cfg(feature = "mcp")]
    Mcp(Mcp),
    Doctor,
    Open {
        port: Option<u16>,
    },
    #[cfg(feature = "iot")]
    Twin {
        command: TwinCommands,
    },
}

#[derive(Debug, Clone)]
pub enum SimulatorAction {
    Start {
        services_dir: String,
        force: bool,
        template: Option<String>,
    },
    Stop {
        force: bool,
    },
    Status {
        detailed: bool,
    },
    Validate {
        file: String,
        recursive: bool,
        verbose: bool,
    },
    Logs {
        service: String,
        limit: usize,
        method: Option<String>,
        route: Option<String>,
        status: Option<u16>,
        output: Option<String>,
    },
    Monitor {
        service: Option<String>,
        json: bool,
        interval: Option<u64>,
    },
    SetScenario {
        scenario: String,
    },
    Import {
        file: String,
        output: String,
    },
    Export {
        file: String,
        output: String,
        format: ExportFormat,
    },
    GenerateTypes {
        file: String,
        output: String,
    },
    GenerateQuery {
        file: String,
        output: String,
    },
    GenerateView {
        file: String,
        output: String,
    },
    New {
        output: String,
    },
    NewGraphql {
        name: String,
        output: String,
    },
    Edit {
        file: String,
    },
    Record {
        output: String,
        url: Option<String>,
    },
    Dockerize {
        file: Vec<String>,
        output: String,
    },
    Test {
        path: String,
        url: String,
        env: String,
        quiet: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExportFormat {
    Openapi,
    Postman,
}

#[derive(Debug, Clone)]
pub enum AiAction {
    Generate { prompt: String },
}

#[cfg(feature = "iot")]
#[derive(Debug, Clone)]
pub enum TwinCommands {
    Run(TwinRunArgs),
}

#[cfg(feature = "iot")]
#[derive(Debug, Clone)]
pub struct TwinRunArgs {
    pub device: String,
    pub override_config: Option<String>,
    pub library: String,
}

#[cfg(feature = "mcp")]
#[derive(Debug, Clone)]
pub struct Mcp {
    pub test: bool,
}
