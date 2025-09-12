use crate::config::ExecutionMode;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the pulse.json config file
    #[arg(short, long, default_value = "pulse.json")]
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
}

#[derive(Subcommand)]
pub enum ContractAction {
    /// Register a new contract from a service specification
    Register {
        /// Service name
        #[arg(short = 'n', long)]
        service: String,

        /// Path to YAML specification file
        #[arg(short = 's', long)]
        spec: String,

        /// Contract description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// List all registered contracts
    List {
        /// Show detailed contract information
        #[arg(short, long)]
        detailed: bool,

        /// Filter by service name
        #[arg(long)]
        service: Option<String>,
    },

    /// Validate a contract against real API
    Validate {
        /// Contract ID to validate
        #[arg(short, long)]
        contract_id: String,

        /// Environment to test against (prod, staging, dev)
        #[arg(short, long, default_value = "dev")]
        environment: String,

        /// Compatibility policy (strict, moderate, lenient)
        #[arg(short, long, default_value = "moderate")]
        policy: String,

        /// Generate HTML report
        #[arg(long)]
        html_report: bool,

        /// Send notifications
        #[arg(long)]
        notify: bool,
    },

    /// Validate all contracts
    ValidateAll {
        /// Environment to test against
        #[arg(short, long, default_value = "dev")]
        environment: String,

        /// Compatibility policy
        #[arg(short, long, default_value = "moderate")]
        policy: String,

        /// Continue on first failure
        #[arg(long)]
        fail_fast: bool,

        /// Generate comprehensive report
        #[arg(long)]
        report: bool,
    },

    /// Delete a contract
    Delete {
        /// Contract ID to delete
        #[arg(short, long)]
        contract_id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Show contract details
    Show {
        /// Contract ID to show
        #[arg(short, long)]
        contract_id: String,

        /// Show validation history
        #[arg(long)]
        history: bool,
    },

    /// Import contracts from directory
    Import {
        /// Directory containing YAML specifications
        #[arg(short, long, default_value = "mock_services")]
        directory: String,

        /// Import recursively
        #[arg(short, long)]
        recursive: bool,

        /// Overwrite existing contracts
        #[arg(long)]
        overwrite: bool,
    },

    /// Complete contract testing demo - validates contract, mock API, and real API
    Demo {
        /// Contract ID to demonstrate
        #[arg(short, long)]
        contract_id: String,

        /// YAML spec file to use as contract (alternative to registered contract)
        #[arg(long)]
        spec_file: Option<String>,

        /// Mock API port (default: 8080)
        #[arg(long, default_value = "8080")]
        mock_port: u16,

        /// Real API base URL (will detect from contract or use provided)
        #[arg(long)]
        real_api_url: Option<String>,

        /// Test endpoints to validate (comma-separated, e.g., "/people/1,/people/999")
        #[arg(long)]
        test_endpoints: Option<String>,

        /// Compatibility policy (strict, moderate, lenient)
        #[arg(short, long, default_value = "moderate")]
        policy: String,

        /// Start mock server automatically
        #[arg(long)]
        auto_start_mock: bool,

        /// Generate detailed HTML report
        #[arg(long)]
        html_report: bool,

        /// Incluir arranque y comprobación del API Simulator
        #[arg(long)]
        with_simulator: bool,

        /// Número máximo de endpoints a muestrear por servicio del simulador
        #[arg(long, default_value = "2")]
        simulator_sample: usize,
    },
}

#[derive(Subcommand)]
pub enum Commands {

    /// Watch for changes and run impacted tests
    Watch {
        /// Number of parallel test runners
        #[arg(short, long, default_value_t = 4)]
        workers: usize,

        /// Number of retries for failed tests
        #[arg(short, long, default_value_t = 3)]
        retries: u8,

        /// Debounce time in milliseconds
        #[arg(long, default_value_t = 1000)]
        debounce_ms: u64,
    },

    /// Run all tests once
    Run {
        /// Number of parallel test runners
        #[arg(short, long, default_value_t = 4)]
        workers: usize,

        /// Number of retries for failed tests
        #[arg(short, long, default_value_t = 3)]
        retries: u8,
    },

    /// Setup npm scripts for pulse integration
    SetupNpm {
        /// Force overwrite existing scripts
        #[arg(long)]
        force: bool,

        /// Only show instructions without modifying package.json
        #[arg(long)]
        instructions_only: bool,

        /// Test npm script execution
        #[arg(long)]
        test: bool,

        /// Show usage examples
        #[arg(long)]
        examples: bool,
    },

    /// Generate TypeScript documentation with TypeDoc
    Docs {
        /// Generate documentation and open in browser
        #[arg(long)]
        serve: bool,

        /// Output directory for generated docs
        #[arg(short, long, default_value = "docs")]
        output: String,

        /// Watch for changes and regenerate docs
        #[arg(short, long)]
        watch: bool,
    },
    /// Sirve una API mock basada en un archivo YAML (data-driven)
    MockApi {
        /// Ruta al archivo YAML con la definición de endpoints
        #[arg(short, long, default_value = "pulse-mock.yaml")]
        spec: String,

        /// Solo validar el YAML sin levantar el servidor
        #[arg(long)]
        validate: bool,
    },

    /// API Simulator operations for managing service definitions
    Simulator {
        #[command(subcommand)]
        action: SimulatorAction,
    },

    /// Contract Testing operations for validating APIs against specifications
    Contract {
        #[command(subcommand)]
        action: ContractAction,
    },

    /// Launch the graphical editor for mock services
    Gui,
}

/// Parse command line arguments into a [`Cli`] instance.
pub fn parse() -> Cli {
    Cli::parse()
}
