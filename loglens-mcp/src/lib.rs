use sqlx::SqlitePool;

pub mod tools;
pub mod server;
pub mod transport;
pub mod schema;
pub mod validation;

/// Database wrapper for MCP server
#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        // Parse database URL
        let opts = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);

        // Create connection pool
        let pool = SqlitePool::connect_with(opts).await?;

        // Run migrations from loglens-web/migrations
        // This ensures the database has the proper schema including log_files table
        sqlx::migrate!("../loglens-web/migrations")
            .run(&pool)
            .await?;

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