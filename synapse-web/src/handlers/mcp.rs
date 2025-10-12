use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error_handling::AppError, models::*, AppState};
use synapse_core::process_mcp_request;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub request: String, // JSON string for MCP request
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub response: String,            // JSON string for MCP response
    pub analysis_id: Option<String>, // If analysis was created
}

/// Handle MCP requests via web API
pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<McpRequest>,
) -> Result<Json<McpResponse>, AppError> {
    // Verify project exists
    let _project = sqlx::query!("SELECT id FROM projects WHERE id = ?", project_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to fetch project {}: {}", project_id, e);
            AppError::Database(e)
        })?
        .ok_or_else(|| AppError::not_found(format!("Project {} not found", project_id)))?;

    // Process MCP request
    let mcp_response = process_mcp_request(&req.request)
        .await
        .map_err(|e| AppError::internal(format!("MCP request processing failed: {}", e)))?;

    // TODO: If this was an analysis request, we could store it in the database
    // For now, just return the MCP response

    Ok(Json(McpResponse {
        response: mcp_response,
        analysis_id: None,
    }))
}

/// Get analysis results formatted for MCP consumption
pub async fn get_analysis_for_mcp(
    State(state): State<AppState>,
    Path(analysis_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = ?"
    )
    .bind(&analysis_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch analysis {}: {}", analysis_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Analysis {} not found", analysis_id)))?;

    // Format for MCP consumption
    let mcp_data = serde_json::json!({
        "analysis_id": analysis.id,
        "status": match analysis.status {
            AnalysisStatus::Pending => "pending",
            AnalysisStatus::Running => "running",
            AnalysisStatus::Completed => "completed",
            AnalysisStatus::Failed => "failed",
        },
        "provider": analysis.provider,
        "level_filter": analysis.level_filter,
        "started_at": analysis.started_at,
        "completed_at": analysis.completed_at,
        "result": if analysis.status as i32 == AnalysisStatus::Completed as i32 {
            analysis.result.and_then(|r| serde_json::from_str::<Value>(&r).ok())
        } else {
            None
        },
        "error": analysis.error_message
    });

    Ok(Json(mcp_data))
}

/// List recent analyses in MCP-friendly format
pub async fn list_analyses_for_mcp(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let analyses = sqlx::query!(
        "SELECT id, analysis_type, provider, level_filter, status, started_at, completed_at
         FROM analyses WHERE project_id = ? ORDER BY started_at DESC LIMIT 20",
        project_id
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to fetch analyses for project {}: {}", project_id, e);
        AppError::Database(e)
    })?;

    let mcp_analyses: Vec<Value> = analyses
        .into_iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "type": a.analysis_type,
                "provider": a.provider,
                "level": a.level_filter,
                "status": match a.status {
                    0 => "pending",
                    1 => "running",
                    2 => "completed",
                    3 => "failed",
                    _ => "unknown"
                },
                "started_at": a.started_at,
                "completed_at": a.completed_at
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "project_id": project_id,
        "analyses": mcp_analyses,
        "count": mcp_analyses.len()
    })))
}
