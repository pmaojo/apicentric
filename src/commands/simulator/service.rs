use crate::collab::share;
use crate::commands::shared::{scaffold_endpoint_definition, scaffold_service_definition};
use libp2p::PeerId;
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
                Err(e) => Err(ApicentricError::runtime_error(
                    format!("Failed to share service: {}", e),
                    None::<String>,
                )),
            }
        } else {
            println!("⚠️ Service '{}' not found", service);
            Ok(())
        }
    } else {
        Err(ApicentricError::config_error(
            "API simulator is not enabled or configured",
            Some("Enable simulator in apicentric.json"),
        ))
    }
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
    let peer_id = peer.parse::<PeerId>().map_err(|e| {
        ApicentricError::runtime_error(format!("Invalid peer id: {}", e), None::<String>)
    })?;
    let token = token.unwrap_or("").to_string();
    share::connect_service(peer_id, token, service.to_string(), port)
        .await
        .map_err(|e| ApicentricError::runtime_error(format!("Failed to connect: {}", e), None::<String>))
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
