use std::io::Write;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use tempfile::NamedTempFile;

use super::openapi;
use crate::simulator::config::ServiceDefinition;

/// Generate TypeScript types for a service definition using `openapi-typescript`
///
/// This helper converts the simulator's `ServiceDefinition` into an OpenAPI
/// 3.0 spec and runs `openapi-typescript` to produce TypeScript definitions.
pub fn to_typescript(service: &ServiceDefinition) -> Result<String> {
    // Convert to OpenAPI 3.0
    let spec = openapi::to_openapi(service);
    let spec_json = serde_json::to_string(&spec).context("serialize OpenAPI spec")?;

    // Write spec to temporary file
    let mut spec_file = NamedTempFile::new().context("create temp spec file")?;
    spec_file
        .write_all(spec_json.as_bytes())
        .context("write spec to temp file")?;
    let spec_path = spec_file.path().to_path_buf();

    // Run openapi-typescript
    let output = Command::new("npx")
        .args(["-y", "openapi-typescript", spec_path.to_str().unwrap()])
        .output()
        .context("running openapi-typescript")?;
    if !output.status.success() {
        return Err(anyhow!(
            "openapi-typescript failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let types = String::from_utf8(output.stdout).context("parse openapi-typescript output")?;
    Ok(types)
}
