// LogLens MCP Server - stdio transport for Model Context Protocol
//
// This binary runs the LogLens MCP server using stdio transport,
// suitable for integration with Claude Desktop and other MCP clients.

use loglens_core::mcp_server::LogLensServer;
use rmcp::{
    handler::server::router::Router,
    serve_server,
    transport::io::stdio,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to stderr (stdout is used for MCP protocol)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("loglens=info".parse().unwrap())
                .add_directive("loglens_core=info".parse().unwrap()),
        )
        .init();

    info!("Starting LogLens MCP server");

    // Create MCP server instance with router
    let server = Router::new(LogLensServer::new());

    // Get stdio transport
    let (stdin, stdout) = stdio();

    info!("MCP server listening on stdio");

    // Serve the MCP server over stdio
    let _running = serve_server(server, (stdin, stdout)).await?;

    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;

    info!("Shutting down MCP server");

    Ok(())
}
