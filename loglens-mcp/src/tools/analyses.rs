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
        "SELECT id, project_id, log_file_id, status, started_at, completed_at, error_message
         FROM analyses
         WHERE project_id = ?
         ORDER BY started_at DESC
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
        let log_file_id: Option<String> = row.get("log_file_id");
        let status: i32 = row.get("status");
        let started_at: chrono::DateTime<chrono::Utc> = row.get("started_at");
        let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
        let error_message: Option<String> = row.get("error_message");

        // Convert status code to string
        let status_str = match status {
            0 => "pending",
            1 => "running",
            2 => "completed",
            3 => "failed",
            _ => "unknown"
        };

        serde_json::json!({
            "id": id,
            "project_id": project_id,
            "log_file_id": log_file_id,
            "status": status_str,
            "started_at": started_at,
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
        "SELECT id, project_id, log_file_id, status, started_at, completed_at,
         error_message, result
         FROM analyses
         WHERE id = ?"
    )
    .bind(&analysis_id)
    .fetch_optional(&db.pool)
    .await?;

    match row {
        Some(row) => {
            let id: String = row.get("id");
            let project_id: String = row.get("project_id");
            let log_file_id: Option<String> = row.get("log_file_id");
            let status: i32 = row.get("status");
            let started_at: chrono::DateTime<chrono::Utc> = row.get("started_at");
            let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
            let error_message: Option<String> = row.get("error_message");
            let result: Option<String> = row.get("result");

            // Convert status code to string
            let status_str = match status {
                0 => "pending",
                1 => "running",
                2 => "completed",
                3 => "failed",
                _ => "unknown"
            };

            // Parse the result JSON if available
            let result_value: Value = result
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(Value::Object(serde_json::Map::new()));

            Ok(serde_json::json!({
                "id": id,
                "project_id": project_id,
                "log_file_id": log_file_id,
                "status": status_str,
                "result": result_value,
                "started_at": started_at,
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
            let status: i32 = row.get("status");
            let error_message: Option<String> = row.get("error_message");

            // Convert status code to string and calculate progress
            let (status_str, progress) = match status {
                0 => ("pending", 0),
                1 => ("running", 50),
                2 => ("completed", 100),
                3 => ("failed", 100),
                _ => ("unknown", 0)
            };

            Ok(serde_json::json!({
                "id": id,
                "status": status_str,
                "progress": progress,
                "error_message": error_message
            }))
        }
        None => Err(anyhow::anyhow!("Analysis not found: {}", analysis_id))
    }
}