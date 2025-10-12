use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    error_handling::AppError,
    streaming::{StreamingLogEntry, sources::{StreamingSourceConfig, StreamingSourceType, ParserConfig, LogFormat}},
    AppState,
};
use std::path::PathBuf;
use tokio::time::Duration;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
) -> Result<Json<StreamingSourceResponse>, AppError> {
    // Validate request
    if request.source_type.is_empty() || request.name.is_empty() {
        return Err(AppError::bad_request("Invalid request: source_type and name are required"));
    }

    // Parse source type and create config
    let source_type = parse_source_type(&request.source_type, &request.config)?;
    let parser_config = parse_parser_config(request.parser_config.clone());

    let config = StreamingSourceConfig {
        source_type,
        project_id,
        name: request.name.clone(),
        parser_config,
        buffer_size: request.buffer_size.unwrap_or(100),
        batch_timeout: Duration::from_secs(request.batch_timeout_seconds.unwrap_or(2)),
        restart_on_error: request.restart_on_error.unwrap_or(true),
        max_restarts: request.max_restarts,
    };

    // Start source via manager
    let mut manager = state.streaming_manager.write().await;
    let source_id = manager.start_source(config).await
        .map_err(|e| {
            tracing::error!("Failed to start streaming source: {}", e);
            AppError::internal(format!("Failed to start source: {}", e))
        })?;
    drop(manager); // Release lock

    // Persist source config to database
    persist_source_config(&state, &source_id, project_id, &request).await?;

    let response = StreamingSourceResponse {
        source_id,
        name: request.name,
        source_type: request.source_type,
        project_id,
        status: "active".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    tracing::info!("Created streaming source {} for project {}", response.source_id, project_id);
    Ok(Json(response))
}

/// Stop a streaming source
pub async fn stop_streaming_source(
    Path((_project_id, source_id)): Path<(Uuid, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    // Stop source via manager
    let mut manager = state.streaming_manager.write().await;
    manager.stop_source(&source_id).await
        .map_err(|e| {
            tracing::error!("Failed to stop streaming source {}: {}", source_id, e);
            AppError::internal(format!("Failed to stop source: {}", e))
        })?;
    drop(manager); // Release lock

    // Remove from database
    delete_source_config(&state, &source_id).await?;

    tracing::info!("Stopped streaming source {}", source_id);
    Ok(StatusCode::NO_CONTENT)
}

/// List active streaming sources for a project
pub async fn list_streaming_sources(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StreamingSourceResponse>>, AppError> {
    // Note: We query sources from the database rather than the in-memory manager
    // to ensure we only return sources for this specific project

    // Convert to response format with project filtering
    // Note: The manager returns all sources, so we need to filter by project_id from database
    let rows = sqlx::query(
        "SELECT id, name, source_type, project_id, status, created_at
         FROM streaming_sources
         WHERE project_id = ? AND status = 'active'"
    )
    .bind(project_id.to_string())
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch streaming sources for project {}: {}", project_id, e);
        AppError::Database(e)
    })?;

    let source_responses: Vec<StreamingSourceResponse> = rows
        .into_iter()
        .filter_map(|row| {
            Some(StreamingSourceResponse {
                source_id: row.try_get("id").ok()?,
                name: row.try_get("name").ok()?,
                source_type: row.try_get("source_type").ok()?,
                project_id: Uuid::parse_str(&row.try_get::<String, _>("project_id").ok()?).ok()?,
                status: row.try_get::<String, _>("status").ok()?,
                created_at: row.try_get::<String, _>("created_at").ok()?,
            })
        })
        .collect();

    tracing::debug!("Retrieved {} active streaming sources for project {}", source_responses.len(), project_id);
    Ok(Json(source_responses))
}

/// Get streaming statistics for a project
pub async fn get_streaming_stats(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<StreamingStatsResponse>, AppError> {
    let connection_count = state.streaming_hub.get_connection_count(project_id).await;

    // Get active sources from database for this project
    let rows = sqlx::query(
        "SELECT id, name, source_type, project_id, status, created_at
         FROM streaming_sources
         WHERE project_id = ? AND status = 'active'"
    )
    .bind(project_id.to_string())
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch streaming sources for project {}: {}", project_id, e);
        AppError::Database(e)
    })?;

    let source_responses: Vec<StreamingSourceResponse> = rows
        .into_iter()
        .filter_map(|row| {
            Some(StreamingSourceResponse {
                source_id: row.try_get("id").ok()?,
                name: row.try_get("name").ok()?,
                source_type: row.try_get("source_type").ok()?,
                project_id: Uuid::parse_str(&row.try_get::<String, _>("project_id").ok()?).ok()?,
                status: row.try_get::<String, _>("status").ok()?,
                created_at: row.try_get::<String, _>("created_at").ok()?,
            })
        })
        .collect();

    let active_sources = source_responses.len();

    let stats = StreamingStatsResponse {
        active_sources,
        active_connections: connection_count,
        total_logs_processed: 0, // TODO: Track this in future
        sources: source_responses,
    };

    Ok(Json(stats))
}

