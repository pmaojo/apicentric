// Disabled heavy P2P dependencies for lighter CLI build  
// use crate::collab::share;
use crate::commands::shared::{scaffold_endpoint_definition, scaffold_service_definition};
// use libp2p::PeerId;
use apicentric::{Context, ExecutionContext, ApicentricError, ApicentricResult};

pub async fn handle_record(
    context: &Context,
    output: &str,
    url: &Option<String>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
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
        Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ))
    }
}

pub async fn handle_share(
    context: &Context,
    service: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would share service '{}'", service);
        return Ok(());
    }
    // Disabled heavy P2P sharing functionality for lighter CLI build
    println!("📡 Sharing feature temporarily disabled for lighter CLI build");
    println!("   Service: {}", service);
    println!("   (P2P sharing will be re-enabled in future versions)");
    Ok(())
}

pub async fn handle_connect(
    peer: &str,
    service: &str,
    port: u16,
    token: Option<&str>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would connect to peer '{}' service '{}' on port {}",
            peer, service, port
        );
        return Ok(());
    }
    // Disabled heavy P2P connection functionality for lighter CLI build
    println!("🔗 Connection feature temporarily disabled for lighter CLI build");
    println!("   Peer: {}", peer);
    println!("   Service: {}", service);
    println!("   Port: {}", port);
    if let Some(t) = token {
        println!("   Token: {}", t);
    }
    println!("   (P2P connection will be re-enabled in future versions)");
    Ok(())
}

pub async fn handle_new(output: &str, exec_ctx: &ExecutionContext) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would scaffold new service in {}", output);
        return Ok(());
    }

    let service = scaffold_service_definition()?;
    std::fs::create_dir_all(output).map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to create directory {}: {}", output, e),
            None::<String>,
        )
    })?;
    let file_path = std::path::Path::new(output).join(format!("{}.yaml", service.name));
    if file_path.exists() {
        return Err(ApicentricError::fs_error(
            format!("File {} already exists", file_path.display()),
            Some("Choose a different service name"),
        ));
    }
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(&file_path, yaml).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Created service definition at {}", file_path.display());
    Ok(())
}

pub async fn handle_edit(input: &str, exec_ctx: &ExecutionContext) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would edit service {}", input);
        return Ok(());
    }

    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let mut service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;

    let endpoint = scaffold_endpoint_definition()?;
    service.endpoints.push(endpoint);

    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(input, yaml).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✏️  Updated service at {}", input);
    Ok(())
}
