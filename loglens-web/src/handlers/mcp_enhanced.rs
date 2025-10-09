use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{error_handling::AppError, models::*, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPTicketRequest {
    pub analysis_id: String,
    pub error_summary: String,
    pub affected_lines: Option<String>,
    pub root_cause: Option<String>,
    pub context_payload: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPTicketResponse {
    pub ticket_id: String,
    pub error_summary: String,
    pub affected_lines: Option<String>,
    pub root_cause: Option<String>,
    pub context_payload: String,
    pub deep_link: String,
    pub created_at: String,
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPContextRequest {
    pub analysis_id: String,
    pub detail_level: String, // "minimal", "standard", "full"
    pub include_correlations: bool,
    pub include_metrics: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPContextResponse {
    pub ticket_id: String,
    pub context_payload: String,
    pub detail_level: String,
    pub size_bytes: usize,
    pub created_at: String,
}

// Generate MCP error ticket with structured references
pub async fn generate_mcp_ticket(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<MCPTicketRequest>,
) -> Result<Json<MCPTicketResponse>, AppError> {
    // Verify analysis exists and belongs to project
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = $1 AND project_id = $2"
    )
    .bind(&req.analysis_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch analysis {} for MCP ticket: {}", req.analysis_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Analysis {} not found", req.analysis_id)))?;

    // Generate ticket ID with project prefix and timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d");
    let ticket_id = format!("{}-{}", project_id.to_uppercase(), timestamp);

    // Create minimal context payload if not provided
    let context_payload = req.context_payload.unwrap_or_else(|| {
        format!(
            "Analysis: {} | Provider: {} | Level: {}",
            analysis.id, analysis.provider, analysis.level_filter
        )
    });

    // Generate deep link to web interface
    let deep_link = if let Some(lines) = &req.affected_lines {
        format!("http://localhost:3000/analysis/{}#{}", analysis.id, lines)
    } else {
        format!("http://localhost:3000/analysis/{}", analysis.id)
    };

    let response = MCPTicketResponse {
        ticket_id: ticket_id.clone(),
        error_summary: req.error_summary,
        affected_lines: req.affected_lines,
        root_cause: req.root_cause,
        context_payload,
        deep_link,
        created_at: chrono::Utc::now().to_rfc3339(),
        project_id: project_id.clone(),
    };

    // Store ticket reference for future lookups
    let solution_data = serde_json::json!({
        "analysis_id": req.analysis_id,
        "expires_at": chrono::Utc::now() + chrono::Duration::hours(24),
        "password_protected": false,
        "allow_download": true
    });

    let ticket_id = format!("MCP Ticket: {}", response.ticket_id);
    let error_summary = response.error_summary.clone();
    let solution_json = serde_json::to_string(&solution_data)
        .map_err(|e| {
            tracing::error!("Failed to serialize MCP solution data: {}", e);
            AppError::internal(format!("Failed to serialize solution data: {}", e))
        })?;

    let tags_json = serde_json::to_string(&vec!["mcp", "ticket", "error"])
        .map_err(|e| {
            tracing::error!("Failed to serialize MCP tags: {}", e);
            AppError::internal(format!("Failed to serialize tags: {}", e))
        })?;

    let entry_id = uuid::Uuid::new_v4().to_string();
    let project_id_ref = project_id.clone();
    let now = chrono::Utc::now();

    sqlx::query!(
        "INSERT INTO knowledge_base (id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        entry_id,
        project_id_ref,
        ticket_id,
        error_summary,
        solution_json,
        tags_json,
        "high",
        true, // Public tickets
        0,
        now,
        now
    )
    .execute(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to insert MCP ticket: {}", e);
        AppError::Database(e)
    })?;

    Ok(Json(response))
}

