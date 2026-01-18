//! The main entry point for the `apicentric` CLI.
//!
//! This module is responsible for parsing command-line arguments, initializing
//! the application context, and dispatching to the appropriate command handler.

use apicentric::context::init;
pub use apicentric::{ApicentricError, ApicentricResult as _ApicentricResult};
use apicentric::{ApicentricResult, ContextBuilder, ExecutionContext};
use clap::{Parser, Subcommand, ValueEnum};
#[path = "../commands/ai.rs"]
mod ai_cmd;
#[path = "../commands/shared.rs"]
mod shared_impl;
#[path = "../commands/simulator/mod.rs"]
mod simulator_cmd;

#[cfg(feature = "tui")]
#[path = "../commands/tui.rs"]
mod tui_cmd;
#[cfg(feature = "tui")]
#[path = "../commands/tui_events.rs"]
mod tui_events;
#[cfg(feature = "tui")]
#[path = "../commands/tui_render.rs"]
mod tui_render;
#[cfg(feature = "tui")]
#[path = "../commands/tui_state.rs"]
mod tui_state;

#[cfg(feature = "gui")]
#[path = "../commands/gui/mod.rs"]
mod gui_cmd;

#[cfg(feature = "webui")]
#[path = "../commands/cloud.rs"]
mod cloud_cmd;

#[cfg(feature = "mcp")]
#[path = "../commands/mcp/mod.rs"]
mod mcp_cmd;

#[path = "../commands/doctor.rs"]
mod doctor_cmd;
#[path = "../commands/new.rs"]
mod new_cmd;
#[path = "../commands/open.rs"]
mod open_cmd;

#[cfg(feature = "iot")]
use apicentric::iot::args::TwinCommands;

#[cfg(feature = "iot")]
#[path = "../commands/twin.rs"]
mod twin_cmd;

mod commands {
    pub mod shared {
        pub use crate::shared_impl::*;
    }
}
// Disabled heavy P2P dependencies for lighter CLI build
// mod collab {
//     pub use apicentric::collab::*;
// }

/// The command-line interface for `apicentric`.
#[derive(Parser)]
#[command(author, version, about = "apicentric CLI (lightweight)")]
struct Cli {
    /// The path to the `apicentric.json` config file.
    #[arg(short, long, default_value = "apicentric.json")]
    config: String,

    /// The execution mode (overrides config).
    #[arg(long, value_enum)]
    mode: Option<CliExecutionMode>,

    /// Enables dry-run mode (shows what would be executed).
    #[arg(long)]
    dry_run: bool,

    /// Enables verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// The path to the SQLite database for simulator storage.
    #[arg(long, default_value = "apicentric.db")]
    db_path: String,

    #[command(subcommand)]
    command: Commands,
}

/// The execution mode for the CLI.
#[derive(Clone, ValueEnum)]
enum CliExecutionMode {
    CI,
    Development,
    Debug,
}

impl From<CliExecutionMode> for apicentric::config::ExecutionMode {
    fn from(cli_mode: CliExecutionMode) -> Self {
        match cli_mode {
            CliExecutionMode::CI => apicentric::config::ExecutionMode::CI,
            CliExecutionMode::Development => apicentric::config::ExecutionMode::Development,
            CliExecutionMode::Debug => apicentric::config::ExecutionMode::Debug,
        }
    }
}

/// The commands available in the `apicentric` CLI.
#[derive(Subcommand)]
enum Commands {
    /// API Simulator operations.
    Simulator {
        #[command(subcommand)]
        action: Option<simulator_cmd::SimulatorAction>,
    },
    /// AI-assisted generation.
    Ai {
        #[command(subcommand)]
        action: ai_cmd::AiAction,
    },
    /// Launches the terminal dashboard (requires the 'tui' feature).
    #[cfg(feature = "tui")]
    Tui,
    /// Launches the graphical user interface (requires the 'gui' feature).
    #[cfg(feature = "gui")]
    Gui,
    /// Launches the cloud API server (requires the 'webui' feature).
    #[cfg(feature = "webui")]
    Cloud,
    /// Creates a new service from a template.
    ///
    /// Downloads and installs a service definition from the marketplace templates.
    New {
        /// The name of the new service.
        name: String,
        /// The template ID to use (e.g., stripe, slack, petstore).
        #[arg(long, short)]
        template: Option<String>,
    },
    /// Starts the MCP server for AI agent interaction (requires the 'mcp' feature).
    #[cfg(feature = "mcp")]
    Mcp(mcp_cmd::Mcp),
    /// Diagnose environment issues
    Doctor,
    /// Open the WebUI in default browser
    Open {
        /// Port number (default: 9002)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Manage IoT Digital Twins (requires the 'iot' feature).
    #[cfg(feature = "iot")]
    Twin {
        #[command(subcommand)]
        command: TwinCommands,
    },
}

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

    let cli = Cli::parse();

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
            Some(simulator_cmd::SimulatorAction::Start {
                services_dir,
                force: _,
                p2p: _,
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
            Some(action) => match &action {
                simulator_cmd::SimulatorAction::Start {
                    services_dir: _,
                    force: _,
                    p2p,
                } => {
                    // Start and block to keep services alive
                    if let Some(sim) = context.api_simulator() {
                        if exec_ctx.dry_run {
                            println!("🏃 Dry run: Would start API simulator");
                            return Ok(());
                        }
                        if *p2p {
                            sim.enable_p2p(true).await;
                        }
                        println!("🚀 Starting API Simulator (blocking)…");
                        sim.start().await?;
                        let status = sim.get_status().await;
                        println!(
                            "✅ API Simulator started ({} services, {} active)",
                            status.services_count,
                            status.active_services.len()
                        );
                        for svc in &status.active_services {
                            println!(
                                "   - {}: http://localhost:{}{}",
                                svc.name, svc.port, svc.base_path
                            );
                        }
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
        Commands::Ai { action } => ai_cmd::ai_command(&action, &context, &exec_ctx).await,
        #[cfg(feature = "tui")]
        Commands::Tui => tui_cmd::tui_command().await,
        #[cfg(feature = "gui")]
        Commands::Gui => gui_cmd::gui_command().await,
        #[cfg(feature = "webui")]
        Commands::Cloud => cloud_cmd::cloud_command().await,
        Commands::New { name, template } => {
            new_cmd::new_command(name.clone(), template.clone()).await
        }
        #[cfg(feature = "mcp")]
        Commands::Mcp(mcp) => mcp_cmd::mcp_command(&mcp, &context, &exec_ctx).await,
        Commands::Doctor => doctor_cmd::doctor_command().await,
        Commands::Open { port } => open_cmd::open_command(port).await,
        #[cfg(feature = "iot")]
        Commands::Twin { command } => match command {
            TwinCommands::Run(args) => {
                if exec_ctx.dry_run {
                    println!("🏃 Dry run: Would start Digital Twin from {}", args.device);
                    return Ok(());
                }
                twin_cmd::run(args)
                    .await
                    .map_err(|e| ApicentricError::Runtime {
                        message: e.to_string(),
                        suggestion: None,
                    })
            }
        },
    }
}
