//! The main entry point for the `apicentric` CLI.
//!
//! This module is responsible for parsing command-line arguments, initializing
//! the application context, and dispatching to the appropriate command handler.

use apicentric::cli::{parse, Cli, Commands, SimulatorAction};
use apicentric::context::init;
use apicentric::{Context, ContextBuilder, ApicentricResult, ExecutionContext, ApicentricError};
use apicentric::commands::{
    ai_command, api_command, contract_command, gui_command, setup_npm, simulator_command,
};
#[cfg(feature = "tui")]
use apicentric::commands::tui_command;

/// The entry point for the `apicentric` CLI.
#[tokio::main]
async fn main() {
    let cli = parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        if let Some(suggestion) = e.suggestion() {
            eprintln!("💡 Suggestion: {}", suggestion);
        }
        std::process::exit(1);
    }
}

/// Runs the `apicentric` CLI.
///
/// # Arguments
///
/// * `cli` - The parsed command-line arguments.
async fn run(cli: Cli) -> ApicentricResult<()> {
    // Setup logging
    if cli.verbose || matches!(cli.mode, Some(apicentric::cli::CliExecutionMode::Debug)) {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    // Load configuration
    let config_path = std::path::Path::new(&cli.config);
    let mut cfg = apicentric::config::load_config(config_path)?;

    // Override config with CLI args
    if let Commands::Simulator {
        action: SimulatorAction::Start { services_dir, p2p: _, force: _ },
    } = &cli.command
    {
        if cfg.simulator.is_none() {
            cfg.simulator = Some(apicentric::SimulatorConfig::default());
        }
        if let Some(ref mut sim_config) = cfg.simulator {
            sim_config.services_dir = std::path::PathBuf::from(services_dir);
        }
    }
    
    // Build context
    let builder = ContextBuilder::new(cfg.clone());
    let api_simulator = init::build_api_simulator(&cfg);
    let context = builder.with_api_simulator(api_simulator).build()?;

    // Set DB path if provided
    if let Some(sim) = context.api_simulator() {
        sim.set_db_path(&cli.db_path).await?;
    }
    
    // Build execution context
    let mut exec_ctx = ExecutionContext::new(context.config());
    if let Some(mode) = cli.mode {
        exec_ctx = exec_ctx.with_mode(mode.into());
    }
    if cli.dry_run {
        exec_ctx = exec_ctx.with_dry_run(true);
    }
    if cli.verbose {
        exec_ctx = exec_ctx.with_verbose(true);
    }

    // Dispatch command
    match cli.command {
        Commands::Simulator { action } => simulator_command(&action, &context, &exec_ctx).await,
        Commands::Contract { action } => contract_command(&action, &context, &exec_ctx).await,
        Commands::Ai { action } => ai_command(&action, &context, &exec_ctx).await,
        Commands::Api { action } => api_command(&action, &context, &exec_ctx).await,
        #[cfg(feature = "gui")]
        Commands::Gui => gui_command().await,
        #[cfg(feature = "tui")]
        Commands::Tui => tui_command().await,
        Commands::SetupNpm { force, instructions_only, test, examples } => {
            setup_npm::setup_npm_scripts(
                &std::env::current_dir()?,
                force,
                instructions_only,
                test,
                examples
            )
        }
    }
}
