use serde_json::Value;
use crate::Database;
use anyhow::Result;
use sqlx::Row;

/// List analyses for a specific project with pagination
pub async fn list_analyses(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid project_id parameter"))?;
    
    let limit: i64 = params.get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(50)
        .min(200); // Enforce maximum limit
    
    let offset: i64 = params.get("offset")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let rows = sqlx::query(
        "SELECT id, project_id, file_id, status, created_at, completed_at, error_message
         FROM analyses
         WHERE project_id = ?
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(&project_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&db.pool)
    .await?;

    let total_row: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM analyses WHERE project_id = ?"
    )
    .bind(&project_id)
    .fetch_one(&db.pool)
    .await?;

    let analyses_json: Vec<Value> = rows.into_iter().map(|row| {
        let id: String = row.get("id");
        let project_id: String = row.get("project_id");
        let file_id: String = row.get("file_id");
        let status: String = row.get("status");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
        let error_message: Option<String> = row.get("error_message");

        serde_json::json!({
            "id": id,
            "project_id": project_id,
            "file_id": file_id,
            "status": status,
            "created_at": created_at,
            "completed_at": completed_at,
            "error_message": error_message
        })
    }).collect();

    Ok(serde_json::json!({
        "analyses": analyses_json,
        "total": total_row
    }))
}

/// Get complete analysis results
pub async fn get_analysis(db: &Database, params: Value) -> Result<Value> {
    let analysis_id: String = serde_json::from_value(params["analysis_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid analysis_id parameter"))?;

    let row = sqlx::query(
        "SELECT a.id, a.project_id, a.file_id, a.status, a.created_at, a.completed_at, 
         a.error_message, a.summary, ar.errors, ar.warnings, ar.recommendations, 
         ar.patterns, ar.performance_metrics, ar.metadata
         FROM analyses a
         LEFT JOIN analysis_results ar ON a.id = ar.analysis_id
         WHERE a.id = ?"
    )
    .bind(&analysis_id)
    .fetch_optional(&db.pool)
    .await?;

    match row {
        Some(row) => {
            let id: String = row.get("id");
            let project_id: String = row.get("project_id");
            let file_id: String = row.get("file_id");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
            let error_message: Option<String> = row.get("error_message");
            let summary: Option<String> = row.get("summary");
            let errors: Option<String> = row.get("errors");
            let warnings: Option<String> = row.get("warnings");
            let recommendations: Option<String> = row.get("recommendations");
            let patterns: Option<String> = row.get("patterns");
            let performance_metrics: Option<String> = row.get("performance_metrics");
            let metadata: Option<String> = row.get("metadata");

            let errors_value: Value = errors.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Array(vec![]));
            let warnings_value: Value = warnings.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Array(vec![]));
            let recommendations_value: Value = recommendations.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Array(vec![]));
            let patterns_value: Value = patterns.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Array(vec![]));
            let performance_value: Value = performance_metrics.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Object(serde_json::Map::new()));
            let metadata_value: Value = metadata.as_ref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or(Value::Object(serde_json::Map::new()));

            Ok(serde_json::json!({
                "id": id,
                "project_id": project_id,
                "file_id": file_id,
                "status": status,
                "summary": summary,
                "errors": errors_value,
                "warnings": warnings_value,
                "recommendations": recommendations_value,
                "patterns": patterns_value,
                "performance_metrics": performance_value,
                "metadata": metadata_value,
                "created_at": created_at,
                "completed_at": completed_at,
                "error_message": error_message
            }))
        }
        None => Err(anyhow::anyhow!("Analysis not found: {}", analysis_id))
    }
}

/// Get analysis status for polling
pub async fn get_analysis_status(db: &Database, params: Value) -> Result<Value> {
    let analysis_id: String = serde_json::from_value(params["analysis_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid analysis_id parameter"))?;

    let row = sqlx::query(
        "SELECT id, status, error_message FROM analyses WHERE id = ?"
    )
    .bind(&analysis_id)
    .fetch_optional(&db.pool)
    .await?;

    match row {
        Some(row) => {
            let id: String = row.get("id");
            let status: String = row.get("status");
            let error_message: Option<String> = row.get("error_message");
            let progress = calculate_progress(&status);
            Ok(serde_json::json!({
                "id": id,
                "status": status,
                "progress": progress,
                "error_message": error_message
            }))
        }
        None => Err(anyhow::anyhow!("Analysis not found: {}", analysis_id))
    }
}

/// Calculate progress percentage based on status
fn calculate_progress(status: &str) -> i32 {
    match status {
        "pending" => 0,
        "running" => 50,
        "completed" => 100,
        "failed" => 100,
        _ => 0
    }
}