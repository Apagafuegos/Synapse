use rmcp::{transport::io::stdio, service::serve_server};
use std::sync::Arc;
use crate::server::LogLensMcpHandler;

pub async fn run_stdio_server(handler: Arc<LogLensMcpHandler>) -> anyhow::Result<()> {
    // NO LOGGING - stdio transport requires pure JSON-RPC on stdout
    // Any log output (even to stderr via tracing) can corrupt the protocol

    let (stdin, stdout) = stdio();

    // Extract the handler from Arc for serve_server
    let handler = Arc::try_unwrap(handler).map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?;

    // Use the rmcp service pattern with stdio transport
    let _service = serve_server(handler, (stdin, stdout)).await?;

    // Keep the server running indefinitely
    // In a real implementation, you'd handle shutdown gracefully
    std::future::pending::<()>().await;

    Ok(())
}