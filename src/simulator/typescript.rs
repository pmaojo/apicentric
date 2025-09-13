use std::io::Write;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use tempfile::NamedTempFile;

use crate::simulator::config::ServiceDefinition;
use super::openapi;

/// Generate TypeScript types for a service definition using `openapi-typescript`
///
/// This helper converts the simulator's `ServiceDefinition` into an OpenAPI
/// (Swagger 2.0) spec, upgrades it to OpenAPI 3 using `swagger2openapi`, and
/// finally runs `openapi-typescript` to produce TypeScript definitions.
pub fn to_typescript(service: &ServiceDefinition) -> Result<String> {
    // Convert to OpenAPI (Swagger 2.0)
    let spec = openapi::to_openapi(service);
    let spec_json = serde_json::to_string(&spec).context("serialize OpenAPI spec")?;

    // Write spec to temporary file
    let mut spec_v2 = NamedTempFile::new().context("create temp spec file")?;
    spec_v2
        .write_all(spec_json.as_bytes())
        .context("write spec to temp file")?;
    let spec_v2_path = spec_v2.path().to_path_buf();

    // Prepare temporary file for OpenAPI 3 output
    let spec_v3 = NamedTempFile::new().context("create temp OpenAPI3 file")?;
    let spec_v3_path = spec_v3.path().to_path_buf();

    // Convert Swagger 2 -> OpenAPI 3
    let status = Command::new("npx")
        .args([
            "-y",
            "swagger2openapi",
            spec_v2_path.to_str().unwrap(),
            "--outfile",
            spec_v3_path.to_str().unwrap(),
        ])
        .status()
        .context("running swagger2openapi")?;
    if !status.success() {
        return Err(anyhow!("swagger2openapi failed"));
    }

    // Run openapi-typescript
    let output = Command::new("npx")
        .args(["-y", "openapi-typescript", spec_v3_path.to_str().unwrap()])
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

