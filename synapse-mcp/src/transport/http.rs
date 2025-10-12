use rmcp::transport::sse_server::SseServer;
use std::sync::Arc;
use crate::server::SynapseMcpHandler;

pub async fn run_http_server(handler: Arc<SynapseMcpHandler>, port: u16) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("Starting SSE MCP server on {}", addr);
    
    // Clone handler for the server
    let server_handler = handler.as_ref().clone();
    
    // Create and start SSE server  
    let server = SseServer::serve(addr.parse()?)
        .await?
        .with_service_directly(move || server_handler.clone());
    
    tracing::info!("SSE MCP server listening on port {}", port);
    tracing::info!("Available endpoints: SSE /sse");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down SSE MCP server...");
    server.cancel();
    
    Ok(())
}