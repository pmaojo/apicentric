use mockforge::{Context, ExecutionContext, PulseError, PulseResult};

use super::control;

/// Start the simulator optionally loading environment variables from a file
pub async fn handle_start(
    context: &Context,
    services_dir: &str,
    force: bool,
    p2p: bool,
    env_file: &Option<String>,
    exec_ctx: &ExecutionContext,
) -> PulseResult<()> {
    if let Some(path) = env_file {
        dotenvy::from_filename(path).map_err(|e| {
            PulseError::config_error(
                format!("Failed to load env file {}: {}", path, e),
                Some("Ensure the file exists and has valid KEY=VALUE pairs"),
            )
        })?;
    }
    control::handle_start(context, services_dir, force, p2p, exec_ctx).await
}
