use crate::commands::shared::{
    find_yaml_files, scaffold_endpoint_definition, scaffold_service_definition, validate_yaml_file,
};
use clap::Subcommand;
use mockforge::simulator::log::RequestLogEntry;
use mockforge::{Context, ExecutionContext, PulseError, PulseResult};

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
    /// Show recent request logs for a service
    Logs {
        /// Service name
        service: String,
        /// Number of log entries to display
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Filter by HTTP method
        #[arg(long)]
        method: Option<String>,
        /// Filter by route substring
        #[arg(long)]
        route: Option<String>,
        /// Filter by response status code
        #[arg(long)]
        status: Option<u16>,
        /// Output file to write logs as JSON
        #[arg(long)]
        output: Option<String>,
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
    /// Import a service from a Postman or Insomnia collection
    #[command(name = "import-postman")]
    ImportPostman {
        /// Path to Postman/Insomnia JSON export
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
    /// Export a service definition to a Postman collection
    #[command(name = "export-postman")]
    ExportPostman {
        /// Path to service YAML definition
        #[arg(short, long)]
        input: String,
        /// Output path for Postman collection JSON
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
    /// Record live API interactions into service definitions
    Record {
        /// Output directory for generated YAML services
        #[arg(short, long, default_value = "services")]
        output: String,
        /// Target URL to proxy requests to (defaults to config base_url)
        #[arg(long)]
        url: Option<String>,
    },
    /// Share a running service over libp2p
    Share {
        /// Service name to expose
        service: String,
    },
    /// Connect to a shared service and proxy locally
    Connect {
        /// Remote peer ID
        peer: String,
        /// Service name to access
        #[arg(long)]
        service: String,
        /// Local port to listen on
        #[arg(long)]
        port: u16,
        /// Authentication token issued by the peer
        #[arg(long)]
        token: Option<String>,
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
            p2p,
        } => handle_start(context, services_dir, *force, *p2p, exec_ctx).await,
        SimulatorAction::Stop { force } => handle_stop(context, *force, exec_ctx).await,
        SimulatorAction::Status { detailed } => handle_status(context, *detailed, exec_ctx).await,
        SimulatorAction::Validate {
            path,
            recursive,
            verbose,
        } => handle_validate(path, *recursive, *verbose, exec_ctx).await,
        SimulatorAction::Logs {
            service,
            limit,
            method,
            route,
            status,
            output,
        } => {
            handle_logs(
                context,
                service,
                *limit,
                method.as_deref(),
                route.as_deref(),
                *status,
                output.as_deref(),
                exec_ctx,
            )
            .await
        }
        SimulatorAction::SetScenario { scenario } => {
            handle_set_scenario(context, scenario, exec_ctx).await
        }
        SimulatorAction::Import { input, output } => handle_import(input, output, exec_ctx).await,
        SimulatorAction::ImportMockoon { input, output } => {
            handle_import_mockoon(input, output, exec_ctx).await
        }
        SimulatorAction::ImportPostman { input, output } => {
            handle_import_postman(input, output, exec_ctx).await
        }
        SimulatorAction::Export { input, output } => handle_export(input, output, exec_ctx).await,
        SimulatorAction::ExportTypes { input, output } => {
            handle_export_types(input, output, exec_ctx).await
        }
        SimulatorAction::ExportPostman { input, output } => {
            handle_export_postman(input, output, exec_ctx).await
        }
        SimulatorAction::New { output } => handle_new(output, exec_ctx).await,
        SimulatorAction::Edit { input } => handle_edit(input, exec_ctx).await,
        SimulatorAction::Record { output, url } => {
            handle_record(context, output, url, exec_ctx).await
        }
        SimulatorAction::Share { service } => {
            handle_share(context, service, exec_ctx).await
        }
        SimulatorAction::Connect { peer, service, port, token } => {
            handle_connect(peer, service, *port, token.as_deref(), exec_ctx).await
        }
    }
}

async fn handle_start(
    context: &Context,
    services_dir: &str,
    force: bool,
    p2p: bool,
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
        if p2p {
            simulator.enable_p2p(true).await;
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
            Some("Enable simulator in mockforge.json"),
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
            Some("Enable simulator in mockforge.json"),
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
            "   Status: ⚪ Not configured\n   💡 Enable simulator in mockforge.json to see status"
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
    method: Option<&str>,
    route: Option<&str>,
    status: Option<u16>,
    output: Option<&str>,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would fetch logs for service '{}' (limit={}, method={:?}, route={:?}, status={:?}, output={:?})",
            service, limit, method, route, status, output
        );
        return Ok(());
    }
    if let Some(simulator) = context.api_simulator() {
        let sim_status = simulator.get_status().await;
        if let Some(info) = sim_status
            .active_services
            .iter()
            .find(|s| s.name == service)
        {
            let mut url = format!("http://localhost:{}{}", info.port, info.base_path);
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("__mockforge/logs?limit=");
            url.push_str(&limit.to_string());
            if let Some(m) = method {
                url.push_str("&method=");
                url.push_str(m);
            }
            if let Some(r) = route {
                url.push_str("&route=");
                url.push_str(r);
            }
            if let Some(s) = status {
                url.push_str("&status=");
                url.push_str(&s.to_string());
            }
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
                if let Some(path) = output {
                    let file = std::fs::File::create(path).map_err(|e| {
                        PulseError::runtime_error(
                            format!("Failed to write logs to {}: {}", path, e),
                            None::<String>,
                        )
                    })?;
                    serde_json::to_writer_pretty(file, &logs).map_err(|e| {
                        PulseError::runtime_error(
                            format!("Failed to serialize logs: {}", e),
                            None::<String>,
                        )
                    })?;
                    println!("Saved {} log entries to {}", logs.len(), path);
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
            Some("Enable simulator in mockforge.json"),
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
            Some("Enable simulator in mockforge.json"),
        ))
    }
}

