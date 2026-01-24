use crate::commands::shared::{scaffold_endpoint_definition, scaffold_service_definition};
use apicentric::{ApicentricError, ApicentricResult, Context, ExecutionContext};

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
    match service.endpoints.as_mut() {
        Some(endpoints) => endpoints.push(endpoint),
        None => service.endpoints = Some(vec![endpoint]),
    }

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
        println!(
            "🏃 Dry run: Would create new GraphQL service '{}' in {}",
            name, output
        );
        return Ok(());
    }

    let service_name = name.to_string();
    let mut service = apicentric::simulator::config::ServiceDefinition {
        name: service_name.clone(),
        version: Some("1.0.0".to_string()),
        description: Some("A GraphQL service".to_string()),
        server: Some(apicentric::simulator::config::ServerConfig {
            port: Some(9001),
            base_path: "/".to_string(),
            proxy_base_url: None,
            cors: None,
            record_unknown: false,
        }),
        models: None,
        fixtures: None,
        bucket: None,
        endpoints: Some(Vec::new()),
        graphql: None,
        behavior: None,
        twin: None,
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
        ApicentricError::fs_error(
            format!("Failed to create directory {}: {}", output, e),
            None::<String>,
        )
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

    let yaml = serde_yaml::to_string(&service)
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    tokio::fs::write(&yaml_path, yaml)
        .await
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!(
        "✅ Created GraphQL service definition at {}",
        yaml_path.display()
    );

    let schema_path = std::path::Path::new(output).join(gql_schema_filename);
    let schema_content = "type Query {\n  hello: String\n}";
    tokio::fs::write(&schema_path, schema_content)
        .await
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!("✅ Created GraphQL schema at {}", schema_path.display());

    let mock_path = std::path::Path::new(output).join(example_query_filename);
    let mock_content = "{\n  \"data\": {\n    \"hello\": \"world\"\n  }\n}";
    tokio::fs::write(&mock_path, mock_content)
        .await
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    println!(
        "✅ Created example mock response at {}",
        mock_path.display()
    );

    Ok(())
}
