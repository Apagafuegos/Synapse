use serde_json::Value;
use crate::Database;
use anyhow::Result;
use uuid::Uuid;
use sqlx::Row;

/// Trigger new analysis on existing file
pub async fn analyze_file(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid project_id parameter"))?;
    
    let file_id: String = serde_json::from_value(params["file_id"].clone())
        .map_err(|_| anyhow::anyhow!("Invalid file_id parameter"))?;
    
    let provider: String = params.get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("openrouter")
        .to_string();

    // Validate project exists
    let project_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects WHERE id = ?"
    )
    .bind(&project_id)
    .fetch_one(&db.pool)
    .await?;

    if project_count == 0 {
        return Err(anyhow::anyhow!("Project not found: {}", project_id));
    }

    // Validate file exists and belongs to project
    let file_row = sqlx::query(
        "SELECT id, project_id, file_path, original_name FROM files WHERE id = ? AND project_id = ?"
    )
    .bind(&file_id)
    .bind(&project_id)
    .fetch_optional(&db.pool)
    .await?;

    let row = match file_row {
        Some(row) => row,
        None => return Err(anyhow::anyhow!("File not found: {}", file_id))
    };

    let file_id: String = row.get("id");
    let project_id: String = row.get("project_id");
    let file_path: String = row.get("file_path");
    let _original_name: String = row.get("original_name");

    // Create analysis record
    let analysis_id = Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO analyses (id, project_id, file_id, status, provider) 
         VALUES (?, ?, ?, 'pending', ?)"
    )
    .bind(&analysis_id)
    .bind(&project_id)
    .bind(&file_id)
    .bind(&provider)
    .execute(&db.pool)
    .await?;

    // Spawn background analysis task
    let db_clone = db.clone();
    let file_path_clone = file_path.clone();
    let analysis_id_clone = analysis_id.clone();
    let provider_clone = provider.clone();

    tokio::spawn(async move {
        if let Err(e) = run_analysis(&db_clone, &analysis_id_clone, &file_path_clone, &provider_clone).await {
            // Log errors to file only, not stdout/stderr to avoid stdio contamination
            eprintln!("[BACKGROUND ERROR] Analysis task failed: {}", e);
        }
    });

    Ok(serde_json::json!({
        "analysis_id": analysis_id,
        "status": "pending"
    }))
}

/// Run analysis in background task
async fn run_analysis(
    db: &Database, 
    analysis_id: &str, 
    file_path: &str, 
    provider: &str
) -> Result<()> {
    // Update status to running
    sqlx::query(
        "UPDATE analyses SET status = 'running' WHERE id = ?"
    )
    .bind(analysis_id)
    .execute(&db.pool)
    .await?;

    // Read the log file
    let raw_lines = match loglens_core::input::read_log_file(file_path).await {
        Ok(lines) => lines,
        Err(e) => {
            // Update status to failed with error message
            sqlx::query(
                "UPDATE analyses SET status = 'failed', error_message = ?, completed_at = CURRENT_TIMESTAMP 
                 WHERE id = ?"
            )
            .bind(e.to_string())
            .bind(analysis_id)
            .execute(&db.pool)
            .await?;
            return Err(e);
        }
    };

    // Call loglens_core analysis function
    let result = loglens_core::analyze_lines(
        raw_lines, 
        "ERROR", // Default level for MCP analysis
        provider,
        None, // API key will be resolved from config
        None // Use default model
    ).await;

    match result {
        Ok(analysis) => {
            // Convert AnalysisResponse to database format
            let errors_json = serde_json::to_string(&analysis.errors_found.unwrap_or_default())?;
            let warnings_json = serde_json::to_string(&Vec::<String>::new())?; // TODO: Extract from analysis
            let recommendations_json = serde_json::to_string(&analysis.recommendations)?;
            let patterns_json = serde_json::to_string(&analysis.patterns.unwrap_or_default())?;
            let performance_json = serde_json::to_string(&analysis.performance)?;
            let metadata_json = serde_json::json!({
                "confidence": analysis.confidence,
                "root_cause": analysis.root_cause,
                "related_errors": analysis.related_errors,
                "unrelated_errors": analysis.unrelated_errors
            });
            let metadata_string = serde_json::to_string(&metadata_json)?;

            sqlx::query(
                "INSERT INTO analysis_results 
                 (analysis_id, summary, errors, warnings, recommendations, patterns, 
                  performance_metrics, metadata)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(analysis_id)
            .bind(&analysis.sequence_of_events)
            .bind(errors_json)
            .bind(warnings_json)
            .bind(recommendations_json)
            .bind(patterns_json)
            .bind(performance_json)
            .bind(metadata_string)
            .execute(&db.pool)
            .await?;

            // Update status to completed
            sqlx::query(
                "UPDATE analyses SET status = 'completed', completed_at = CURRENT_TIMESTAMP
                 WHERE id = ?"
            )
            .bind(analysis_id)
            .execute(&db.pool)
            .await?;

            // Success - no logging to avoid stdio contamination
        }
        Err(e) => {
            // Update status to failed with error message
            sqlx::query(
                "UPDATE analyses SET status = 'failed', error_message = ?, completed_at = CURRENT_TIMESTAMP
                 WHERE id = ?"
            )
            .bind(e.to_string())
            .bind(analysis_id)
            .execute(&db.pool)
            .await?;

            // Error info is already in database - no logging to avoid stdio contamination
        }
    }

    Ok(())
}