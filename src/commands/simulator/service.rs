use crate::commands::shared::{scaffold_endpoint_definition, scaffold_service_definition};
use apicentric::{Context, ExecutionContext, ApicentricError, ApicentricResult};

#[cfg(feature = "p2p")]
use apicentric::collab::share;
#[cfg(feature = "p2p")]
use libp2p::PeerId;

pub async fn handle_record(
    context: &Context,
    output: &str,
    url: &Option<String>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    let target = url
        .clone()
        .unwrap_or_else(|| "http://localhost:8080".to_string());
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

#[cfg(feature = "p2p")]
pub async fn handle_share(
    _context: &Context,
    service: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would share service '{}'", service);
        return Ok(());
    }
    
    // For now, we'll use a default port. In a full implementation, this would
    // query the running service from the simulator to get its actual port.
    // This is a simplified implementation for the P2P feature gate demonstration.
    let port = 8080; // Default port - should be retrieved from running service
    
    println!("📡 Sharing service '{}' over P2P...", service);
    println!("   Note: Using default port {}. Ensure service is running on this port.", port);
    
    let (peer_id, token) = share::share_service(port).await.map_err(|e| {
        ApicentricError::config_error(
            &format!("Failed to share service: {}", e),
            Some("Check network connectivity and firewall settings"),
        )
    })?;
    
    println!("✅ Service shared successfully!");
    println!("   Peer ID: {}", peer_id);
    println!("   Token: {}", token);
    println!("   Port: {}", port);
    println!("\nOthers can connect with:");
    println!("   apicentric simulator connect --peer {} --service {} --port <local-port> --token {}", 
             peer_id, service, token);
    
    Ok(())
}

#[cfg(not(feature = "p2p"))]
pub async fn handle_share(
    _context: &Context,
    service: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would share service '{}'", service);
        return Ok(());
    }
    
    println!("📡 P2P sharing is not available in this build");
    println!("   Service: {}", service);
    println!("\n💡 To enable P2P features, rebuild with:");
    println!("   cargo build --release --features p2p");
    
    Err(ApicentricError::config_error(
        "P2P features not available",
        Some("Rebuild with --features p2p to enable collaboration"),
    ))
}

#[cfg(feature = "p2p")]
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
    
    let token = token.ok_or_else(|| {
        ApicentricError::config_error(
            "Authentication token is required",
            Some("Use --token <TOKEN> to provide the token from the sharing peer"),
        )
    })?;
    
    let peer_id = peer.parse::<PeerId>().map_err(|e| {
        ApicentricError::config_error(
            &format!("Invalid peer ID: {}", e),
            Some("Check the peer ID format from the sharing peer"),
        )
    })?;
    
    println!("🔗 Connecting to peer '{}' for service '{}'...", peer, service);
    println!("   Local port: {}", port);
    
    share::connect_service(peer_id, token.to_string(), service.to_string(), port)
        .await
        .map_err(|e| {
            ApicentricError::config_error(
                &format!("Failed to connect to peer: {}", e),
                Some("Check network connectivity and verify peer ID and token"),
            )
        })?;
    
    Ok(())
}

#[cfg(not(feature = "p2p"))]
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
    
    println!("🔗 P2P connection is not available in this build");
    println!("   Peer: {}", peer);
    println!("   Service: {}", service);
    println!("   Port: {}", port);
    if let Some(t) = token {
        println!("   Token: {}", t);
    }
    println!("\n💡 To enable P2P features, rebuild with:");
    println!("   cargo build --release --features p2p");
    
    Err(ApicentricError::config_error(
        "P2P features not available",
        Some("Rebuild with --features p2p to enable collaboration"),
    ))
}

pub async fn handle_new(output: &str, exec_ctx: &ExecutionContext) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would scaffold new service in {}", output);
        return Ok(());
    }

    let service = scaffold_service_definition()?;
    tokio::fs::create_dir_all(output).await.map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to create directory {}: {}", output, e),
            None::<String>,
        )
    })?;
    let file_path = std::path::Path::new(output).join(format!("{}.yaml", service.name));

    let exists = tokio::fs::try_exists(&file_path).await.map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to check if file exists: {}", e),
            None::<String>,
        )
    })?;

    if exists {
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
    tokio::fs::write(&file_path, yaml).await.map_err(|e| {
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

    let yaml = tokio::fs::read_to_string(input).await.map_err(|e| {
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
    tokio::fs::write(input, yaml).await.map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✏️  Updated service at {}", input);
    Ok(())
}

pub async fn handle_new_graphql(
    name: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would create new GraphQL service '{}' in {}", name, output);
        return Ok(());
    }

    let service_name = name.to_string();
    let mut service = apicentric::simulator::config::ServiceDefinition {
        name: service_name.clone(),
        version: Some("1.0.0".to_string()),
        description: Some("A GraphQL service".to_string()),
        server: apicentric::simulator::config::ServerConfig {
            port: Some(9001),
            base_path: "/".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        },
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Vec::new(),
        graphql: None,
        behavior: None,
    };

    let gql_schema_filename = format!("{}.gql", service_name);
    let example_query_name = "helloQuery".to_string();
    let example_query_filename = format!("{}.json", example_query_name);

    service.graphql = Some(apicentric::simulator::config::GraphQLConfig {
        schema_path: gql_schema_filename.clone(),
        mocks: std::collections::HashMap::from([(
            example_query_name.clone(),
            example_query_filename.clone(),
        )]),
    });

    tokio::fs::create_dir_all(output).await.map_err(|e| {
        ApicentricError::fs_error(format!("Failed to create directory {}: {}", output, e), None::<String>)
    })?;

    let yaml_path = std::path::Path::new(output).join(format!("{}.yaml", service_name));

    let exists = tokio::fs::try_exists(&yaml_path).await.map_err(|e| {
        ApicentricError::fs_error(
            format!("Failed to check if file exists: {}", e),
            None::<String>,
        )
    })?;

    if exists {
        return Err(ApicentricError::fs_error(
            format!("File {} already exists", yaml_path.display()),
            Some("Choose a different service name"),
        ));
    }

    let yaml = serde_yaml::to_string(&service).map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    tokio::fs::write(&yaml_path, yaml).await.map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!("✅ Created GraphQL service definition at {}", yaml_path.display());

    let schema_path = std::path::Path::new(output).join(gql_schema_filename);
    let schema_content = "type Query {\n  hello: String\n}";
    tokio::fs::write(&schema_path, schema_content).await.map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!("✅ Created GraphQL schema at {}", schema_path.display());

    let mock_path = std::path::Path::new(output).join(example_query_filename);
    let mock_content = "{\n  \"data\": {\n    \"hello\": \"world\"\n  }\n}";
    tokio::fs::write(&mock_path, mock_content).await.map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!("✅ Created example mock response at {}", mock_path.display());

    Ok(())
}
