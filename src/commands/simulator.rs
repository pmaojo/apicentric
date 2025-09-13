use crate::commands::shared::{
    find_yaml_files, scaffold_endpoint_definition, scaffold_service_definition, validate_yaml_file,
};
use clap::Subcommand;
use pulse::simulator::log::RequestLogEntry;
use pulse::{Context, ExecutionContext, PulseError, PulseResult};

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
    /// Show recent request logs for a service
    Logs {
        /// Service name
        service: String,
        /// Number of log entries to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Set default scenario for all services
    SetScenario {
        /// Scenario name to activate
        scenario: String,
    },
    /// Import a service from an OpenAPI spec file
    Import {
        /// Path to OpenAPI (Swagger) spec
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Import a service from a Mockoon export
    ImportMockoon {
        /// Path to Mockoon JSON export
        #[arg(short, long)]
        input: String,
        /// Output path for service YAML definition
        #[arg(short, long)]
        output: String,
    },
    /// Export a service definition to an OpenAPI spec file
    Export {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
        /// Output path for OpenAPI spec
        #[arg(short, long)]
        output: String,
    },
    /// Export TypeScript types for a service
    ExportTypes {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
        /// Output path for generated TypeScript
        #[arg(short, long)]
        output: String,
    },
    /// Create a new service definition interactively
    New {
        /// Output directory for the service YAML
        #[arg(short, long, default_value = "services")]
        output: String,
    },
    /// Edit an existing service definition (add endpoint)
    Edit {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
    },
}

pub async fn simulator_command(
    action: &SimulatorAction,
    context: &Context,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    match action {
        SimulatorAction::Start {
            services_dir,
            force,
        } => handle_start(context, services_dir, *force, exec_ctx).await,
        SimulatorAction::Stop { force } => handle_stop(context, *force, exec_ctx).await,
        SimulatorAction::Status { detailed } => handle_status(context, *detailed, exec_ctx).await,
        SimulatorAction::Validate {
            path,
            recursive,
            verbose,
        } => handle_validate(path, *recursive, *verbose, exec_ctx).await,
        SimulatorAction::Logs { service, limit } => {
            handle_logs(context, service, *limit, exec_ctx).await
        }
        SimulatorAction::SetScenario { scenario } => {
            handle_set_scenario(context, scenario, exec_ctx).await
        }
        SimulatorAction::Import { input, output } => handle_import(input, output, exec_ctx).await,
        SimulatorAction::ImportMockoon { input, output } => {
            handle_import_mockoon(input, output, exec_ctx).await
        }
        SimulatorAction::Export { input, output } => handle_export(input, output, exec_ctx).await,
        SimulatorAction::ExportTypes { input, output } => {
            handle_export_types(input, output, exec_ctx).await
        }
        SimulatorAction::New { output } => handle_new(output, exec_ctx).await,
        SimulatorAction::Edit { input } => handle_edit(input, exec_ctx).await,
    }
}

async fn handle_start(
    context: &Context,
    services_dir: &str,
    force: bool,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would start API simulator (dir={}, force={})",
            services_dir, force
        );
        return Ok(());
    }
    println!(
        "🚀 Starting API Simulator...\n📁 Services directory: {}",
        services_dir
    );
    if let Some(simulator) = context.api_simulator() {
        if force && simulator.is_active().await {
            println!("🔄 Force stopping existing simulator...");
            simulator.stop().await?;
        }
        match simulator.start().await {
            Ok(_) => {
                let status = simulator.get_status().await;
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
            }
            Err(e) => {
                return Err(PulseError::runtime_error(
                    format!("Failed to start simulator: {}", e),
                    Some("Check service configurations and port availability"),
                ))
            }
        }
    } else {
        return Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in pulse.json"),
        ));
    }
    Ok(())
}

async fn handle_stop(
    context: &Context,
    force: bool,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would stop API simulator (force={})", force);
        return Ok(());
    }
    println!("🛑 Stopping API Simulator...");
    if let Some(simulator) = context.api_simulator() {
        if simulator.is_active().await {
            simulator.stop().await?;
            println!("✅ API Simulator stopped");
        } else {
            println!("⚠️ API Simulator not running");
        }
    } else {
        return Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in pulse.json"),
        ));
    }
    Ok(())
}

async fn handle_status(
    context: &Context,
    detailed: bool,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would show simulator status (detailed={})",
            detailed
        );
        return Ok(());
    }
    println!("📊 API Simulator Status");
    if let Some(simulator) = context.api_simulator() {
        let status = simulator.get_status().await;
        println!(
            "   Status: {}",
            if status.is_active {
                "🟢 Running"
            } else {
                "🔴 Stopped"
            }
        );
        println!("   Services: {} total", status.services_count);
        println!("   Active Services: {}", status.active_services.len());
        if detailed && !status.active_services.is_empty() {
            println!("\n📋 Service Details:");
            for svc in &status.active_services {
                println!(
                    "   - {} (port {} base {}) endpoints:{} running:{}",
                    svc.name,
                    svc.port,
                    svc.base_path,
                    svc.endpoints_count,
                    if svc.is_running { "yes" } else { "no" }
                );
            }
        }
    } else {
        println!(
            "   Status: ⚪ Not configured\n   💡 Enable simulator in pulse.json to see status"
        );
    }
    Ok(())
}

