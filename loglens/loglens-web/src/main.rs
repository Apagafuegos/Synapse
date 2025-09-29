use axum::{extract::DefaultBodyLimit, http::StatusCode, response::Json, routing::get, Router};
use performance::{create_performance_indexes, OptimizedDbOps};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer, services::ServeDir};

mod cache;
mod circuit_breaker;
mod config;
mod database;
mod error_handling;
mod handlers;
mod middleware;
mod models;
mod performance;
mod routes;
mod streaming;
mod validation;

use cache::CacheManager;
use circuit_breaker::CircuitBreakerRegistry;
use config::WebConfig;
use database::Database;
use error_handling::{
    check_cache_health, check_database_health, handle_404, trace_request, HealthStatus,
};
use handlers::websocket::status_ws_handler;
use streaming::StreamingHub;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = WebConfig::load()?;

    // Initialize database
    let db = Database::new(&config.database_url).await?;
    db.migrate().await?;

    // Create performance indexes
    if let Err(e) = create_performance_indexes(&db.pool()).await {
        tracing::warn!("Failed to create performance indexes: {}", e);
    }

    // Initialize cache manager
    let cache_manager = Arc::new(CacheManager::new());

    // Initialize circuit breaker registry
    let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());

    // Start background task to process pending analyses
    let db_pool = db.pool().clone();
    let circuit_breakers_clone = circuit_breakers.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        process_pending_analyses_task(db_pool, circuit_breakers_clone, config_clone.analysis_timeout_secs).await;
    });

    // Initialize streaming hub for real-time log streaming
    let streaming_hub = Arc::new(StreamingHub::new());

    // Initialize optimized database operations
    let optimized_db = Arc::new(OptimizedDbOps::new(
        db.pool().clone(),
        Arc::clone(&cache_manager),
    ));

    // Build application router
    let app = create_app(
        db,
        cache_manager,
        circuit_breakers,
        streaming_hub,
        optimized_db,
        config.clone(),
    )
    .await;

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    println!("🚀 LogLens web server starting on http://{}", addr);
    tracing::info!(
        "Server configuration: workers={}, max_upload={}",
        4,
        config.max_upload_size
    );

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn create_app(
    db: Database,
    cache_manager: Arc<CacheManager>,
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    streaming_hub: Arc<StreamingHub>,
    optimized_db: Arc<OptimizedDbOps>,
    config: WebConfig,
) -> Router {
    Router::new()
        .route("/health", get(enhanced_health_check))
        .nest("/api", routes::api_routes())
        .route("/ws", get(status_ws_handler))
        .nest_service("/", ServeDir::new("frontend/dist").fallback(ServeDir::new("frontend/dist/index.html")))
        .fallback(handle_404)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(trace_request))
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(config.max_upload_size)),
        )
        .with_state(AppState {
            db,
            config: config.clone(),
            circuit_breakers,
            cache_manager,
            streaming_hub,
            optimized_db,
        })
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "loglens-web",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn enhanced_health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<HealthStatus>, StatusCode> {
    let mut services = std::collections::HashMap::new();

    // Check database health
    services.insert(
        "database".to_string(),
        check_database_health(&state.db.pool()).await,
    );

    // Check cache health
    services.insert(
        "cache".to_string(),
        check_cache_health(&state.cache_manager).await,
    );

    // Determine overall status
    let overall_status = if services.values().all(|s| s.status == "healthy") {
        "healthy"
    } else {
        "degraded"
    };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    };

    Ok(Json(health_status))
}

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: WebConfig,
    pub circuit_breakers: Arc<CircuitBreakerRegistry>,
    pub cache_manager: Arc<CacheManager>,
    pub streaming_hub: Arc<StreamingHub>,
    pub optimized_db: Arc<OptimizedDbOps>,
}

async fn process_pending_analyses_task(
    db_pool: sqlx::SqlitePool,
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    timeout_secs: u64,
) {
    use models::AnalysisStatus;
    use handlers::analysis::perform_analysis;
    
    // Wait a bit for server to fully start
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    tracing::info!("Starting pending analyses processing task");
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // Get all pending analyses
        let pending_analyses = match sqlx::query!(
            "SELECT a.id, a.provider, a.level_filter, lf.upload_path 
             FROM analyses a 
             JOIN log_files lf ON a.log_file_id = lf.id 
             WHERE a.status = ? AND a.log_file_id IS NOT NULL",
            AnalysisStatus::Pending as i32
        )
        .fetch_all(&db_pool)
        .await
        {
            Ok(analyses) => analyses,
            Err(e) => {
                tracing::error!("Failed to fetch pending analyses: {}", e);
                continue;
            }
        };
        
        if !pending_analyses.is_empty() {
            tracing::info!("Found {} pending analyses to process", pending_analyses.len());
        }
        
        for analysis in pending_analyses {
            let analysis_id = match analysis.id {
                Some(id) => id,
                None => continue, // Skip if no ID
            };
            let db_pool_clone = db_pool.clone();
            let circuit_breakers_clone = circuit_breakers.clone();
            
            // Process each analysis in a separate task
            tokio::spawn(async move {
                // Update status to running
                if let Err(e) = sqlx::query!(
                    "UPDATE analyses SET status = ? WHERE id = ?",
                    AnalysisStatus::Running as i32,
                    analysis_id
                ).execute(&db_pool_clone).await {
                    tracing::error!("Failed to update analysis {} to running: {}", analysis_id, e);
                    return;
                }
                
                // Fetch API key from settings
                let api_key = match sqlx::query!("SELECT api_key FROM settings LIMIT 1")
                    .fetch_optional(&db_pool_clone)
                    .await
                {
                    Ok(Some(settings)) if !settings.api_key.is_empty() => Some(settings.api_key),
                    Ok(_) => None,
                    Err(e) => {
                        tracing::error!("Failed to fetch API key from settings: {}", e);
                        None
                    }
                };
                
                // Perform analysis
                let result = perform_analysis(
                    &analysis.upload_path,
                    &analysis.level_filter,
                    &analysis.provider,
                    api_key.as_deref(),
                    &circuit_breakers_clone,
                    timeout_secs,
                ).await;
                
                match result {
                    Ok(analysis_result) => {
                        match serde_json::to_string(&analysis_result) {
                            Ok(result_json) => {
                                if let Err(e) = sqlx::query!(
                                    "UPDATE analyses SET status = ?, result = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                                    AnalysisStatus::Completed as i32,
                                    result_json,
                                    analysis_id
                                ).execute(&db_pool_clone).await {
                                    tracing::error!("Failed to update analysis {} with result: {}", analysis_id, e);
                                } else {
                                    tracing::info!("Analysis {} completed successfully", analysis_id);
                                }
                            }
                            Err(e) => {
                                let error_msg = format!("Serialization error: {}", e);
                                if let Err(e) = sqlx::query!(
                                    "UPDATE analyses SET status = ?, error_message = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                                    AnalysisStatus::Failed as i32,
                                    error_msg,
                                    analysis_id
                                ).execute(&db_pool_clone).await {
                                    tracing::error!("Failed to update analysis {} with serialization error: {}", analysis_id, e);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let error_msg = error.to_string();
                        if let Err(e) = sqlx::query!(
                            "UPDATE analyses SET status = ?, error_message = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                            AnalysisStatus::Failed as i32,
                            error_msg,
                            analysis_id
                        ).execute(&db_pool_clone).await {
                            tracing::error!("Failed to update analysis {} with error: {}", analysis_id, e);
                        } else {
                            tracing::info!("Analysis {} failed: {}", analysis_id, error_msg);
                        }
                    }
                }
            });
        }
    }
}
