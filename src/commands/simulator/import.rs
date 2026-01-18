use apicentric::{ApicentricError, ApicentricResult, ExecutionContext};
<<<<<<< HEAD
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
=======
use serde_yaml::Value;
>>>>>>> origin/main

pub async fn handle_import(
    input: &str,
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
<<<<<<< HEAD
            "🏃 Dry run: Would auto-detect and import '{}' into service '{}'",
=======
            "🏃 Dry run: Would import OpenAPI '{}' into service '{}'",
>>>>>>> origin/main
            input, output
        );
        return Ok(());
    }
<<<<<<< HEAD

    let content = std::fs::read_to_string(input).map_err(|e| {
        ApicentricError::runtime_error(
            format!("Failed to read input file: {}", e),
            Some("Ensure the file is accessible and encoded as UTF-8"),
        )
    })?;

    if let Ok(json_value) = serde_json::from_str::<JsonValue>(&content) {
        if is_openapi(&json_value) {
            println!("Detected OpenAPI format.");
            return import_openapi(&content, output).await;
        }
        if is_postman(&json_value) {
            println!("Detected Postman collection format.");
            return import_postman_from_json(&json_value, output).await;
        }
        if is_wiremock(&json_value) {
            println!("Detected WireMock mapping format.");
            return import_wiremock_from_json(&json_value, output).await;
        }
        if is_mockoon(&json_value) {
            println!("Detected Mockoon environment format.");
            return import_mockoon_from_json(&json_value, output).await;
        }
    }

    println!(
        "Could not detect specific JSON format, attempting to parse as OpenAPI (YAML/JSON)..."
    );
    import_openapi(&content, output).await
}

fn is_openapi(value: &JsonValue) -> bool {
    value.get("openapi").is_some() || value.get("swagger").is_some()
}

fn is_postman(value: &JsonValue) -> bool {
    value
        .get("info")
        .and_then(|i| i.get("_postman_id"))
        .is_some()
}

fn is_wiremock(value: &JsonValue) -> bool {
    if value.is_object() && value.get("request").is_some() && value.get("response").is_some() {
        return true;
    }
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            return first.get("request").is_some() && first.get("response").is_some();
        }
    }
    false
}

fn is_mockoon(value: &JsonValue) -> bool {
    value.get("source").is_some() && value.get("routes").is_some() && value.get("id").is_some()
}

async fn import_openapi(content: &str, output: &str) -> ApicentricResult<()> {
    let spec: YamlValue = serde_yaml::from_str(content).map_err(|e| {
        ApicentricError::validation_error(
            e.to_string(),
            Option::<String>::None,
            Option::<String>::None,
        )
    })?;
    let service = apicentric::simulator::openapi::from_openapi(&spec);
    write_service_file(service, output, "OpenAPI")
}

async fn import_postman_from_json(value: &JsonValue, output: &str) -> ApicentricResult<()> {
    let service = apicentric::simulator::postman::from_json(value).map_err(|e| {
        ApicentricError::validation_error(
            e.to_string(),
            Option::<String>::None,
            Option::<String>::None,
        )
    })?;
    write_service_file(service, output, "Postman")
}

async fn import_wiremock_from_json(value: &JsonValue, output: &str) -> ApicentricResult<()> {
    let service = apicentric::simulator::wiremock::from_json(value).map_err(|e| {
        ApicentricError::validation_error(
            e.to_string(),
            Option::<String>::None,
            Option::<String>::None,
        )
    })?;
    write_service_file(service, output, "WireMock")
}

async fn import_mockoon_from_json(value: &JsonValue, output: &str) -> ApicentricResult<()> {
    let service = apicentric::simulator::mockoon::from_json(value).map_err(|e| {
        ApicentricError::validation_error(
            e.to_string(),
            Option::<String>::None,
            Option::<String>::None,
        )
    })?;
    write_service_file(service, output, "Mockoon")
}

fn write_service_file(
    service: apicentric::simulator::config::ServiceDefinition,
    output: &str,
    format_name: &str,
) -> ApicentricResult<()> {
    let yaml = serde_yaml::to_string(&service)
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), Option::<String>::None))?;
    std::fs::write(output, yaml)?;
    println!("✅ Imported {} file to {}", format_name, output);
=======
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
>>>>>>> origin/main
    Ok(())
}
