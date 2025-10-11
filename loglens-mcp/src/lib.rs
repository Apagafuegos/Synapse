use sqlx::{SqlitePool, Row};

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

        tracing::debug!("Creating database with URL: {}", database_url);

        // Parse database URL
        let opts = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);

        // Create connection pool
        let pool = SqlitePool::connect_with(opts).await?;

        // Try to run migrations with better error handling
        tracing::debug!("Running database migrations...");
        match sqlx::migrate!("../loglens-web/migrations")
            .run(&pool)
            .await 
        {
            Ok(_) => {
                tracing::info!("Database migrations completed successfully");
            }
            Err(e) => {
                let error_msg = e.to_string();
                tracing::error!("Migration error: {}", error_msg);
                
                // If it's a table already exists error, check if we can continue
                if error_msg.contains("already exists") {
                    tracing::warn!("Database tables already exist. This is likely okay if schema is compatible.");
                    
                    // Verify essential tables exist
                    let table_count_result = sqlx::query(
                        "SELECT COUNT(*) as count FROM sqlite_master WHERE type='table' AND name IN ('projects', 'analyses', 'log_files')"
                    )
                    .fetch_one(&pool)
                    .await;
                    
                    let tables_exist = match table_count_result {
                        Ok(row) => {
                            let count = row.get::<i64, _>("count");
                            tracing::debug!("Table count query result: {}", count);
                            count
                        },
                        Err(e) => {
                            tracing::error!("Failed to check table existence: {}", e);
                            return Err(anyhow::anyhow!("Failed to verify database schema: {}", e));
                        }
                    };
                    
                    tracing::debug!("Found {} essential tables", tables_exist);
                    tracing::info!("Expected 3 essential tables (projects, analyses, log_files), found {}", tables_exist);
                    
                    if tables_exist >= 3 {
                        tracing::info!("Essential database tables exist, proceeding...");
                    } else {
                        return Err(anyhow::anyhow!("Database is in inconsistent state: expected 3 essential tables, found {}", tables_exist));
                    }
                } else {
                    return Err(anyhow::anyhow!("Database migration failed: {}", e));
                }
            }
        }

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

// Re-export the server and handler structs
pub use server::{McpServer, LogLensMcpHandler};