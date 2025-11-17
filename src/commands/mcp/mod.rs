//! The `mcp` command for `apicentric`.
//!
//! This command starts the MCP server, allowing AI agents to interact with
//! `apicentric`'s mock services.

#![cfg(feature = "mcp")]

mod server;

use apicentric::{ApicentricError, ApicentricResult, Context, ExecutionContext};
use clap::Parser;
use rmcp::ServiceExt;
use server::ApicentricMcpService;
use tokio::io::{stdin, stdout};

#[derive(Parser, Debug)]
pub struct Mcp {
    /// Runs the server in test mode, processing one request and then exiting.
    #[arg(long, hide = true)]
    pub test: bool,
}

/// Starts the `apicentric` MCP server.
///
/// # Arguments
///
/// * `context` - The application context.
/// * `_exec_ctx` - The execution context.
pub async fn mcp_command(
    mcp: &Mcp,
    context: &Context,
    _exec_ctx: &ExecutionContext,
) -> ApicentricResult<()> {
    let transport = (stdin(), stdout());
    let service = ApicentricMcpService::new(context.clone());

    let server = service
        .serve(transport)
        .await
        .map_err(|e| ApicentricError::server_error(e.to_string(), None::<String>))?;

    if mcp.test {
        // In test mode, we don't want to block. The server will process the first
        // request and then exit.
    } else {
        server
            .waiting()
            .await
            .map_err(|e| ApicentricError::runtime_error(e.to_string(), None::<String>))?;
    }

    Ok(())
}
