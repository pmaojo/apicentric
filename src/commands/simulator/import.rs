use apicentric::{ApicentricError, ApicentricResult, ExecutionContext};
use serde_yaml::Value;

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
    let content = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to read OpenAPI: {}", e),
            Some("Ensure the file is accessible and encoded as UTF-8 YAML or JSON"),
        )
    })?;
    let spec: Value = serde_yaml::from_str(&content).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to parse OpenAPI: {}", e),
            Some("Ensure the document is a valid OpenAPI 2.0 or 3.x specification"),
        )
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
        ApicentricError::runtime_error(
            format!("Failed to read Mockoon: {}", e),
            Some("Ensure the file is a valid Mockoon environment export in JSON format"),
        )
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

pub async fn handle_import_wiremock(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would import WireMock '{}' into service '{}'",
            input, output
        );
        return Ok(());
    }
    let service = apicentric::simulator::wiremock::from_path(input).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to read WireMock: {}", e),
            Some(String::from(
                "Ensure the file is a WireMock stub mapping export in JSON format (mappings.json or a single stub)",
            )),
        )
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
        ApicentricError::runtime_error(
            format!("Failed to read Postman: {}", e),
            Some("Ensure the file is a valid Postman Collection v2.1 export in JSON format"),
        )
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
