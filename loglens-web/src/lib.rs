// LogLens Web Backend Library
// High-performance Rust web backend for intelligent log analysis

pub mod cache;
pub mod circuit_breaker;
pub mod config;
pub mod database;
pub mod error_handling;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod performance;
pub mod routes;
pub mod streaming;
pub mod validation;

// Re-export commonly used types and functions
pub use cache::CacheManager;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerRegistry};
pub use config::WebConfig;
pub use database::Database;
pub use error_handling::{AppError, AppResult};
pub use models::*;
pub use performance::OptimizedDbOps;

// Re-export main application factory for testing
// Re-export main components

use std::sync::Arc;

// Main application state
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: WebConfig,
    pub circuit_breakers: Arc<CircuitBreakerRegistry>,
    pub cache_manager: Arc<CacheManager>,
    pub streaming_hub: Arc<crate::streaming::StreamingHub>,
    pub streaming_manager: Arc<tokio::sync::RwLock<crate::streaming::sources::StreamingSourceManager>>,
    pub optimized_db: Arc<OptimizedDbOps>,
    pub metrics_collector: Arc<crate::middleware::metrics::MetricsCollector>,
}

impl AppState {
    pub async fn new(config: WebConfig) -> anyhow::Result<Self> {
        // Initialize database
        let db = Database::new(&config.database_url).await?;
        db.migrate().await?;

        // Create performance indexes
        if let Err(e) = performance::create_performance_indexes(db.pool()).await {
            tracing::warn!("Failed to create performance indexes: {}", e);
        }

        // Initialize cache manager
        let cache_manager = Arc::new(CacheManager::new());

        // Initialize circuit breaker registry
        let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());

        // Initialize streaming hub
        let streaming_hub = Arc::new(streaming::StreamingHub::new());

        // Initialize streaming source manager
        let streaming_manager = Arc::new(tokio::sync::RwLock::new(
            crate::streaming::sources::StreamingSourceManager::new(Arc::clone(&streaming_hub))
        ));

        // Restore active streaming sources from database
        if let Err(e) = restore_streaming_sources(&db, &streaming_manager).await {
            tracing::warn!("Failed to restore streaming sources: {}", e);
        }

        // Initialize optimized database operations
        let optimized_db = Arc::new(OptimizedDbOps::new(
            db.pool().clone(),
            Arc::clone(&cache_manager)
        ));

        // Initialize metrics collector
        let metrics_collector = Arc::new(middleware::metrics::MetricsCollector::new());
        metrics_collector.clone().start_background_tasks();