async fn handle_record(
    context: &Context,
    output: &str,
    url: &Option<String>,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    let target = url
        .clone()
        .unwrap_or_else(|| context.config().base_url.clone());
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would record traffic to '{}' (target={})",
            output, target
        );
        return Ok(());
    }
    if let Some(simulator) = context.api_simulator() {
        simulator
            .record(&target, std::path::PathBuf::from(output))
            .await?;
        Ok(())
    } else {
        Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in mockforge.json"),
        ))
    }
}

async fn handle_share(
    context: &Context,
    service: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would share service '{}'", service);
        return Ok(());
    }
    if let Some(simulator) = context.api_simulator() {
        let registry = simulator.service_registry().read().await;
        if let Some(instance) = registry.get_service(service) {
            let port = instance.read().await.port();
            drop(registry);
            match share::share_service(port).await {
                Ok((peer, token)) => {
                    println!("📡 Sharing service '{}'", service);
                    println!("   Peer ID: {}", peer);
                    println!("   Token: {}", token);
                    Ok(())
                }
                Err(e) => Err(PulseError::runtime_error(
                    format!("Failed to share service: {}", e),
                    None::<String>,
                )),
            }
        } else {
            println!("⚠️ Service '{}' not found", service);
            Ok(())
        }
    } else {
        Err(PulseError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in mockforge.json"),
        ))
    }
}

async fn handle_connect(
    peer: &str,
    service: &str,
    port: u16,
    token: Option<&str>,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would connect to peer '{}' service '{}' on port {}",
            peer, service, port
        );
        return Ok(());
    }
    let peer_id = peer
        .parse::<libp2p::PeerId>()
        .map_err(|e| PulseError::runtime_error(format!("Invalid peer id: {}", e), None::<String>))?;
    let token = token.unwrap_or("").to_string();
    share::connect_service(peer_id, token, service.to_string(), port)
        .await
        .map_err(|e| {
            PulseError::runtime_error(format!("Failed to connect: {}", e), None::<String>)
        })
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
    let service = mockforge::simulator::openapi::from_openapi(&spec);
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
    let service = mockforge::simulator::mockoon::from_path(input).map_err(|e| {
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

async fn handle_import_postman(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import Postman '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let service = mockforge::simulator::postman::from_path(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read Postman: {}", e), None::<String>)
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
    let service: mockforge::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let spec = mockforge::simulator::openapi::to_openapi(&service);
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

async fn handle_export_postman(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export service '{}' to Postman '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        PulseError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: mockforge::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let json = mockforge::simulator::postman::to_string(&service).map_err(|e| {
        PulseError::runtime_error(
            format!("Failed to serialize Postman: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, json).map_err(|e| {
        PulseError::runtime_error(format!("Failed to write collection: {}", e), None::<String>)
    })?;
    println!("✅ Exported Postman collection to {}", output);
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
    let service: mockforge::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            PulseError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let ts = mockforge::simulator::typescript::to_typescript(&service).map_err(|e| {
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
    let mut service: mockforge::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
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
