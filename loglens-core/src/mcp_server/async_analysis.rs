use std::path::PathBuf;
use tracing::{info, debug, error};

#[cfg(feature = "project-management")]
use {
    anyhow::Result,
    chrono::Utc,
    serde_json::Value,
    sqlx::SqlitePool,
    tokio::fs,
    crate::{
        LogLens,
        OutputFormat,
        project::{
            models::{AnalysisStatus, Pattern},
            queries::{store_analysis_results, update_analysis_status},
        },
    },
};

#[cfg(feature = "project-management")]
/// Spawn background analysis task
pub async fn spawn_analysis_task(
    pool: SqlitePool,
    analysis_id: String,
    log_file_path: PathBuf,
    provider: String,
    level: String,
    api_key: Option<String>,
) -> Result<()> {
    info!("Spawning analysis task for analysis_id: {}", analysis_id);
    
    tokio::spawn(async move {
        let result = async {
            debug!("Reading log file: {}", log_file_path.display());
            let log_content = fs::read_to_string(&log_file_path).await?;
            let log_lines: Vec<String> = log_content.lines().map(|s| s.to_string()).collect();
            
            info!("Running analysis on {} log lines with provider: {}", log_lines.len(), provider);
            let loglens = LogLens::new()?;
            let report = loglens.generate_full_report(
                log_lines,
                &level,
                &provider,
                api_key.as_deref(),
                "mcp_async",
                OutputFormat::Json,
            ).await?;
            
            // Parse the JSON report to extract metadata
            let report_json: Value = serde_json::from_str(&report)?;
            
            let summary = report_json.get("summary")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            
            let full_report = Some(report);
            
            let patterns = report_json.get("patterns")
                .and_then(|p| p.as_array())
                .map(|arr| {
                    arr.iter().filter_map(|p| {
                        Some(Pattern {
                            pattern: p.get("pattern")?.as_str()?.to_string(),
                            count: p.get("count")?.as_u64()? as usize,
                            examples: p.get("examples")
                                .and_then(|e| e.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                            severity: p.get("severity").and_then(|s| s.as_str()).unwrap_or("INFO").to_string(),
                            confidence: p.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.0),
                        })
                    }).collect()
                })
                .unwrap_or_default();
            
            let issues_found = report_json.get("total_issues")
                .and_then(|i| i.as_u64())
                .map(|i| i as i64);
            
            debug!("Analysis completed successfully for analysis_id: {}", analysis_id);
            Ok::<_, anyhow::Error>((summary, full_report, patterns, issues_found))
        }.await;
        
        match result {
            Ok((summary, full_report, patterns, issues_found)) => {
                info!("Storing analysis results for analysis_id: {}", analysis_id);
                
                // Store results
                if let Err(e) = store_analysis_results(
                    &pool, 
                    &analysis_id, 
                    summary, 
                    full_report, 
                    patterns, 
                    issues_found
                ).await {
                    error!("Failed to store analysis results: {}", e);
                    let _ = update_analysis_status(
                        &pool, 
                        &analysis_id, 
                        AnalysisStatus::Failed, 
                        Some(Utc::now())
                    ).await;
                    return;
                }
                
                // Mark completed
                if let Err(e) = update_analysis_status(
                    &pool, 
                    &analysis_id, 
                    AnalysisStatus::Completed, 
                    Some(Utc::now())
                ).await {
                    error!("Failed to update analysis status to completed: {}", e);
                } else {
                    info!("Analysis completed successfully for analysis_id: {}", analysis_id);
                }
            },
            Err(e) => {
                error!("Analysis failed for analysis_id {}: {}", analysis_id, e);
                if let Err(status_err) = update_analysis_status(
                    &pool, 
                    &analysis_id, 
                    AnalysisStatus::Failed, 
                    Some(Utc::now())
                ).await {
                    error!("Failed to update analysis status to failed: {}", status_err);
                }
            }
        }
    });
    
    Ok(())
}

// Non-feature-guarded placeholder for compilation
#[cfg(not(feature = "project-management"))]
pub async fn spawn_analysis_task(
    _pool: (),
    _analysis_id: String,
    _log_file_path: PathBuf,
    _provider: String,
    _level: String,
    _api_key: Option<String>,
) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("Project management feature not enabled"))
}

#[cfg(all(test, feature = "project-management"))]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use crate::project::queries::create_analysis;

    async fn setup_test_pool() -> (SqlitePool, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Use the database module's initialization which handles schema creation
        let pool = crate::project::database::initialize_database(&db_path)
            .await
            .unwrap();

        (pool, temp_dir)
    }

    #[tokio::test]
    async fn test_spawn_analysis_task_success() {
        let (pool, _temp) = setup_test_pool().await;

        // Create a test project first (required for foreign key constraint)
        let project_id = crate::project::queries::get_or_create_project(&pool, "/test/project")
            .await
            .unwrap();

        // Create a temporary log file
        let mut log_file = NamedTempFile::new().unwrap();
        writeln!(log_file, "[ERROR] Test error message 1").unwrap();
        writeln!(log_file, "[WARN] Test warning message").unwrap();
        writeln!(log_file, "[ERROR] Test error message 2").unwrap();
        let log_file_path = log_file.path().to_path_buf();

        // Create an analysis record
        let analysis_id = create_analysis(
            &pool,
            project_id,
            log_file_path.to_string_lossy().to_string(),
            "mock".to_string(), // Use mock provider to avoid actual API calls
            "ERROR".to_string(),
        ).await.unwrap();
        
        // Note: This test would need a mock provider to work properly
        // For now, we just test the task spawning
        let result = spawn_analysis_task(
            pool.clone(),
            analysis_id.clone(),
            log_file_path,
            "mock".to_string(),
            "ERROR".to_string(),
            None,
        ).await;
        
        // Should succeed in spawning the task
        assert!(result.is_ok());
        
        // Give the task a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}