        Ok(Self {
            db,
            config,
            circuit_breakers,
            cache_manager,
            streaming_hub,
            streaming_manager,
            optimized_db,
            metrics_collector,
        })
    }

    /// Get performance statistics for monitoring
    pub async fn get_performance_stats(&self) -> serde_json::Value {
        let cache_stats = self.cache_manager.get_cache_stats();
        let monitor = self.optimized_db.get_monitor();
        let performance_summary = monitor.get_performance_summary().await;

        serde_json::json!({
            "cache": cache_stats,
            "database": {
                "total_queries": performance_summary.total_queries,
                "cache_hit_rate": performance_summary.cache_hit_rate,
                "average_response_time_ms": performance_summary.average_response_time_ms
            },
            "circuit_breakers": {
                "registry_size": "unknown" // Would need to add method to CircuitBreakerRegistry
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Clear all caches (useful for testing and debugging)
    pub fn clear_caches(&self) {
        self.cache_manager.clear_all_caches();
    }

    /// Health check for the application state
    pub async fn health_check(&self) -> error_handling::HealthStatus {
        let mut services = std::collections::HashMap::new();
        
        // Check database health
        services.insert("database".to_string(), error_handling::check_database_health(self.db.pool()).await);
        
        // Check cache health
        services.insert("cache".to_string(), error_handling::check_cache_health(&self.cache_manager).await);
        
        // Determine overall status
        let overall_status = if services.values().all(|s| s.status == "healthy") {
            "healthy"
        } else {
            "degraded"
        };

        error_handling::HealthStatus {
            status: overall_status.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            services,
        }
    }
}

/// Stored parser configuration (for deserialization from database)
#[derive(Debug, serde::Deserialize)]
struct StoredParserConfig {
    log_format: String,
    timestamp_format: Option<String>,
    level_field: Option<String>,
    message_field: Option<String>,
    metadata_fields: Option<Vec<String>>,
}

/// Restore active streaming sources from database on startup
async fn restore_streaming_sources(
    db: &Database,
    streaming_manager: &Arc<tokio::sync::RwLock<crate::streaming::sources::StreamingSourceManager>>,
) -> anyhow::Result<()> {
    use sqlx::Row;

    tracing::info!("Restoring active streaming sources from database...");

    // Query all active sources
    let rows = sqlx::query("SELECT * FROM streaming_sources WHERE status = 'active'")
        .fetch_all(db.pool())
        .await?;

    let mut manager = streaming_manager.write().await;
    let mut restored_count = 0;

    for row in rows {
        match restore_source_from_row(&row, &mut manager).await {
            Ok(source_id) => {
                tracing::info!("Restored streaming source: {}", source_id);
                restored_count += 1;
            }
            Err(e) => {
                let id: String = row.try_get("id").unwrap_or_else(|_| "unknown".to_string());
                tracing::error!("Failed to restore streaming source {}: {}", id, e);
            }
        }
    }

    drop(manager);
    tracing::info!("Restored {} streaming sources", restored_count);
    Ok(())
}

/// Helper to restore a single source from a database row
async fn restore_source_from_row(
    row: &sqlx::sqlite::SqliteRow,
    manager: &mut crate::streaming::sources::StreamingSourceManager,
) -> anyhow::Result<String> {
    use sqlx::Row;
    use uuid::Uuid;

    let source_id: String = row.try_get("id")?;
    let project_id: String = row.try_get("project_id")?;
    let project_id = Uuid::parse_str(&project_id)?;
    let name: String = row.try_get("name")?;
    let source_type_str: String = row.try_get("source_type")?;
    let config_json: String = row.try_get("config")?;
    let parser_config_json: Option<String> = row.try_get("parser_config").ok();
    let buffer_size: Option<i64> = row.try_get("buffer_size").ok();
    let batch_timeout_seconds: Option<i64> = row.try_get("batch_timeout_seconds").ok();
    let restart_on_error: Option<bool> = row.try_get("restart_on_error").ok();
    let max_restarts: Option<i64> = row.try_get("max_restarts").ok();

    // Parse source type
    let config_value: serde_json::Value = serde_json::from_str(&config_json)?;
    let source_type = parse_source_type_from_config(&source_type_str, &config_value)?;

    // Parse parser config
    let parser_config = if let Some(json_str) = parser_config_json {
        let stored_config: StoredParserConfig = serde_json::from_str(&json_str)?;
        parse_stored_parser_config(stored_config)
    } else {
        crate::streaming::sources::ParserConfig::default()
    };

    // Create source config
    let config = crate::streaming::sources::StreamingSourceConfig {
        source_type,
        project_id,
        name,
        parser_config,
        buffer_size: buffer_size.map(|s| s as usize).unwrap_or(100),
        batch_timeout: tokio::time::Duration::from_secs(batch_timeout_seconds.map(|s| s as u64).unwrap_or(2)),
        restart_on_error: restart_on_error.unwrap_or(true),
        max_restarts: max_restarts.map(|m| m as u32),
    };

    // Start the source
    manager.start_source(config).await?;

    Ok(source_id)
}

/// Parse source type from stored config
fn parse_source_type_from_config(
    source_type: &str,
    config: &serde_json::Value,
) -> anyhow::Result<crate::streaming::sources::StreamingSourceType> {
    use std::path::PathBuf;

    match source_type {
        "file" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path' for file source"))?;
            Ok(crate::streaming::sources::StreamingSourceType::File { path: PathBuf::from(path) })
        }
        "command" => {
            let command = config.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'command'"))?;
            let args = config.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Ok(crate::streaming::sources::StreamingSourceType::Command { command: command.to_string(), args })
        }
        "tcp" => {
            let port = config.get("port")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'port' for TCP source"))?;
            Ok(crate::streaming::sources::StreamingSourceType::TcpListener { port: port as u16 })
        }
        "stdin" => Ok(crate::streaming::sources::StreamingSourceType::Stdin),
        "http" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path' for HTTP source"))?;
            Ok(crate::streaming::sources::StreamingSourceType::HttpEndpoint { path: path.to_string() })
        }
        _ => Err(anyhow::anyhow!("Unknown source type: {}", source_type))
    }
}

/// Parse parser config from stored configuration
fn parse_stored_parser_config(
    stored: StoredParserConfig,
) -> crate::streaming::sources::ParserConfig {
    let log_format = match stored.log_format.as_str() {
        "json" => crate::streaming::sources::LogFormat::Json,
        "syslog" => crate::streaming::sources::LogFormat::Syslog,
        "common" => crate::streaming::sources::LogFormat::CommonLog,
        _ => crate::streaming::sources::LogFormat::Text,
    };

    crate::streaming::sources::ParserConfig {
        log_format,
        timestamp_format: stored.timestamp_format,
        level_field: stored.level_field,
        message_field: stored.message_field,
        metadata_fields: stored.metadata_fields.unwrap_or_default(),
    }
}

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Start the web server with the given port
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    use axum::{extract::DefaultBodyLimit, routing::get, Router};
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use tokio::net::TcpListener;
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer, services::{ServeDir, ServeFile}};

    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Initialize tracing (only if not already initialized)
    // Use try_init() to avoid panic when called from CLI which already initializes tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    tracing::info!("Starting LogLens web server on port {}", port);

    // Load configuration
    let mut config = WebConfig::load()?;
    config.port = port;

    // Determine frontend directory path
    let mut frontend_path = PathBuf::from(&config.frontend_dir);

    // Canonicalize to absolute path if the path exists
    // This is critical for Windows compatibility with ServeDir
    if frontend_path.exists() {
        match frontend_path.canonicalize() {
            Ok(canonical_path) => {
                tracing::info!("Canonicalized frontend path: {}", canonical_path.display());
                frontend_path = canonical_path;
            }
            Err(e) => {
                tracing::warn!("Failed to canonicalize frontend path: {}", e);
            }
        }
    }

    let index_path = frontend_path.join("index.html");

    tracing::info!("Serving frontend from: {}", frontend_path.display());
    tracing::info!("Frontend path is absolute: {}", frontend_path.is_absolute());
    tracing::info!("Frontend path exists: {}", frontend_path.exists());
    tracing::info!("Index.html exists: {}", index_path.exists());

    // Create application state
    let state = AppState::new(config.clone()).await?;

    // Create SPA-compatible static file service
    let serve_dir = ServeDir::new(&frontend_path)
        .not_found_service(ServeFile::new(&index_path));

    // Build router with proper SPA fallback
    let app = Router::new()
        .route("/api/health", get(handlers::dashboard::get_dashboard_stats))
        .nest("/api", routes::api_routes())
        // Serve static files and handle SPA routing with fallback
        .fallback_service(serve_dir)
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB
        )
        .with_state(state);

    // Create server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    
    tracing::info!("Server listening on http://{}", addr);
    
    // Start server
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = WebConfig::default();
        let app_state = AppState::new(config).await;
        assert!(app_state.is_ok());
    }

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
}