/// Manually ingest log data via HTTP POST
pub async fn ingest_logs(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(logs): Json<Vec<serde_json::Value>>,
) -> Result<StatusCode, AppError> {
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
                .unwrap_or(synapse_core::filter::LogLevel::Info);

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
            .map_err(|e: anyhow::Error| {
                tracing::error!("Failed to add logs to streaming hub: {}", e);
                AppError::internal(format!("Failed to add logs: {}", e))
            })?;
    }

    Ok(StatusCode::ACCEPTED)
}

/// Force flush all buffers for a project
pub async fn flush_project_buffers(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    state.streaming_hub
        .flush_project_buffers(project_id)
        .await
        .map_err(|e: anyhow::Error| {
            tracing::error!("Failed to flush buffers for project {}: {}", project_id, e);
            AppError::internal(format!("Failed to flush buffers: {}", e))
        })?;

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
    Path(_project_id): Path<Uuid>,
    Query(_filters): Query<StreamingFiltersQuery>,
    State(_state): State<AppState>,
) -> Result<Json<Vec<StreamingLogEntry>>, AppError> {
    // This would require storing recent logs in memory or database
    // For now, return empty array as this is primarily for the streaming interface
    let logs = Vec::new();
    Ok(Json(logs))
}

// Helper functions for parsing source configurations

/// Parse source type from request
fn parse_source_type(source_type: &str, config: &serde_json::Value) -> Result<StreamingSourceType, AppError> {
    match source_type {
        "file" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'path' for file source"))?;
            Ok(StreamingSourceType::File { path: PathBuf::from(path) })
        }
        "command" => {
            let command = config.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'command'"))?;
            let args = config.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Ok(StreamingSourceType::Command { command: command.to_string(), args })
        }
        "tcp" => {
            let port = config.get("port")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| AppError::bad_request("Missing 'port' for TCP source"))?;
            Ok(StreamingSourceType::TcpListener { port: port as u16 })
        }
        "stdin" => Ok(StreamingSourceType::Stdin),
        "http" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'path' for HTTP source"))?;
            Ok(StreamingSourceType::HttpEndpoint { path: path.to_string() })
        }
        _ => Err(AppError::bad_request(format!("Unknown source type: {}", source_type)))
    }
}

/// Parse parser config from request
fn parse_parser_config(req: Option<ParserConfigRequest>) -> ParserConfig {
    let req = match req {
        Some(r) => r,
        None => return ParserConfig::default(),
    };

    let log_format = match req.log_format.as_str() {
        "json" => LogFormat::Json,
        "syslog" => LogFormat::Syslog,
        "common" => LogFormat::CommonLog,
        _ => LogFormat::Text,
    };

    ParserConfig {
        log_format,
        timestamp_format: req.timestamp_format,
        level_field: req.level_field,
        message_field: req.message_field,
        metadata_fields: req.metadata_fields.unwrap_or_default(),
    }
}

/// Persist streaming source configuration to database
async fn persist_source_config(
    state: &AppState,
    source_id: &str,
    project_id: Uuid,
    request: &CreateStreamingSourceRequest,
) -> Result<(), AppError> {
    let config_json = serde_json::to_string(&request.config)
        .map_err(|e| AppError::internal(format!("Failed to serialize config: {}", e)))?;

    let parser_config_json = request.parser_config.as_ref()
        .map(|p| serde_json::to_string(p))
        .transpose()
        .map_err(|e| AppError::internal(format!("Failed to serialize parser config: {}", e)))?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO streaming_sources
         (id, project_id, name, source_type, config, parser_config,
          buffer_size, batch_timeout_seconds, restart_on_error, max_restarts,
          status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)"
    )
    .bind(source_id)
    .bind(project_id.to_string())
    .bind(&request.name)
    .bind(&request.source_type)
    .bind(&config_json)
    .bind(parser_config_json.as_ref())
    .bind(request.buffer_size.map(|s| s as i64))
    .bind(request.batch_timeout_seconds.map(|s| s as i64))
    .bind(request.restart_on_error)
    .bind(request.max_restarts.map(|m| m as i64))
    .bind(&now)
    .bind(&now)
    .execute(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to persist streaming source {}: {}", source_id, e);
        AppError::Database(e)
    })?;

    tracing::info!("Persisted streaming source {} to database", source_id);
    Ok(())
}

/// Delete streaming source configuration from database
async fn delete_source_config(state: &AppState, source_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM streaming_sources WHERE id = ?")
        .bind(source_id)
        .execute(state.db.pool())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete streaming source {}: {}", source_id, e);
            AppError::Database(e)
        })?;

    tracing::info!("Deleted streaming source {} from database", source_id);
    Ok(())
}