use clap::Subcommand;
use pulse::{PulseResult, PulseError, Context, ExecutionContext};
use crate::commands::shared::{find_yaml_files, validate_yaml_file};

#[derive(Subcommand, Debug)]
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

pub async fn simulator_command(action: &SimulatorAction, context: &Context, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    match action {
        SimulatorAction::Start { services_dir, force } => handle_start(context, services_dir, *force, exec_ctx).await,
        SimulatorAction::Stop { force } => handle_stop(context, *force, exec_ctx).await,
        SimulatorAction::Status { detailed } => handle_status(context, *detailed, exec_ctx).await,
        SimulatorAction::Validate { path, recursive, verbose } => handle_validate(path, *recursive, *verbose, exec_ctx).await,
    }
}

async fn handle_start(context: &Context, services_dir: &str, force: bool, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run { println!("🏃 Dry run: Would start API simulator (dir={}, force={})", services_dir, force); return Ok(()); }
    println!("🚀 Starting API Simulator...\n📁 Services directory: {}", services_dir);
    if let Some(simulator) = context.api_simulator() {
        if force && simulator.is_active().await { println!("🔄 Force stopping existing simulator..."); simulator.stop().await?; }
        match simulator.start().await {
            Ok(_) => {
                let status = simulator.get_status().await;
                println!("✅ API Simulator started ({} services, {} active)", status.services_count, status.active_services.len());
                for svc in &status.active_services { println!("   - {}: http://localhost:{}{}", svc.name, svc.port, svc.base_path); }
            }
            Err(e) => return Err(PulseError::runtime_error(format!("Failed to start simulator: {}", e), Some("Check service configurations and port availability")))
        }
    } else { return Err(PulseError::config_error("API simulator is not enabled or configured", Some("Enable simulator in pulse.json"))); }
    Ok(())
}

async fn handle_stop(context: &Context, force: bool, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run { println!("🏃 Dry run: Would stop API simulator (force={})", force); return Ok(()); }
    println!("🛑 Stopping API Simulator...");
    if let Some(simulator) = context.api_simulator() {
        if simulator.is_active().await { simulator.stop().await?; println!("✅ API Simulator stopped"); } else { println!("⚠️ API Simulator not running"); }
    } else { return Err(PulseError::config_error("API simulator is not enabled or configured", Some("Enable simulator in pulse.json"))); }
    Ok(())
}

async fn handle_status(context: &Context, detailed: bool, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run { println!("🏃 Dry run: Would show simulator status (detailed={})", detailed); return Ok(()); }
    println!("📊 API Simulator Status");
    if let Some(simulator) = context.api_simulator() {
        let status = simulator.get_status().await;
        println!("   Status: {}", if status.is_active { "🟢 Running" } else { "🔴 Stopped" });
        println!("   Services: {} total", status.services_count);
        println!("   Active Services: {}", status.active_services.len());
        if detailed && !status.active_services.is_empty() {
            println!("\n📋 Service Details:");
            for svc in &status.active_services {
                println!("   - {} (port {} base {}) endpoints:{} running:{}", svc.name, svc.port, svc.base_path, svc.endpoints_count, if svc.is_running {"yes"} else {"no"});
            }
        }
    } else { println!("   Status: ⚪ Not configured\n   💡 Enable simulator in pulse.json to see status"); }
    Ok(())
}

async fn handle_validate(path: &str, recursive: bool, verbose: bool, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run { println!("🏃 Dry run: Would validate service definitions (path={}, recursive={}, verbose={})", path, recursive, verbose); return Ok(()); }
    println!("🔍 Validating service definitions...\n📁 Path: {}", path);
    let path_buf = std::path::PathBuf::from(path);
    if !path_buf.exists() { println!("⚠️ Path does not exist: {}", path); return Ok(()); }
    let files = if path_buf.is_file() { vec![path_buf] } else { find_yaml_files(&path_buf, recursive)? };
    let mut valid = 0usize;
    for f in &files { if verbose { println!("🔎 {}", f.display()); }
        match validate_yaml_file(f) { Ok(_) => { valid += 1; if verbose { println!("   ✅ valid"); } }, Err(e) => println!("   ❌ {}", e) }
    }
    println!("\n📊 Validation Results: total={} valid={} invalid={}", files.len(), valid, files.len()-valid);
    if valid == files.len() { println!("✅ All files valid"); }
    Ok(())
}
