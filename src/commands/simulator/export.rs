<<<<<<< HEAD
use super::ExportFormat;
use apicentric::{ApicentricError, ApicentricResult, ExecutionContext};
=======
use apicentric::{ExecutionContext, ApicentricError, ApicentricResult};
use openapi::to_json;
>>>>>>> origin/main

pub async fn handle_export(
    input: &str,
    output: &str,
<<<<<<< HEAD
    format: &ExportFormat,
=======
>>>>>>> origin/main
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
<<<<<<< HEAD
            "🏃 Dry run: Would export service '{}' to '{}' in {:?} format",
            input, output, format
        );
        return Ok(());
    }

=======
            "🏃 Dry run: Would export service '{}' to OpenAPI '{}'",
            input, output
        );
        return Ok(());
    }
>>>>>>> origin/main
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
<<<<<<< HEAD

    let output_content = match format {
        ExportFormat::Openapi => {
            let spec = apicentric::simulator::openapi::to_openapi(&service);
            serde_json::to_string_pretty(&spec).map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Failed to serialize OpenAPI: {}", e),
                    None::<String>,
                )
            })?
        }
        ExportFormat::Postman => {
            apicentric::simulator::postman::to_string(&service).map_err(|e| {
                ApicentricError::runtime_error(
                    format!("Failed to serialize Postman: {}", e),
                    None::<String>,
                )
            })?
        }
    };

    std::fs::write(output, output_content).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write exported file: {}", e),
            None::<String>,
        )
    })?;

    println!("✅ Exported service to {} in {:?} format", output, format);
=======
    let spec = apicentric::simulator::openapi::to_openapi(&service);
    let spec_json = to_json(&spec).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize OpenAPI: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, spec_json).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write spec file: {}", e), None::<String>)
    })?;
    println!("✅ Exported OpenAPI to {}", output);
    Ok(())
}

pub async fn handle_export_postman(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export service '{}' to Postman '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let json = apicentric::simulator::postman::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize Postman: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, json).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write collection: {}", e), None::<String>)
    })?;
    println!("✅ Exported Postman collection to {}", output);
>>>>>>> origin/main
    Ok(())
}

pub async fn handle_export_types(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export TypeScript types from '{}' to '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let ts = apicentric::simulator::typescript::to_typescript(&service).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to generate types: {}", e), None::<String>)
    })?;
    std::fs::write(output, ts).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write types file: {}", e), None::<String>)
    })?;
    println!("✅ Exported TypeScript types to {}", output);
    Ok(())
}

pub async fn handle_export_query(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export React Query hooks from '{}' to '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let ts = apicentric::simulator::react_query::to_react_query(&service).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to generate hooks: {}", e), None::<String>)
    })?;
    std::fs::write(output, ts).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write hooks file: {}", e), None::<String>)
    })?;
    println!("✅ Exported React Query hooks to {}", output);
    Ok(())
}

pub async fn handle_export_view(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would export React view from '{}' to '{}'",
            input, output
        );
        return Ok(());
    }
    let yaml = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read service: {}", e), None::<String>)
    })?;
    let service: apicentric::simulator::config::ServiceDefinition = serde_yaml::from_str(&yaml)
        .map_err(|e| {
            ApicentricError::runtime_error(format!("Invalid service YAML: {}", e), None::<String>)
        })?;
    let tsx = apicentric::simulator::react_view::to_react_view(&service).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to generate view: {}", e), None::<String>)
    })?;
    std::fs::write(output, tsx).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to write view file: {}", e), None::<String>)
    })?;
    println!("✅ Exported React view to {}", output);
    Ok(())
}
