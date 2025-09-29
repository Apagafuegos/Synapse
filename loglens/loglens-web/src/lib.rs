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
    pub optimized_db: Arc<OptimizedDbOps>,
}

impl AppState {
    pub async fn new(config: WebConfig) -> anyhow::Result<Self> {
        // Initialize database
        let db = Database::new(&config.database_url).await?;
        db.migrate().await?;

        // Create performance indexes
        if let Err(e) = performance::create_performance_indexes(&db.pool()).await {
            tracing::warn!("Failed to create performance indexes: {}", e);
        }

        // Initialize cache manager
        let cache_manager = Arc::new(CacheManager::new());

        // Initialize circuit breaker registry
        let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());

        // Initialize streaming hub
        let streaming_hub = Arc::new(streaming::StreamingHub::new());

        // Initialize optimized database operations
        let optimized_db = Arc::new(OptimizedDbOps::new(
            db.pool().clone(),
            Arc::clone(&cache_manager)
        ));

        Ok(Self {
            db,
            config,
            circuit_breakers,
            cache_manager,
            streaming_hub,
            optimized_db,
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
        services.insert("database".to_string(), error_handling::check_database_health(&self.db.pool()).await);
        
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

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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