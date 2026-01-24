//! The `mcp` command for `apicentric`.
//!
//! This command starts the MCP server, allowing AI agents to interact with
//! `apicentric`'s mock services.

pub mod server;

use apicentric::cli::args::Mcp;
use apicentric::{ApicentricError, ApicentricResult, Context, ExecutionContext};
use rmcp::ServiceExt;
use server::ApicentricMcpService;
use tokio::io::{stdin, stdout};

/// Starts the `apicentric` MCP server.
///
/// # Arguments
///
/// * `context` - The application context.
/// * `_exec_ctx` - The execution context.
pub async fn mcp_command(
    _mcp: &Mcp,
    context: &Context,
    _exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    // We use stderr for logging to avoid corrupting the JSON-RPC stdout stream.
    // This is handled globally in logging initialization, but good to keep in mind.

    let transport = (stdin(), stdout());
    let service = ApicentricMcpService::new(context.clone());

    let server = service
        .serve(transport)
        .await
        .map_err(|e| ApicentricError::server_error(e.to_string(), None::<String>))?;

    // Always wait for the server to complete (which happens when stdin closes or error occurs).
    // The --test flag logic was previously exiting too early.
    server
        .waiting()
        .await
        .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;

    Ok(())
}
