use axum::{
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};

use crate::{handlers, middleware, streaming, AppState};

pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Dashboard routes
        .route("/dashboard/stats", get(handlers::get_dashboard_stats))
        // Project routes
        .route("/projects", get(handlers::list_projects))
        .route("/projects", post(handlers::create_project))
        .route("/projects/:id", get(handlers::get_project))
        .route("/projects/:id", delete(handlers::delete_project))
        // File routes
        .route("/projects/:id/files", get(handlers::list_log_files))
        .route("/projects/:id/files", post(handlers::upload_log_file))
        .route(
            "/projects/:project_id/files/:file_id",
            delete(handlers::delete_log_file),
        )
        // Analysis routes
        .route(
            "/projects/:project_id/files/:file_id/analyze",
            post(handlers::start_analysis),
        )
        .route(
            "/projects/:project_id/files/:file_id/analyze/ws",
            get(handlers::websocket::websocket_analysis_handler),
        )
        .route("/analyses/:id", get(handlers::get_analysis))
        .route("/projects/:id/analyses", get(handlers::list_analyses))
        .route("/analyses/:id/performance-metrics", get(handlers::get_performance_metrics))
        .route("/projects/:id/error-correlations", get(handlers::get_error_correlations))
        // MCP integration routes
        .route("/projects/:id/mcp", post(handlers::handle_mcp_request))
        .route("/analyses/:id/mcp", get(handlers::get_analysis_for_mcp))
        .route(
            "/projects/:id/mcp/analyses",
            get(handlers::list_analyses_for_mcp),
        )
        // Knowledge Base routes (Phase 4.1)
        .route(
            "/projects/:id/knowledge",
            post(handlers::create_knowledge_entry),
        )
        .route(
            "/projects/:id/knowledge",
            get(handlers::get_knowledge_entries),
        )
        .route(
            "/projects/:id/knowledge/:entry_id",
            get(handlers::get_knowledge_entry),
        )
        .route("/knowledge/public", get(handlers::get_public_knowledge))
        .route("/projects/:id/patterns", get(handlers::get_error_patterns))
        .route(
            "/projects/:id/patterns/:pattern_id",
            post(handlers::update_pattern_frequency),
        )
        .route(
            "/projects/:id/recognize-patterns",
            post(handlers::recognize_patterns),
        )
        // Enhanced MCP routes (Phase 4.2)
        .route(
            "/projects/:id/mcp/tickets",
            post(handlers::generate_mcp_ticket),
        )
        .route("/projects/:id/mcp/tickets", get(handlers::get_mcp_tickets))
        .route(
            "/projects/:id/mcp/context/:analysis_id",
            get(handlers::get_mcp_context),
        )
        // Advanced Analysis routes (Phase 4.3)
        .route(
            "/projects/:id/correlations",
            post(handlers::analyze_correlations),
        )
        .route("/projects/:id/anomalies", post(handlers::detect_anomalies))
        .route(
            "/projects/:id/multi-log",
            post(handlers::analyze_multiple_logs),
        )
        // Export and Reporting routes (Phase 4.4)
        .route(
            "/projects/:id/analyses/:analysis_id/export/html",
            get(handlers::export_html_report),
        )
        .route(
            "/projects/:id/analyses/:analysis_id/export/json",
            get(handlers::export_json_data),
        )
        .route(
            "/projects/:id/analyses/:analysis_id/export/csv",
            get(handlers::export_csv_data),
        )
        .route(
            "/projects/:id/analyses/:analysis_id/export/pdf",
            get(handlers::export_pdf_report),
        )
        .route(
            "/projects/:id/analyses/:analysis_id/export/md",
            get(handlers::export_markdown_report),
        )
        .route("/projects/:id/share", post(handlers::create_share_link))
        .route("/projects/:id/exports", get(handlers::get_export_history))
        // Shared analysis access
        .route("/shared/:share_id", get(handlers::get_shared_analysis))
        // Streaming dashboard template
        .route("/projects/:project_id/dashboard", get(handlers::streaming_dashboard))
        // Settings routes
        .route("/settings", get(handlers::settings::get_settings))
        .route("/settings", axum::routing::patch(handlers::settings::update_settings))
        // Model configuration routes
        .route("/models/available", post(handlers::models::get_available_models))
        .route("/models/cache/clear", post(handlers::models::clear_models_cache))
        // Real-time streaming WebSocket endpoint (Phase 6.1)
        .route("/projects/:project_id/stream", get(streaming::websocket_handler))
        // Streaming source management routes (Phase 6.1)
        .route("/projects/:project_id/streaming/sources", post(handlers::streaming::create_streaming_source))
        .route("/projects/:project_id/streaming/sources", get(handlers::streaming::list_streaming_sources))
        .route("/projects/:project_id/streaming/sources/:source_id", delete(handlers::streaming::stop_streaming_source))
        .route("/projects/:project_id/streaming/stats", get(handlers::streaming::get_streaming_stats))
        .route("/projects/:project_id/streaming/ingest", post(handlers::streaming::ingest_logs))
        .route("/projects/:project_id/streaming/flush", post(handlers::streaming::flush_project_buffers))
        .route("/projects/:project_id/streaming/logs", get(handlers::streaming::get_recent_logs))
        // Metrics routes
        .route("/metrics", get(metrics_handler_wrapper))
        .route("/health/metrics", get(health_with_metrics_handler_wrapper))
}

/// Wrapper to extract metrics_collector from AppState for metrics endpoint
async fn metrics_handler_wrapper(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    match middleware::metrics::metrics_handler(state.metrics_collector).await {
        Ok(json) => json.into_response(),
        Err(status) => status.into_response(),
    }
}

/// Wrapper to extract metrics_collector from AppState for health endpoint
async fn health_with_metrics_handler_wrapper(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    match middleware::metrics::health_with_metrics_handler(state.metrics_collector).await {
        Ok(json) => json.into_response(),
        Err(status) => status.into_response(),
    }
}
