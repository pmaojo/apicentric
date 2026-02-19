//! The main entry point for the `apicentric` CLI.
//!
//! This module is responsible for parsing command-line arguments, initializing
//! the application context, and dispatching to the appropriate command handler.

use apicentric::cli::args::SimulatorAction;
use apicentric::cli::{parse, Cli, Commands};
use apicentric::context::init;
pub use apicentric::{ApicentricError, ApicentricResult as _ApicentricResult};
use apicentric::{ApicentricResult, ContextBuilder, ExecutionContext};

#[cfg(feature = "iot")]
use apicentric::cli::args::TwinCommands;

/// The entry point for the `apicentric` CLI.
#[tokio::main]
async fn main() {
    // Skip logging for TUI mode to prevent log bleed into the terminal UI
    let args: Vec<String> = std::env::args().collect();
    let is_tui = args.iter().any(|a| a == "tui");

    if !is_tui {
        // Initialize structured logging only for non-TUI commands
        apicentric::logging::init();
    }

    let cli = parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Runs the `apicentric` CLI.
///
/// # Arguments
///
/// * `cli` - The parsed command-line arguments.
async fn run(cli: Cli) -> ApicentricResult<()> {
    // Load config from file
    let config_path = std::path::Path::new(&cli.config);
    let mut cfg = apicentric::config::load_config(config_path)?;

    // Apply environment variables overrides
    if let Ok(env_cfg) = apicentric::env_config::EnvConfig::load(false) {
        env_cfg.apply(&mut cfg);
    }

    // Override config with CLI args
    if let Commands::Simulator {
        action:
            Some(SimulatorAction::Start {
                services_dir,
                force: _,
                template: _,
            }),
    } = &cli.command
    {
        if cfg.simulator.is_none() {
            cfg.simulator = Some(apicentric::SimulatorConfig::default());
        }
        if let Some(ref mut sim_config) = cfg.simulator {
            sim_config.services_dir = std::path::PathBuf::from(services_dir);
        }
    }

    let builder = ContextBuilder::new(cfg.clone());

    // Build simulator from config
    let api_simulator = init::build_api_simulator(&cfg);

    let context = builder.with_api_simulator(api_simulator).build()?;

    if let Some(sim) = context.api_simulator() {
        sim.set_db_path(&cli.db_path).await.ok();
    }

    let mut exec_ctx = ExecutionContext::new();
    if let Some(mode) = cli.mode {
        exec_ctx = exec_ctx.with_mode(mode.into());
    }
    if cli.dry_run {
        exec_ctx = exec_ctx.with_dry_run(true);
    }
    if cli.verbose {
        exec_ctx = exec_ctx.with_verbose(true);
    }

    match cli.command {
        Commands::Simulator { action } => match action {
            Some(action) => {
                apicentric::commands::simulator::simulator_command(&action, &context, &exec_ctx)
                    .await
            }
            None => {
                use colored::Colorize;
                println!("{}", "APICENTRIC SIMULATOR".bold().green());
                println!("Usage: apicentric simulator <COMMAND>");
                println!();
                println!("Common commands:");
                println!("  {}     Start the simulator", "start".cyan());
                println!("  {}      Show status", "status".cyan());
                println!("  {}     Show request logs", "logs".cyan());
                println!();
                println!(
                    "Run '{}' for full list.",
                    "apicentric simulator --help".yellow()
                );
                Ok(())
            }
        },
        Commands::Ai { action } => {
            apicentric::commands::ai::ai_command(&action, &context, &exec_ctx).await
        }
        #[cfg(feature = "tui")]
        Commands::Tui => apicentric::commands::tui::tui_command().await,
        #[cfg(feature = "gui")]
        Commands::Gui => apicentric::commands::gui::gui_command().await,
        #[cfg(feature = "webui")]
        Commands::Cloud => apicentric::commands::cloud::cloud_command().await,
        Commands::New { name, template } => {
            apicentric::commands::new::new_command(name.clone(), template.clone()).await
        }
        #[cfg(feature = "mcp")]
        Commands::Mcp(mcp) => {
            apicentric::commands::mcp::mcp_command(&mcp, &context, &exec_ctx).await
        }
        Commands::Doctor => apicentric::commands::doctor::doctor_command().await,
        Commands::Open { port } => apicentric::commands::open::open_command(port).await,
        #[cfg(feature = "iot")]
        Commands::Twin { command } => match command {
            TwinCommands::Run(args) => {
                if exec_ctx.dry_run {
                    println!("🏃 Dry run: Would start Digital Twin from {}", args.device);
                    return Ok(());
                }
                apicentric::commands::twin::run(args)
                    .await
                    .map_err(|e| ApicentricError::Runtime {
                        message: e.to_string(),
                        suggestion: None,
                    })
            }
        },
    }
}
