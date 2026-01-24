use apicentric::{ApicentricResult, ExecutionContext};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ServiceInfo {
    name: String,
    server: ServerConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    port: u16,
}

pub async fn handle_dockerize(
    inputs: &[String],
    output: &str,
    exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    if exec_ctx.dry_run {
        println!(
            "🏃 Dry run: Would dockerize services '{:?}' to '{}'",
            inputs, output
        );
        return Ok(());
    }

    println!("🐳 Dockerizing services '{:?}' to '{}'", inputs, output);

    let output_path = Path::new(output);
    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
    }

    let mut ports = Vec::new();
    let mut service_names = Vec::new();

    let services_dir = output_path.join("services");
    fs::create_dir_all(&services_dir)?;

    for input in inputs {
        let service_content = fs::read_to_string(input).map_err(|e| {
            apicentric::ApicentricError::fs_error(
                format!("Failed to read service file '{}': {}", input, e),
                Some("Check if the file exists and is readable"),
            )
        })?;
        let service_def: ServiceInfo = serde_yaml::from_str(&service_content).map_err(|e| {
            apicentric::ApicentricError::validation_error(
                format!("Failed to parse service definition '{}': {}", input, e),
                None::<String>,
                Some("Ensure the file is valid YAML and contains 'name' and 'server.port' fields."),
            )
        })?;
        ports.push(service_def.server.port);
        service_names.push(service_def.name.clone());

        let service_filename = Path::new(input).file_name().unwrap().to_str().unwrap();
        fs::copy(input, services_dir.join(service_filename))?;
    }

    let expose_ports = ports
        .iter()
        .map(|p| format!("EXPOSE {}", p))
        .collect::<Vec<String>>()
        .join("\n");

    let dockerfile_content = format!(
        r#"
# Stage 1: Build the apicentric binary
FROM rust:1.78 as builder

# Install apicentric from crates.io
# This makes the Dockerfile portable and not dependent on local source code.
# You can pin to a specific version using --version <VERSION>
RUN cargo install apicentric --no-default-features --features simulator

# Stage 2: Create the final minimal image
FROM debian:buster-slim

# Copy the apicentric binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/apicentric /usr/local/bin/apicentric

# Create a directory for the service definitions
WORKDIR /app
COPY --chown=root:root services/ ./services/

# Expose the ports from the service definitions
{}

# Run the apicentric simulator, pointing to the services directory
ENTRYPOINT ["apicentric", "simulator", "start", "--services-dir", "./services"]
"#,
        expose_ports
    );

    fs::write(output_path.join("Dockerfile"), dockerfile_content)?;

    let dockerignore_content = r#"
# Ignore build artifacts and local state
target
.git
*.db
"#;

    fs::write(output_path.join(".dockerignore"), dockerignore_content)?;

    println!("✅ Dockerized services successfully to '{}'.", output);
    println!("   - Dockerfile and .dockerignore created.");
    for input in inputs {
        let service_filename = Path::new(input).file_name().unwrap().to_str().unwrap();
        println!(
            "   - Service '{}' copied into 'services/' directory.",
            service_filename
        );
    }
    println!("\nTo build the image, run:");
    println!(
        "   cd {} && docker build -t {}-service .",
        output,
        service_names.join("-").to_lowercase().replace(' ', "-")
    );

    Ok(())
}
