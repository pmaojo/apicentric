use clap::{Parser, Subcommand, ValueEnum};
use mockforge::{ExecutionContext, PulseResult, ContextBuilder};
pub use mockforge::{PulseError, PulseResult as _PulseResult};
use mockforge::context::init;
#[path = "../commands/simulator.rs"]
mod simulator_cmd;
#[path = "../commands/shared.rs"]
mod shared_impl;
mod commands { pub mod shared { pub use crate::shared_impl::*; } }
mod collab { pub use mockforge::collab::*; }

#[derive(Parser)]
#[command(author, version, about = "MockForge CLI (lightweight)")]
struct Cli {
    /// Path to the mockforge.json config file
    #[arg(short, long, default_value = "mockforge.json")]
    config: String,

    /// Execution mode (overrides config)
    #[arg(long, value_enum)]
    mode: Option<CliExecutionMode>,

    /// Enable dry-run mode (show what would be executed)
    #[arg(long)]
    dry_run: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum CliExecutionMode {
    CI,
    Development,
    Debug,
}

impl From<CliExecutionMode> for mockforge::config::ExecutionMode {
    fn from(cli_mode: CliExecutionMode) -> Self {
        match cli_mode {
            CliExecutionMode::CI => mockforge::config::ExecutionMode::CI,
            CliExecutionMode::Development => mockforge::config::ExecutionMode::Development,
            CliExecutionMode::Debug => mockforge::config::ExecutionMode::Debug,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// API Simulator operations
    Simulator {
        #[command(subcommand)]
        action: simulator_cmd::SimulatorAction,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> PulseResult<()> {
    let config_path = std::path::Path::new(&cli.config);
    let builder = ContextBuilder::new(config_path)?;
    let cfg = builder.config().clone();

    // Build simulator from config
    let api_simulator = init::build_api_simulator(&cfg);

    let context = builder
        .with_api_simulator(api_simulator)
        .build()?;

    let mut exec_ctx = ExecutionContext::new(context.config());
    if let Some(mode) = cli.mode { exec_ctx = exec_ctx.with_mode(mode.into()); }
    if cli.dry_run { exec_ctx = exec_ctx.with_dry_run(true); }
    if cli.verbose { exec_ctx = exec_ctx.with_verbose(true); }

    match cli.command {
        Commands::Simulator { action } => match &action {
            simulator_cmd::SimulatorAction::Start { services_dir: _, force: _, p2p } => {
                // Start and block to keep services alive
                if let Some(sim) = context.api_simulator() {
                    if exec_ctx.dry_run { println!("🏃 Dry run: Would start API simulator"); return Ok(()); }
                    if *p2p { sim.enable_p2p(true).await; }
                    println!("🚀 Starting API Simulator (blocking)…");
                    sim.start().await?;
                    println!("🔄 Simulator running... Press Ctrl+C to stop");
                    tokio::signal::ctrl_c().await.ok();
                    println!("🛑 Stopping simulator…");
                    sim.stop().await.ok();
                    Ok(())
                } else {
                    simulator_cmd::simulator_command(&action, &context, &exec_ctx).await
                }
            }
            _ => simulator_cmd::simulator_command(&action, &context, &exec_ctx).await,
        },
    }
}
