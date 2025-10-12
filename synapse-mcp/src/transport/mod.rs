use std::sync::Arc;
use crate::server::SynapseMcpHandler;

pub mod stdio;
pub mod http;

/// Transport types supported by the MCP server
#[derive(Debug, Clone)]
pub enum TransportType {
    Stdio,
    Http { port: u16 },
}

/// Create a transport instance and run the server
pub async fn create_and_run_transport(
    transport_type: TransportType,
    handler: Arc<SynapseMcpHandler>,
) -> anyhow::Result<()> {
    match transport_type {
        TransportType::Stdio => {
            stdio::run_stdio_server(handler).await?;
        }
        TransportType::Http { port } => {
            http::run_http_server(handler, port).await?;
        }
    }
    Ok(())
}