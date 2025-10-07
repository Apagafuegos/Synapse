use sqlx::SqlitePool;

pub mod tools;
pub mod server;
pub mod transport;
pub mod schema;

/// Database wrapper for MCP server
#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

/// Configuration for MCP server
#[derive(Debug, Clone)]
pub struct Config {
    pub server_name: String,
    pub server_version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_name: "loglens-mcp".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Factory function to create MCP server
pub async fn create_server(db: Database, config: Config) -> anyhow::Result<McpServer> {
    Ok(McpServer::new(db, config))
}

// Re-export the server struct
pub use server::McpServer;