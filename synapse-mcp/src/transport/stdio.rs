use rmcp::{transport::io::stdio, service::serve_server};
use std::sync::Arc;
use crate::server::SynapseMcpHandler;

pub async fn run_stdio_server(handler: Arc<SynapseMcpHandler>) -> anyhow::Result<()> {
    // NO LOGGING - stdio transport requires pure JSON-RPC on stdout

    // Get stdio transport
    let (stdin, stdout) = stdio();

    // DON'T unwrap Arc - clone the handler for the service
    let handler_clone = (*handler).clone();

    // Create and run the service - let it run to completion
    let _service = serve_server(handler_clone, (stdin, stdout)).await?;

    // The service should run until the connection is closed
    std::future::pending::<()>().await;

    Ok(())
}