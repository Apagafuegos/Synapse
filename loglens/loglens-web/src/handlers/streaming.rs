use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::{
    streaming::{StreamingLogEntry},
    AppState,
};

// Note: For now we'll use a simplified approach without persistent streaming managers
// In a production system, this would be properly integrated into the AppState

#[derive(Debug, Deserialize)]
pub struct CreateStreamingSourceRequest {
    pub name: String,
    pub source_type: String,
    pub config: serde_json::Value,
    pub parser_config: Option<ParserConfigRequest>,
    pub buffer_size: Option<usize>,
    pub batch_timeout_seconds: Option<u64>,
    pub restart_on_error: Option<bool>,
    pub max_restarts: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ParserConfigRequest {
    pub log_format: String,
    pub timestamp_format: Option<String>,
    pub level_field: Option<String>,
    pub message_field: Option<String>,
    pub metadata_fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct StreamingSourceResponse {
    pub source_id: String,
    pub name: String,
    pub source_type: String,
    pub project_id: Uuid,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct StreamingStatsResponse {
    pub active_sources: usize,
    pub active_connections: usize,
    pub total_logs_processed: u64,
    pub sources: Vec<StreamingSourceResponse>,
}

/// Create a new streaming source for a project
pub async fn create_streaming_source(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<CreateStreamingSourceRequest>,
) -> Result<Json<StreamingSourceResponse>, StatusCode> {
    // Simplified source creation - just validate the basic request
    if request.source_type.is_empty() || request.name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, return a mock source_id - this would be properly implemented with persistent managers
    let source_id = Uuid::new_v4().to_string();
    
    // Register the source with the streaming hub
    let _registered_id = state.streaming_hub.register_source(project_id, request.name.clone()).await;

    let response = StreamingSourceResponse {
        source_id,
        name: request.name,
        source_type: request.source_type,
        project_id,
        status: "active".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

/// Stop a streaming source
pub async fn stop_streaming_source(
    Path((project_id, source_id)): Path<(Uuid, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    // Remove the source from the streaming hub
    state.streaming_hub.remove_source(project_id, &source_id).await;
    Ok(StatusCode::NO_CONTENT)
}

/// List active streaming sources for a project
pub async fn list_streaming_sources(
    Path(project_id): Path<Uuid>,
    State(_state): State<AppState>,
) -> Result<Json<Vec<StreamingSourceResponse>>, StatusCode> {
    // For now, return empty list - this would be populated from the streaming hub's active sources
    let sources = Vec::new();
    Ok(Json(sources))
}

/// Get streaming statistics for a project
pub async fn get_streaming_stats(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<StreamingStatsResponse>, StatusCode> {
    let connection_count = state.streaming_hub.get_connection_count(project_id).await;
    
    let stats = StreamingStatsResponse {
        active_sources: 0, // Would be populated from streaming hub
        active_connections: connection_count,
        total_logs_processed: 0, // Would need to be tracked
        sources: Vec::new(), // Would be populated from streaming hub
    };

    Ok(Json(stats))
}

/// Manually ingest log data via HTTP POST
pub async fn ingest_logs(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(logs): Json<Vec<serde_json::Value>>,
) -> Result<StatusCode, StatusCode> {
    // Convert JSON logs to streaming log entries
    let entries: Vec<StreamingLogEntry> = logs
        .into_iter()
        .enumerate()
        .filter_map(|(i, log)| {
            // Try to parse as structured log
            let message = log.get("message")
                .or_else(|| log.get("msg"))
                .and_then(|v| v.as_str())
                .unwrap_or(&format!("Log entry {}", i))
                .to_string();

            let level = log.get("level")
                .or_else(|| log.get("severity"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(loglens_core::filter::LogLevel::Info);

            let timestamp = log.get("timestamp")
                .or_else(|| log.get("time"))
                .and_then(|v| v.as_str())
                .unwrap_or(&chrono::Utc::now().to_rfc3339())
                .to_string();

            // Extract metadata
            let mut metadata = std::collections::HashMap::new();
            if let serde_json::Value::Object(obj) = log {
                for (key, value) in obj {
                    if !["message", "msg", "level", "severity", "timestamp", "time"].contains(&key.as_str()) {
                        metadata.insert(key, value);
                    }
                }
            }

            Some(StreamingLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp: Some(timestamp),
                level: Some(level.to_string()),
                message,
                source: "http-ingest".to_string(),
                project_id,
                line_number: None,
            })
        })
        .collect();

    if !entries.is_empty() {
        // Add logs to the streaming hub
        state.streaming_hub
            .add_logs(project_id, "http-ingest", entries)
            .await
            .map_err(|_: anyhow::Error| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::ACCEPTED)
}

/// Force flush all buffers for a project
pub async fn flush_project_buffers(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    state.streaming_hub
        .flush_project_buffers(project_id)
        .await
        .map_err(|_: anyhow::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct StreamingFiltersQuery {
    pub level: Option<String>,
    pub source: Option<String>,
    pub since: Option<String>,
}

/// Get recent streaming logs (for debugging/testing)
pub async fn get_recent_logs(
    Path(project_id): Path<Uuid>,
    Query(filters): Query<StreamingFiltersQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StreamingLogEntry>>, StatusCode> {
    // This would require storing recent logs in memory or database
    // For now, return empty array as this is primarily for the streaming interface
    let logs = Vec::new();
    Ok(Json(logs))
}