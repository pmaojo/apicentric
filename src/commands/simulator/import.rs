use apicentric::{ExecutionContext, ApicentricError, ApicentricResult};
use openapi::from_path;

pub async fn handle_import(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import OpenAPI '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let spec = from_path(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read OpenAPI: {}", e), None::<String>)
    })?;
    let service = apicentric::simulator::openapi::from_openapi(&spec);
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, yaml).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Imported service to {}", output);
    Ok(())
}

pub async fn handle_import_mockoon(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import Mockoon '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let service = apicentric::simulator::mockoon::from_path(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read Mockoon: {}", e), None::<String>)
    })?;
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, yaml).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Imported service to {}", output);
    Ok(())
}

pub async fn handle_import_postman(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import Postman '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let service = apicentric::simulator::postman::from_path(input).map_err(|e| {
        ApicentricError::runtime_error(format!("Failed to read Postman: {}", e), None::<String>)
    })?;
    let yaml = serde_yaml::to_string(&service).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to serialize service: {}", e),
            None::<String>,
        )
    })?;
    std::fs::write(output, yaml).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to write service file: {}", e),
            None::<String>,
        )
    })?;
    println!("✅ Imported service to {}", output);
    Ok(())
}
