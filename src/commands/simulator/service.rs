// Disabled heavy P2P dependencies for lighter CLI build  
// use crate::collab::share;
use crate::commands::shared::{scaffold_endpoint_definition, scaffold_service_definition, scaffold_graphql_service_definition};
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

pub async fn handle_new_graphql(
    output: &str,
    name: &Option<String>,
    port: &Option<u16>,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!("🏃 Dry run: Would scaffold new GraphQL service in {}", output);
        return Ok(());
    }

    let mut service = if let (Some(name), Some(port)) = (name, port) {
        apicentric::simulator::config::ServiceDefinition {
            name: name.clone(),
            version: Some("1.0.0".to_string()),
            description: Some("A GraphQL service".to_string()),
            server: apicentric::simulator::config::ServerConfig {
                port: Some(*port),
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
        }
    } else {
        scaffold_graphql_service_definition()?
    };
    let service_name = service.name.clone();

    // Define GraphQL specific files
    let gql_schema_filename = format!("{}.gql", service_name);
    let example_query_name = "helloQuery".to_string();
    let example_query_filename = format!("{}.json", example_query_name);

    // Create GraphQL config for the YAML file
    let graphql_config = apicentric::simulator::config::GraphQLConfig {
        schema_path: gql_schema_filename.clone(),
        mocks: std::collections::HashMap::from([(
            example_query_name.clone(),
            example_query_filename.clone(),
        )]),
    };
    service.graphql = Some(graphql_config);

    // Create output directory
    std::fs::create_dir_all(output).map_err(|e| {
        ApicentricError::fs_error(format!("Failed to create directory {}: {}", output, e), None::<String>)
    })?;

    // Write main service YAML file
    let yaml_path = std::path::Path::new(output).join(format!("{}.yaml", service_name));
    if yaml_path.exists() {
        return Err(ApicentricError::fs_error(
            format!("File {} already exists", yaml_path.display()),
            Some("Choose a different service name"),
        ));
    }
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to serialize service: {}", e), None::<String>)
    })?;
    std::fs::write(&yaml_path, yaml).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write service file: {}", e), None::<String>)
    })?;
    println!("✅ Created GraphQL service definition at {}", yaml_path.display());

    // Write .gql schema file
    let schema_path = std::path::Path::new(output).join(gql_schema_filename);
    let schema_content = "type Query {\n  hello: String\n}";
    std::fs::write(&schema_path, schema_content).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write schema file: {}", e), None::<String>)
    })?;
    println!("✅ Created GraphQL schema at {}", schema_path.display());

    // Write example mock response file
    let mock_path = std::path::Path::new(output).join(example_query_filename);
    let mock_content = "{\n  \"data\": {\n    \"hello\": \"world\"\n  }\n}";
    std::fs::write(&mock_path, mock_content).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write mock file: {}", e), None::<String>)
    })?;
    println!("✅ Created example mock response at {}", mock_path.display());

    Ok(())
}
