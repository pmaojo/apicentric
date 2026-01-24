use std::io::Write;
use std::process::Command;

use crate::errors::{ApicentricError, ApicentricResult};
use tempfile::NamedTempFile;

use super::openapi;
use crate::simulator::config::ServiceDefinition;

/// Generate TypeScript types for a service definition using `openapi-typescript`
///
/// This helper converts the simulator's `ServiceDefinition` into an OpenAPI
/// 3.0 spec and runs `openapi-typescript` to produce TypeScript definitions.
pub fn to_typescript(service: &ServiceDefinition) -> ApicentricResult<String> {
    // Convert to OpenAPI 3.0
    let spec = openapi::to_openapi(service);
    let spec_json = serde_json::to_string(&spec).map_err(ApicentricError::Json)?;

    // Write spec to temporary file
    let mut spec_file = NamedTempFile::new().map_err(ApicentricError::Io)?;
    spec_file
        .write_all(spec_json.as_bytes())
        .map_err(ApicentricError::Io)?;
    let spec_path = spec_file.path().to_path_buf();

    // Run openapi-typescript
    let output = Command::new("npx")
        .args(["-y", "openapi-typescript", spec_path.to_str().unwrap()])
        .output()
        .map_err(ApicentricError::Io)?;
    if !output.status.success() {
        return Err(ApicentricError::Runtime {
            message: format!(
                "openapi-typescript failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
            suggestion: Some("Ensure npx and openapi-typescript are available".to_string()),
        });
    }

    let types = String::from_utf8(output.stdout).map_err(|e| ApicentricError::Data {
        message: format!("Failed to parse openapi-typescript output: {}", e),
        suggestion: None,
    })?;
    Ok(types)
}
