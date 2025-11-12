use anyhow::Result;
use clap::Subcommand;
use apicentric::ApicentricResult;
use apicentric::ExecutionContext;
use apicentric::Context;
use apicentric::cloud::CloudServer;

#[derive(Subcommand)]
pub enum ApiAction {
    /// Starts the REST API server.
    Start {
        /// Port to listen on.
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

pub async fn api_command(
    action: &ApiAction,
    context: &Context,
    _exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    match action {
        ApiAction::Start { port } => {
            if let Some(sim) = context.api_simulator() {
                let server = CloudServer::new(sim.clone());
                server.start(*port).await.map_err(|e| {
                    apicentric::ApicentricError::server_error(
                        format!("Failed to start API server: {}", e),
                        None::<String>,
                    )
                })?;
            } else {
                return Err(apicentric::ApicentricError::config_error(
                    "API simulator not enabled, cannot start API server.",
                    Some("Ensure the 'simulator' section is present and enabled in your config.")
                ));
            }

            Ok(())
        }
    }
}