async fn handle_validate(
    path: &str,
    recursive: bool,
    verbose: bool,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would validate service definitions (path={}, recursive={}, verbose={})",
            path, recursive, verbose
        );
        return Ok(());
    }
    println!("🔍 Validating service definitions...\n📁 Path: {}", path);
    let path_buf = std::path::PathBuf::from(path);
    if !path_buf.exists() {
        println!("⚠️ Path does not exist: {}", path);
        return Ok(());
    }
    let files = if path_buf.is_file() {
        vec![path_buf]
    } else {
        find_yaml_files(&path_buf, recursive)?
    };
    let mut valid = 0usize;
    for f in &files {
        if verbose {
            println!("🔎 {}", f.display());
        }
        match validate_yaml_file(f) {
            Ok(_) => {
                valid += 1;
                if verbose {
                    println!("   ✅ valid");
                }
            }
            Err(e) => println!("   ❌ {}", e),
        }
    }
    println!(
        "\n📊 Validation Results: total={} valid={} invalid={}",
        files.len(),
        valid,
        files.len() - valid
    );
    if valid == files.len() {
        println!("✅ All files valid");
    }
    Ok(())
}

async fn handle_logs(
    context: &Context,
    service: &str,
    limit: usize,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would fetch logs for service '{}' (limit={})",
            service, limit
        );
        return Ok(());
    }
    if let Some(simulator) = context.api_simulator() {
        let status = simulator.get_status().await;
        if let Some(info) = status.active_services.iter().find(|s| s.name == service) {
            let mut url = format!("http://localhost:{}{}", info.port, info.base_path);
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("__pulse/logs?limit=");
            url.push_str(&limit.to_string());
            let resp = reqwest::get(&url).await.map_err(|e| {
                PulseError::runtime_error(format!("Failed to fetch logs: {}", e), None::<String>)
            })?;
            if !resp.status().is_success() {
                return Err(PulseError::runtime_error(
                    format!("Failed to fetch logs: status {}", resp.status()),
                    None::<String>,
                ));
            }
            let logs: Vec<RequestLogEntry> = resp.json().await.map_err(|e| {
                PulseError::runtime_error(format!("Failed to parse logs: {}", e), None::<String>)
            })?;
            if logs.is_empty() {
                println!("No logs available for service '{}'.", service);
            } else {
                for entry in logs {
                    println!(
                        "[{}] {} {} -> {}",
                        entry.timestamp.to_rfc3339(),
                        entry.method,
                        entry.path,
                        entry.status
                    );
                }
            }
            Ok(())
        } else {
            Err(PulseError::runtime_error(
                format!("Service '{}' not found", service),
                Some("Check simulator status for available services"),
            ))
        }
    } else {
        Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in pulse.json"),
        ))
    }
}

async fn handle_set_scenario(
    context: &Context,
    scenario: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would set scenario '{}'", scenario);
        return Ok(());
    }

    if let Some(simulator) = context.api_simulator() {
        simulator.set_scenario(Some(scenario.to_string())).await?;
        println!("✅ Scenario set to '{}'", scenario);
        Ok(())
    } else {
        Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in pulse.json"),
        ))
    }
}

async fn handle_import(input: &str, output: &str, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import OpenAPI '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let spec = openapi::from_path(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read OpenAPI: {}", e), None::<String>)
    })?;
    let service = pulse::simulator::openapi::from_openapi(&spec);
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, yaml).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Imported service to {}", output);
    Ok(())
}

async fn handle_import_mockoon(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import Mockoon '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let service = pulse::simulator::mockoon::from_path(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read Mockoon: {}", e), None::<String>)
    })?;
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, yaml).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Imported service to {}", output);
    Ok(())
}

async fn handle_export(input: &str, output: &str, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export service '{}' to OpenAPI '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: pulse::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let spec = pulse::simulator::openapi::to_openapi(&service);
    let spec_yaml = openapi::to_yaml(&spec).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize OpenAPI: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, spec_yaml).map_err(|e| {
        PulseError::runtime_error(format!("Failed to write spec file: {}", e), None::<String>)
    })?;
    println!("✅ Exported OpenAPI to {}", output);
    Ok(())
}

async fn handle_export_types(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export TypeScript types from '{}' to '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: pulse::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let ts = pulse::simulator::typescript::to_typescript(&service).map_err(|e| {
        PulseError::runtime_error(format!("Failed to generate types: {}", e), None::<String>)
    })?;
    std::fs::write(output, ts).map_err(|e| {
        PulseError::runtime_error(format!("Failed to write types file: {}", e), None::<String>)
    })?;
    println!("✅ Exported TypeScript types to {}", output);
    Ok(())
}

async fn handle_new(output: &str, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would scaffold new service in {}", output);
        return Ok(());
    }

    let service = scaffold_service_definition()?;
    std::fs::create_dir_all(output).map_err(|e| {
        PulseError::fs_error(
            format!("Failed to create directory {}: {}", output, e),
            None::<String>,
        )
    })?;
    let file_path = std::path::Path::new(output).join(format!("{}.yaml", service.name));
    if file_path.exists() {
        return Err(PulseError::fs_error(
            format!("File {} already exists", file_path.display()),
            Some("Choose a different service name"),
        ));
    }
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(&file_path, yaml).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Created service definition at {}", file_path.display());
    Ok(())
}

async fn handle_edit(input: &str, exec_ctx: &ExecutionContext) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would edit service {}", input);
        return Ok(());
    }

    let yaml = std::fs::read_to_string(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let mut service: pulse::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;

    let endpoint = scaffold_endpoint_definition()?;
    service.endpoints.push(endpoint);

    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(input, yaml).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✏️  Updated service at {}", input);
    Ok(())
}