// Get minimal context payload for MCP integration
pub async fn get_mcp_context(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
    Query(params): Query<MCPContextRequest>,
) -> Result<Json<MCPContextResponse>, AppError> {
    // Verify analysis exists
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = $1 AND project_id = $2"
    )
    .bind(&analysis_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch analysis {} for MCP context: {}", analysis_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Analysis {} not found", analysis_id)))?;

    let mut context_payload = String::new();

    // Build context based on detail level
    match params.detail_level.as_str() {
        "minimal" => {
            context_payload.push_str(&format!(
                "Analysis ID: {}\\nProvider: {}\\nLevel: {}\\nStatus: {:?}",
                analysis.id, analysis.provider, analysis.level_filter, analysis.status
            ));
        }
        "standard" => {
            context_payload.push_str(&format!(
                "Analysis ID: {}\\nProvider: {}\\nLevel: {}\\nStatus: {:?}\\nStarted: {}\\n",
                analysis.id,
                analysis.provider,
                analysis.level_filter,
                analysis.status,
                analysis.started_at
            ));

            if let Some(result) = &analysis.result {
                context_payload.push_str(&format!("Result: {}\\n", result));
            }

            if params.include_correlations {
                if let Ok(correlations) =
                    get_error_correlations_for_analysis(state.clone(), &analysis_id).await
                {
                    context_payload.push_str(&format!("Related Errors: {}\\n", correlations.len()));
                }
            }
        }
        "full" => {
            // Include all standard information
            context_payload.push_str(&format!(
                "Analysis ID: {}\\nProvider: {}\\nLevel: {}\\nStatus: {:?}\\nStarted: {}\\nCompleted: {}\\n",
                analysis.id,
                analysis.provider,
                analysis.level_filter,
                analysis.status,
                analysis.started_at,
                analysis.completed_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "N/A".to_string())
            ));

            if let Some(result) = &analysis.result {
                context_payload.push_str(&format!("Full Result: {}\\n", result));
            }

            if let Some(error) = &analysis.error_message {
                context_payload.push_str(&format!("Error: {}\\n", error));
            }

            // Include correlations if requested
            if params.include_correlations {
                if let Ok(correlations) =
                    get_error_correlations_for_analysis(state.clone(), &analysis_id).await
                {
                    context_payload.push_str("\\n=== Error Correlations ===\\n");
                    for corr in correlations {
                        context_payload.push_str(&format!(
                            "- Correlated with {} (strength: {:.2}, type: {})\\n",
                            corr.correlated_error_id,
                            corr.correlation_strength,
                            corr.correlation_type
                        ));
                    }
                }
            }

            // Include performance metrics if requested
            if params.include_metrics {
                if let Ok(metrics) =
                    get_performance_metrics_for_analysis(state.clone(), &analysis_id).await
                {
                    context_payload.push_str("\\n=== Performance Metrics ===\\n");
                    for metric in metrics {
                        context_payload.push_str(&format!(
                            "- {}: {} {} (threshold: {}, bottleneck: {})\\n",
                            metric.metric_name,
                            metric.metric_value,
                            metric.unit,
                            metric
                                .threshold_value
                                .map_or("N/A".to_string(), |v| v.to_string()),
                            metric.is_bottleneck
                        ));
                    }
                }
            }
        }
        _ => {
            return Err(AppError::bad_request("Invalid detail level. Must be: minimal, standard, or full"));
        }
    }

    let response = MCPContextResponse {
        ticket_id: format!("{}-CONTEXT", analysis_id),
        context_payload: context_payload.clone(),
        detail_level: params.detail_level,
        size_bytes: context_payload.clone().len(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

// Get MCP tickets for a project
pub async fn get_mcp_tickets(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<MCPTicketResponse>>, AppError> {
    let tickets = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
         FROM knowledge_base
         WHERE project_id = $1 AND tags LIKE '%\"mcp\"%' AND tags LIKE '%\"ticket\"%'
         ORDER BY created_at DESC LIMIT 50"
    )
    .bind(&project_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch MCP tickets for project {}: {}", project_id, e);
        AppError::Database(e)
    })?;

    let mut ticket_responses = Vec::new();

    for ticket in tickets {
        // Parse the solution field to extract ticket information
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&ticket.solution) {
            let response = MCPTicketResponse {
                ticket_id: ticket.title.replace("MCP Ticket: ", ""),
                error_summary: ticket.problem_description,
                affected_lines: None,
                root_cause: Some(ticket.solution.clone()),
                context_payload: parsed
                    .get("context_payload")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                deep_link: format!("http://localhost:3000/project/{}", project_id),
                created_at: ticket.created_at.to_rfc3339(),
                project_id: ticket.project_id,
            };
            ticket_responses.push(response);
        }
    }

    Ok(Json(ticket_responses))
}

// Helper functions
async fn get_error_correlations_for_analysis(
    state: AppState,
    analysis_id: &str,
) -> Result<Vec<ErrorCorrelation>, sqlx::Error> {
    sqlx::query_as::<_, ErrorCorrelation>(
        "SELECT id, project_id, primary_error_id, correlated_error_id, correlation_strength, correlation_type, created_at
         FROM error_correlations
         WHERE primary_error_id = $1 OR correlated_error_id = $1
         ORDER BY correlation_strength DESC"
    )
    .bind(analysis_id)
    .fetch_all(state.db.pool())
    .await
}

async fn get_performance_metrics_for_analysis(
    state: AppState,
    analysis_id: &str,
) -> Result<Vec<PerformanceMetric>, sqlx::Error> {
    sqlx::query_as::<_, PerformanceMetric>(
        "SELECT id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at
         FROM performance_metrics
         WHERE analysis_id = $1
         ORDER BY is_bottleneck DESC, metric_name"
    )
    .bind(analysis_id)
    .fetch_all(state.db.pool())
    .await
}
