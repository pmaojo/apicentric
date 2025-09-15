use mockforge::{ExecutionContext, PulseError, PulseResult};
use openapi::to_yaml;

pub async fn handle_export(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
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
    let spec_yaml = to_yaml(&spec).map_err(|e| {
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

pub async fn handle_export_postman(
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

pub async fn handle_export_types(
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

pub async fn handle_export_query(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export React Query hooks from '{}' to '{}'",
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
    let ts = mockforge::simulator::react_query::to_react_query(&service).map_err(|e| {
        PulseError::runtime_error(format!("Failed to generate hooks: {}", e), None::<String>)
    })?;
    std::fs::write(output, ts).map_err(|e| {
        PulseError::runtime_error(format!("Failed to write hooks file: {}", e), None::<String>)
    })?;
    println!("✅ Exported React Query hooks to {}", output);
    Ok(())
}

pub async fn handle_export_view(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export React view from '{}' to '{}'",
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
    let tsx = mockforge::simulator::react_view::to_react_view(&service).map_err(|e| {
        PulseError::runtime_error(format!("Failed to generate view: {}", e), None::<String>)
    })?;
    std::fs::write(output, tsx).map_err(|e| {
        PulseError::runtime_error(format!("Failed to write view file: {}", e), None::<String>)
    })?;
    println!("✅ Exported React view to {}", output);
    Ok(())
}
