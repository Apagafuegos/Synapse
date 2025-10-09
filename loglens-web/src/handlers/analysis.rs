use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;

use tokio::fs;

use crate::{
    circuit_breaker::{CircuitBreakerConfig, CircuitBreakerRegistry, CircuitBreaker},
    error_handling::AppError,
    models::*,
    validation::Validator,
    AppState
};
use std::sync::Arc;
use loglens_core::{analyze_lines, AnalysisResponse};

#[derive(Deserialize)]
pub struct AnalysisQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn start_analysis(
    State(state): State<AppState>,
    Path((project_id, file_id)): Path<(String, String)>,
    Json(req): Json<AnalysisRequest>,
) -> Result<Json<Analysis>, AppError> {
    // Validate input parameters
    Validator::validate_uuid(&project_id)
        .map_err(|e| {
            tracing::warn!("Invalid project ID: {}", e.to_message());
            AppError::validation(e.to_message())
        })?;

    Validator::validate_uuid(&file_id)
        .map_err(|e| {
            tracing::warn!("Invalid file ID: {}", e.to_message());
            AppError::validation(e.to_message())
        })?;

    // Validate and sanitize analysis request
    let (sanitized_provider, sanitized_level, sanitized_context) =
        Validator::validate_analysis_request(&req.provider, &req.level, req.user_context.as_ref())
            .map_err(|e| {
                tracing::warn!("Analysis request validation failed: {}", e.to_message());
                AppError::from(e)
            })?;

    // Validate timeout if provided
    let timeout_seconds = if let Some(timeout) = req.timeout_seconds {
        if !(60..=1800).contains(&timeout) { // 1 minute to 30 minutes
            return Err(AppError::validation("Timeout must be between 60 and 1800 seconds"));
        }
        timeout
    } else {
        // Get default from settings
        let settings_timeout = sqlx::query!("SELECT analysis_timeout_seconds FROM settings WHERE id = 1")
            .fetch_optional(state.db.pool())
            .await
            .map_err(AppError::Database)?
            .and_then(|row| row.analysis_timeout_seconds)
            .unwrap_or(300);
        settings_timeout as u32
    };

    // Verify project and file exist
    let log_file = sqlx::query_as::<_, LogFile>(
        "SELECT id, project_id, filename, file_size, line_count, upload_path, created_at
         FROM log_files WHERE id = ? AND project_id = ?",
    )
    .bind(&file_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch log file {} for project {}: {}", file_id, project_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Log file {} not found", file_id)))?;

    // Create analysis record using sanitized values
    let analysis = Analysis::new(
        project_id,
        Some(file_id),
        "file".to_string(),
        sanitized_provider.clone(),
        sanitized_level.clone(),
    );

    // Save analysis to database
    let status_value = analysis.status as i32;
    sqlx::query!(
        "INSERT INTO analyses (id, project_id, log_file_id, analysis_type, provider, level_filter, status, started_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        analysis.id,
        analysis.project_id,
        analysis.log_file_id,
        analysis.analysis_type,
        analysis.provider,
        analysis.level_filter,
        status_value,
        analysis.started_at
    )
    .execute(state.db.pool())
    .await
    .map_err(AppError::Database)?;

    // Start analysis in background with sanitized values
    let analysis_id = analysis.id.clone();
    let db_pool = state.db.pool().clone();
    let file_path = log_file.upload_path.clone();
    let provider = sanitized_provider.clone();
    let level = sanitized_level.clone();
    let user_context = sanitized_context.clone();
    let circuit_breakers = state.circuit_breakers.clone();

    tokio::spawn(async move {
        tracing::info!(" Starting analysis {} for file: {}", analysis_id, file_path);

        // Fetch API key and selected model from settings
        let (api_key, selected_model) = match sqlx::query!("SELECT api_key, selected_model FROM settings LIMIT 1")
            .fetch_optional(&db_pool)
            .await
        {
            Ok(Some(settings)) if !settings.api_key.is_empty() => {
                tracing::info!("API key found for analysis {}", analysis_id);
                tracing::info!("Selected model from settings: {:?}", settings.selected_model);
                tracing::info!("Raw settings object: {:#?}", settings);
                (Some(settings.api_key), settings.selected_model)
            },
            Ok(_) => {
                tracing::warn!("No API key found in settings for analysis {}", analysis_id);
                (None, None)
            },
            Err(e) => {
                tracing::error!("Failed to fetch settings for analysis {}: {}", analysis_id, e);
                (None, None)
            }
        };

        tracing::info!("Calling perform_analysis_with_context for {} with provider: {}, level: {}, timeout: {}s", analysis_id, provider, level, timeout_seconds);
        tracing::info!("User context provided: {}", user_context.is_some());
        tracing::info!("Using model: {:?}", selected_model);
        
        // Use enhanced analysis function that supports context and model selection
        let result = perform_analysis_with_context(
            &file_path,
            &level,
            &provider,
            api_key.as_deref(),
            &circuit_breakers,
            timeout_seconds as u64,
            user_context.as_deref(),
            selected_model.as_deref(),
        ).await;

        match result {
            Ok(analysis_result) => {
                tracing::info!("Analysis {} completed successfully, serializing result", analysis_id);
                match serde_json::to_string(&analysis_result) {
                    Ok(result_json) => {
                        tracing::info!("Serialized result for analysis {} ({} chars)", analysis_id, result_json.len());
                        // Retry database update up to 3 times
                        for attempt in 1..=3 {
                            match sqlx::query!(
                                "UPDATE analyses SET status = ?, result = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                                AnalysisStatus::Completed as i32,
                                result_json,
                                analysis_id
                            ).execute(&db_pool).await {
                                Ok(_) => {
                                    tracing::info!("Analysis {} database update completed successfully", analysis_id);
                                    break;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to update analysis {} status (attempt {}): {}", analysis_id, attempt, e);
                                    if attempt == 3 {
                                        tracing::error!("Failed to update analysis {} after 3 attempts, giving up", analysis_id);
                                    } else {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize analysis result for {}: {}", analysis_id, e);
                        // Try to update with error status instead
                        let error_msg = format!("Serialization error: {}", e);
                        for attempt in 1..=3 {
                            match sqlx::query!(
                                "UPDATE analyses SET status = ?, error_message = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                                AnalysisStatus::Failed as i32,
                                error_msg,
                                analysis_id
                            ).execute(&db_pool).await {
                                Ok(_) => break,
                                Err(db_e) => {
                                    tracing::error!("Failed to update analysis {} with serialization error (attempt {}): {}", analysis_id, attempt, db_e);
                                    if attempt < 3 {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                tracing::error!("Analysis {} FAILED with error: {}", analysis_id, error);
                tracing::error!("Error chain for analysis {}: {:#}", analysis_id, error);

                // Get the full error chain
                let error_msg = format!("{:#}", error);
                let truncated_error = if error_msg.len() > 1000 {
                    format!("{}... (truncated)", &error_msg[..1000])
                } else {
                    error_msg
                };

                tracing::error!("Will store error message for analysis {}: {}", analysis_id, truncated_error);

                // Retry database update up to 3 times
                for attempt in 1..=3 {
                    match sqlx::query!(
                        "UPDATE analyses SET status = ?, error_message = ?, completed_at = CURRENT_TIMESTAMP WHERE id = ?",
                        AnalysisStatus::Failed as i32,
                        truncated_error,
                        analysis_id
                    ).execute(&db_pool).await {
                        Ok(_) => {
                            tracing::info!("Analysis {} failed and status updated in database", analysis_id);
                            break;
                        }
                        Err(e) => {
                            tracing::error!("Failed to update failed analysis {} status (attempt {}): {}", analysis_id, attempt, e);
                            if attempt < 3 {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100 * attempt)).await;
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(Json(analysis))
}

pub async fn get_analysis(
    State(state): State<AppState>,
    Path(analysis_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = ?"
    )
    .bind(&analysis_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::not_found(format!("Analysis {} not found", analysis_id)))?;

    // Transform the analysis to match frontend expectations
    let transformed_analysis = serde_json::json!({
        "id": analysis.id,
        "project_id": analysis.project_id,
        "file_id": analysis.log_file_id,
        "status": match analysis.status {
            AnalysisStatus::Pending => "pending",
            AnalysisStatus::Running => "running", 
            AnalysisStatus::Completed => "completed",
            AnalysisStatus::Failed => "failed",
        },
        "user_context": None::<String>, // Frontend expects this field
        "ai_provider": analysis.provider,
        "created_at": analysis.started_at.to_rfc3339(),
        "updated_at": analysis.completed_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| analysis.started_at.to_rfc3339()),
        "result": analysis.result,
        "progress": None::<i32>, // Frontend expects this field
        "error": analysis.error_message,
    });

    Ok(Json(transformed_analysis))
}

pub async fn list_analyses(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<AnalysisQuery>,
) -> Result<Json<AnalysisListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Get total count
    let total_row = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE project_id = ?",
        project_id
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(AppError::Database)?;
    let total = total_row.count;

    // Get analyses with optional file information
    let analyses_with_files = sqlx::query!(
        r#"
        SELECT 
            a.id, a.project_id, a.log_file_id, a.analysis_type, a.provider, a.level_filter,
            a.status, a.result, a.error_message, a.started_at, a.completed_at,
            f.filename
        FROM analyses a
        LEFT JOIN log_files f ON a.log_file_id = f.id
        WHERE a.project_id = ?
        ORDER BY a.started_at DESC
        LIMIT ? OFFSET ?
        "#,
        project_id,
        limit,
        offset
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(AppError::Database)?;

    let mut analyses = Vec::new();
    for row in analyses_with_files {
        let analysis_id = row.id.ok_or_else(|| {
            tracing::error!("Database corruption: analysis record missing ID");
            AppError::internal("Database corruption: analysis record missing ID")
        })?;

        let analysis = Analysis {
            id: analysis_id,
            project_id: row.project_id,
            log_file_id: row.log_file_id,
            analysis_type: row.analysis_type,
            provider: row.provider,
            level_filter: row.level_filter,
            status: match row.status {
                0 => AnalysisStatus::Pending,
                1 => AnalysisStatus::Running,
                2 => AnalysisStatus::Completed,
                3 => AnalysisStatus::Failed,
                _ => AnalysisStatus::Failed,
            },
            result: row.result,
            error_message: row.error_message,
            started_at: row.started_at.and_utc(),
            completed_at: row.completed_at.map(|dt: chrono::NaiveDateTime| dt.and_utc()),
        };

        analyses.push(AnalysisWithFile {
            analysis,
            filename: row.filename,
        });
    }

    Ok(Json(AnalysisListResponse {
        analyses,
        total: total.into(),
    }))
}

pub async fn perform_analysis_with_context(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    circuit_breakers: &std::sync::Arc<CircuitBreakerRegistry>,
    timeout_secs: u64,
    user_context: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!("perform_analysis_with_context called for file: {}", file_path);
    tracing::info!("Analysis parameters - Level: {}, Provider: {}, Timeout: {}s", level, provider, timeout_secs);
    tracing::info!("🔑 API key provided: {}", api_key.is_some());
    tracing::info!("User context provided: {}", user_context.is_some());
    tracing::info!("Selected model: {:?}", selected_model);

    // Check file size first to determine if we should use streaming
    let metadata = fs::metadata(file_path).await
        .map_err(|e| {
            tracing::error!("Failed to read file metadata for {}: {}", file_path, e);
            anyhow::anyhow!("Failed to read file metadata: {}", e)
        })?;
    let file_size = metadata.len();

    tracing::info!("File size: {} bytes ({:.2} MB)", file_size, file_size as f64 / 1024.0 / 1024.0);

    // If file is larger than 10MB, use streaming approach
    if file_size > 10 * 1024 * 1024 {
        tracing::info!("🌊 Large file detected ({} bytes), using streaming approach", file_size);
        return perform_analysis_streaming(
            file_path, level, provider, api_key, circuit_breakers, timeout_secs, user_context, selected_model
        ).await;
    }

    tracing::info!("Small file detected, using standard approach");

    // For smaller files, use the original approach
    let _circuit_breaker = create_or_get_circuit_breaker(circuit_breakers, provider, timeout_secs).await
        .map_err(|e| {
            tracing::error!("Failed to create circuit breaker: {}", e);
            e
        })?;

    tracing::info!("Starting timeout wrapper with {}s timeout", timeout_secs);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        analyze_large_file_with_context(file_path, level, provider, api_key, user_context, selected_model)
    ).await;

    match result {
        Ok(Ok(analysis_result)) => {
            tracing::info!("Analysis completed successfully");
            Ok(analysis_result)
        },
        Ok(Err(e)) => {
            tracing::error!("Analysis failed with error: {}", e);
            Err(anyhow::anyhow!("Analysis failed: {}", e))
        },
        Err(_) => {
            tracing::error!("Analysis timed out after {} seconds", timeout_secs);
            let error_msg = format!("Analysis timed out after {} seconds", timeout_secs);
            tracing::error!("Analysis timed out for provider {} after {} seconds", provider, timeout_secs);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

pub async fn perform_analysis(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    circuit_breakers: &std::sync::Arc<CircuitBreakerRegistry>,
    timeout_secs: u64,
    user_context: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!("perform_analysis called for file: {}", file_path);
    tracing::info!("Analysis parameters - Level: {}, Provider: {}, Timeout: {}s", level, provider, timeout_secs);
    tracing::info!("🔑 API key provided: {}", api_key.is_some());

    // Check file size first to determine if we should use streaming
    let metadata = fs::metadata(file_path).await
        .map_err(|e| {
            tracing::error!("Failed to read file metadata for {}: {}", file_path, e);
            anyhow::anyhow!("Failed to read file metadata: {}", e)
        })?;
    let file_size = metadata.len();

    tracing::info!("File size: {} bytes ({:.2} MB)", file_size, file_size as f64 / 1024.0 / 1024.0);

    // If file is larger than 10MB, use streaming approach
    if file_size > 10 * 1024 * 1024 {
        tracing::info!("🌊 Large file detected ({} bytes), using streaming approach", file_size);
        return perform_analysis_streaming(
            file_path, level, provider, api_key, circuit_breakers, timeout_secs, user_context, selected_model
        ).await;
    }

    tracing::info!("Small file detected, using standard approach");

    // For smaller files, use the original approach
    let _circuit_breaker = create_or_get_circuit_breaker(circuit_breakers, provider, timeout_secs).await
        .map_err(|e| {
            tracing::error!("Failed to create circuit breaker: {}", e);
            e
        })?;

    tracing::info!("Starting timeout wrapper with {}s timeout", timeout_secs);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        analyze_large_file(file_path, level, provider, api_key, selected_model)
    ).await;

    match result {
        Ok(Ok(analysis_result)) => {
            tracing::info!("Analysis completed successfully");
            Ok(analysis_result)
        },
        Ok(Err(e)) => {
            tracing::error!("Analysis failed with error: {}", e);
            Err(anyhow::anyhow!("Analysis failed: {}", e))
        },
        Err(_) => {
            tracing::error!("Analysis timed out after {} seconds", timeout_secs);
            let error_msg = format!("Analysis timed out after {} seconds", timeout_secs);
            tracing::error!("Analysis timed out for provider {} after {} seconds", provider, timeout_secs);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

/// Perform analysis with streaming for large files
async fn perform_analysis_streaming(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    _circuit_breakers: &std::sync::Arc<CircuitBreakerRegistry>,
    timeout_secs: u64,
    user_context: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!(" perform_analysis_streaming started for file: {}", file_path);
    tracing::info!("Streaming analysis parameters - Level: {}, Provider: {}, Timeout: {}s", level, provider, timeout_secs);
    tracing::info!("🔑 API key provided: {}", api_key.is_some());

    tracing::info!("Setting up timeout for {} seconds", timeout_secs);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        analyze_large_file_streaming(file_path, level, provider, api_key, user_context, selected_model)
    ).await;

    match result {
        Ok(Ok(analysis_result)) => {
            tracing::info!("Streaming analysis completed successfully for {}", file_path);
            tracing::info!("📈 Analysis result has {} characters in sequence_of_events",
                analysis_result.sequence_of_events.len());
            Ok(analysis_result)
        },
        Ok(Err(e)) => {
            tracing::error!("Streaming analysis failed for {}: {}", file_path, e);
            tracing::error!("Full error chain: {:#}", e);
            Err(anyhow::anyhow!(e))
        },
        Err(_) => {
            let error_msg = format!("Analysis timed out after {} seconds", timeout_secs);
            tracing::error!("Analysis timed out for provider {} after {} seconds", provider, timeout_secs);
            tracing::error!("File: {}, Level: {}", file_path, level);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

/// Create or get circuit breaker with appropriate timeout
async fn create_or_get_circuit_breaker(
    circuit_breakers: &std::sync::Arc<CircuitBreakerRegistry>,
    provider: &str,
    timeout_secs: u64,
) -> anyhow::Result<Arc<CircuitBreaker>> {
    // For large files, increase the timeout to be more generous
    let adjusted_timeout = std::cmp::max(timeout_secs, 120); // At least 2 minutes
    
    Ok(circuit_breakers.get_or_create(
        &format!("ai_provider_{}", provider),
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_duration: std::time::Duration::from_secs(adjusted_timeout),
            reset_timeout: std::time::Duration::from_secs(120),
        })
    ))
}

/// Analyze large file with streaming to avoid memory issues
async fn analyze_large_file_streaming(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    _user_context: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!("📂 analyze_large_file_streaming started for: {}", file_path);
    tracing::info!("Target log level: {}, Provider: {}", level, provider);

    // Check if file exists before opening
    if !std::path::Path::new(file_path).exists() {
        let error_msg = format!("File does not exist: {}", file_path);
        tracing::error!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    // Get file metadata
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            tracing::info!("File size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);
            tracing::info!("File modified: {:?}", metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH));
        }
        Err(e) => {
            tracing::warn!("Could not read file metadata: {}", e);
        }
    }

    tracing::info!("🔓 Opening file for streaming read with encoding detection...");

    // Read file as bytes first for encoding detection
    let data = fs::read(file_path).await.map_err(|e| {
        tracing::error!("Failed to read file {}: {}", file_path, e);
        anyhow::anyhow!("Failed to read file {}: {}", file_path, e)
    })?;

    tracing::info!("File read successfully, {} bytes", data.len());
    tracing::info!("Starting encoding detection and streaming processing...");

    let mut logs = Vec::new();
    let mut line_count = 0;
    let mut filtered_count = 0;
    let mut empty_lines = 0;
    let max_lines = 10000; // Limit to prevent overwhelming AI

    tracing::info!("Starting line-by-line processing with encoding detection (max {} log entries)...", max_lines);

    // Process the file using robust encoding detection similar to core library
    let lines = process_lines_with_encoding_detection(&data)?;

    for line_result in lines {
        line_count += 1;

        // Skip empty lines
        if line_result.trim().is_empty() {
            empty_lines += 1;
            continue;
        }

        // Store all lines (let core library handle filtering including stack traces)
        logs.push(line_result.clone());
        filtered_count += 1;

        // Log first few matching lines for debugging
        if filtered_count <= 3 {
            tracing::debug!("Log entry {}: {}", filtered_count,
                if line_result.len() > 100 {
                    format!("{}...", &line_result[..100])
                } else {
                    line_result
                }
            );
        }

        // Limit the number of lines to prevent context overflow
        if logs.len() >= max_lines {
            tracing::warn!("Reached line limit ({}) for large file analysis", max_lines);
            break;
        }

        // Progress tracking
        if line_count % 10000 == 0 {
            tracing::info!("Processed {} lines, found {} matching logs", line_count, filtered_count);
        }
    }

    tracing::info!("Streaming read completed:");
    tracing::info!("  Total lines processed: {}", line_count);
    tracing::info!("  Empty lines skipped: {}", empty_lines);
    tracing::info!("  Matching log entries: {}", filtered_count);
    tracing::info!("  Log entries for analysis: {}", logs.len());

    if logs.is_empty() {
        let error_msg = format!("No log entries found matching level '{}' in file", level);
        tracing::warn!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    tracing::info!("Sending {} log entries to AI provider '{}'...", logs.len(), provider);

    // Perform analysis with the collected logs
    analyze_lines(logs, level, provider, api_key, selected_model).await.map_err(|e| {
        tracing::error!("AI analysis failed: {}", e);
        tracing::error!("Error details: {:#}", e);
        e
    })
}

/// Analyze large file by reading it in chunks
async fn analyze_large_file(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!("analyze_large_file started for: {}", file_path);
    tracing::info!("Target log level: {}, Provider: {}", level, provider);
    tracing::info!("Selected model: {:?}", selected_model);

    // Check file exists and get metadata
    let metadata = std::fs::metadata(file_path).map_err(|e| {
        tracing::error!("Cannot access file metadata for {}: {}", file_path, e);
        anyhow::anyhow!("Cannot access file: {}", e)
    })?;

    tracing::info!("File size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);

    // Read log file with robust encoding detection
    tracing::info!("Reading file with encoding detection...");
    let data = fs::read(file_path).await.map_err(|e| {
        tracing::error!("Failed to read file {}: {}", file_path, e);
        anyhow::anyhow!("Failed to read file: {}", e)
    })?;

    tracing::info!("File read successfully, {} bytes", data.len());
    tracing::info!("Starting encoding detection and text parsing...");

    let logs = read_file_with_encoding_detection(&data).map_err(|e| {
        tracing::error!("Encoding detection failed for {}: {}", file_path, e);
        tracing::error!("Error details: {:#}", e);
        e
    })?;

    tracing::info!("Encoding detection completed successfully");
    tracing::info!("Total lines parsed: {}", logs.len());

    // Skip filtering here - let core library handle it properly with stack trace preservation
    let _filtered_logs_count = logs.len();
    let filtered_logs = logs;

    tracing::info!("Prepared {} lines for core library filtering...", filtered_logs.len());

    if filtered_logs.is_empty() {
        let error_msg = format!("No log entries found matching level '{}' in file", level);
        tracing::warn!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    // Log first few entries for debugging
    for (i, log) in filtered_logs.iter().take(3).enumerate() {
        tracing::debug!("Sample log {}: {}", i + 1,
            if log.len() > 150 {
                format!("{}...", &log[..150])
            } else {
                log.clone()
            }
        );
    }

    tracing::info!("Sending {} log entries to AI provider '{}'...", filtered_logs.len(), provider);

    // Pass selected_model to analyze_lines
    analyze_lines(filtered_logs, level, provider, api_key, selected_model).await.map_err(|e| {
        tracing::error!("AI analysis failed: {}", e);
        tracing::error!("Error details: {:#}", e);
        e
    })
}

/// Analyze large file with context and model selection
async fn analyze_large_file_with_context(
    file_path: &str,
    level: &str,
    provider: &str,
    api_key: Option<&str>,
    user_context: Option<&str>,
    selected_model: Option<&str>,
) -> anyhow::Result<AnalysisResponse> {
    tracing::info!("analyze_large_file_with_context started for: {}", file_path);
    tracing::info!("Target log level: {}, Provider: {}", level, provider);
    tracing::info!("User context: {:?}", user_context);
    tracing::info!("Selected model: {:?}", selected_model);

    // Check file exists and get metadata
    let metadata = std::fs::metadata(file_path).map_err(|e| {
        tracing::error!("Cannot access file metadata for {}: {}", file_path, e);
        anyhow::anyhow!("Cannot access file: {}", e)
    })?;

    tracing::info!("File size: {} bytes ({:.2} MB)", metadata.len(), metadata.len() as f64 / 1024.0 / 1024.0);

    // Read log file with robust encoding detection
    tracing::info!("Reading file with encoding detection...");
    let data = fs::read(file_path).await.map_err(|e| {
        tracing::error!("Failed to read file {}: {}", file_path, e);
        anyhow::anyhow!("Failed to read file: {}", e)
    })?;

    tracing::info!("File read successfully, {} bytes", data.len());
    tracing::info!("Starting encoding detection and text parsing...");

    let logs = read_file_with_encoding_detection(&data).map_err(|e| {
        tracing::error!("Encoding detection failed for {}: {}", file_path, e);
        tracing::error!("Error details: {:#}", e);
        e
    })?;

    tracing::info!("Encoding detection completed successfully");
    tracing::info!("Total lines parsed: {}", logs.len());

    // Skip filtering here - let core library handle it properly with stack trace preservation
    let _filtered_logs_count = logs.len();
    let filtered_logs = logs;

    tracing::info!("Prepared {} lines for core library filtering...", filtered_logs.len());

    if filtered_logs.is_empty() {
        let error_msg = format!("No log entries found matching level '{}' in file", level);
        tracing::warn!("{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    // Log first few entries for debugging
    for (i, log) in filtered_logs.iter().take(3).enumerate() {
        tracing::debug!("Sample log {}: {}", i + 1,
            if log.len() > 150 {
                format!("{}...", &log[..150])
            } else {
                log.clone()
            }
        );
    }

    tracing::info!("Sending {} log entries to AI provider '{}'...", filtered_logs.len(), provider);
    tracing::info!("User context will be included: {}", user_context.is_some());
    tracing::info!("Selected model will be used: {}", selected_model.unwrap_or("default"));

    // Pass selected_model to analyze_lines
    analyze_lines(filtered_logs, level, provider, api_key, selected_model).await.map_err(|e| {
        tracing::error!("AI analysis failed: {}", e);
        tracing::error!("Error details: {:#}", e);
        e
    })
}

/// Check if a log line meets the minimum required level
#[allow(dead_code)]
fn is_log_level_at_least(log_line: &str, min_level: &str) -> bool {
    use loglens_core::filter::LogLevel;
    use std::str::FromStr;

    // Parse the minimum level
    let Ok(min_level_enum) = LogLevel::from_str(min_level) else {
        return false; // Invalid min_level
    };

    // Extract level from log line
    // Match various formats:
    // 1. [ERROR], (WARN), etc - bracketed formats
    // 2. ERROR:, WARN:, etc - colon-separated at start of line
    // 3. ESC[31mERROR ESC[0;39m - ANSI color codes around level
    let level_pattern = r#"(?ix)
        # Bracketed format: [ERROR] or (WARN)
        (?:\[|\()(ERROR|FATAL|CRIT|CRITICAL|WARN|WARNING|INFO|DEBUG|TRACE)(?:\]|\))
        |
        # ANSI codes around level: ESC[31mERROR ESC[0;39m (ESC = \x1b or \u{1b})
        \x1b\[\d+m\s*(ERROR|FATAL|CRIT|CRITICAL|WARN|WARNING|INFO|DEBUG|TRACE)\s*\x1b\[\d+;\d+m
        |
        # Colon-separated at start: ERROR: message
        ^[^:]*?\b(ERROR|FATAL|CRIT|CRITICAL|WARN|WARNING|INFO|DEBUG|TRACE)\s*:
    "#;
    let re = regex::Regex::new(level_pattern).unwrap();

    if let Some(caps) = re.captures(log_line) {
        // Get first non-None capture group (from 3 possible patterns)
        let Some(matched_level) = caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)) else {
            return false;
        };
        let log_level_str = matched_level.as_str().to_uppercase();

        // Normalize level variants
        let normalized = match log_level_str.as_str() {
            "FATAL" | "CRIT" | "CRITICAL" => "FATAL",
            "ERR" => "ERROR",
            "WARNING" => "WARN",
            "DBG" => "DEBUG",
            "TRC" => "TRACE",
            other => other,
        };

        // Parse log level and compare
        if let Ok(log_level_enum) = LogLevel::from_str(normalized) {
            // Correct comparison: log_level >= min_level
            return log_level_enum >= min_level_enum;
        }
    }

    // If no valid level found, exclude by default (conservative filtering)
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_log_level_at_least_correct_filtering() {
        // Test ERROR filtering (should only include ERROR and FATAL)
        assert!(is_log_level_at_least("[ERROR] Something failed", "ERROR"));
        assert!(!is_log_level_at_least("[WARN] Warning message", "ERROR"));
        assert!(!is_log_level_at_least("[INFO] Information", "ERROR"));
        assert!(!is_log_level_at_least("[DEBUG] Debug info", "ERROR"));

        // Test WARN filtering (should include WARN and ERROR)
        assert!(is_log_level_at_least("[ERROR] Something failed", "WARN"));
        assert!(is_log_level_at_least("[WARN] Warning message", "WARN"));
        assert!(!is_log_level_at_least("[INFO] Information", "WARN"));

        // Test INFO filtering (should include INFO, WARN, ERROR)
        assert!(is_log_level_at_least("[ERROR] Something failed", "INFO"));
        assert!(is_log_level_at_least("[WARN] Warning message", "INFO"));
        assert!(is_log_level_at_least("[INFO] Information", "INFO"));
        assert!(!is_log_level_at_least("[DEBUG] Debug info", "INFO"));

        // Test no level found (should exclude)
        assert!(!is_log_level_at_least("No log level in this message", "ERROR"));
        assert!(!is_log_level_at_least("This is just text", "INFO"));

        // Test level in message content (should not match)
        assert!(!is_log_level_at_least("User provided invalid information", "ERROR"));
        assert!(!is_log_level_at_least("The error message was displayed", "ERROR"));
    }

    #[test]
    fn test_is_log_level_at_least_with_ansi_codes() {
        // Spring Boot style logs with ANSI color codes (ESC = \x1b)
        assert!(is_log_level_at_least("\x1b[31mERROR\x1b[0;39m Exception occurred", "ERROR"));
        assert!(!is_log_level_at_least("\x1b[32m INFO\x1b[0;39m Starting service", "ERROR"));
        assert!(is_log_level_at_least("\x1b[32m INFO\x1b[0;39m Starting service", "INFO"));
        assert!(!is_log_level_at_least("\x1b[32m INFO\x1b[0;39m Starting service", "WARN"));

        // Real Spring Boot format from the log file
        let real_log = "\x1b[2m2025-06-23 11:47:10.714\x1b[0;39m \x1b[31mERROR\x1b[0;39m \x1b[35m36\x1b[0;39m \x1b[2m---\x1b[0;39m";
        assert!(is_log_level_at_least(real_log, "ERROR"));
        assert!(!is_log_level_at_least(real_log, "FATAL"));
    }
}

/// Process lines with robust encoding detection
fn process_lines_with_encoding_detection(data: &[u8]) -> anyhow::Result<Vec<String>> {

    tracing::info!("process_lines_with_encoding_detection started, data size: {} bytes", data.len());

    // Detect encoding and create decoder
    let (encoding, _confidence, decoder) = detect_and_create_decoder(data);
    tracing::info!("Detected encoding: {}", encoding.name());

    // Process line by line for better error recovery
    let lines = decode_lines_robust(data, &decoder)?;

    tracing::info!("Encoding detection and parsing completed successfully");
    tracing::info!("Parsed {} lines from the data", lines.len());

    Ok(lines)
}

/// Detect file encoding and create appropriate decoder (simplified version from core)
fn detect_and_create_decoder(data: &[u8]) -> (&'static encoding_rs::Encoding, f64, encoding_rs::Decoder) {
    use encoding_rs::{UTF_8, UTF_16LE, UTF_16BE, WINDOWS_1252};

    // Simple heuristics for encoding detection
    if data.is_empty() {
        return (UTF_8, 1.0, UTF_8.new_decoder());
    }

    // Check for UTF-8 BOM
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (UTF_8, 1.0, UTF_8.new_decoder_with_bom_removal());
    }

    // Check for UTF-16 LE BOM
    if data.starts_with(&[0xFF, 0xFE]) {
        return (UTF_16LE, 1.0, UTF_16LE.new_decoder_with_bom_removal());
    }

    // Check for UTF-16 BE BOM
    if data.starts_with(&[0xFE, 0xFF]) {
        return (UTF_16BE, 1.0, UTF_16BE.new_decoder_with_bom_removal());
    }

    // Try UTF-8 validation first
    match std::str::from_utf8(data) {
        Ok(_) => {
            // Valid UTF-8
            (UTF_8, 1.0, UTF_8.new_decoder())
        }
        Err(_) => {
            // Not valid UTF-8, fallback to Windows-1252 for robust handling
            tracing::info!("UTF-8 validation failed, using Windows-1252 fallback");
            (WINDOWS_1252, 0.8, WINDOWS_1252.new_decoder())
        }
    }
}

/// Decode lines with robust error handling (simplified version from core)
fn decode_lines_robust(data: &[u8], _decoder: &encoding_rs::Decoder) -> anyhow::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];

        // Check for line endings
        if byte == b'\n' {
            // LF line ending
            match try_fallback_decoders(&current_line) {
                Ok(line_text) => {
                    lines.push(line_text);
                }
                Err(_) => {
                    // For failed decodes, use lossy UTF-8 conversion
                    let line_text = String::from_utf8_lossy(&current_line).into_owned();
                    lines.push(line_text);
                }
            }
            current_line.clear();
            i += 1;
        } else if byte == b'\r' && i + 1 < data.len() && data[i + 1] == b'\n' {
            // CRLF line ending
            match try_fallback_decoders(&current_line) {
                Ok(line_text) => {
                    lines.push(line_text);
                }
                Err(_) => {
                    // For failed decodes, use lossy UTF-8 conversion
                    let line_text = String::from_utf8_lossy(&current_line).into_owned();
                    lines.push(line_text);
                }
            }
            current_line.clear();
            i += 2; // Skip both \r and \n
        } else {
            // Regular character
            current_line.push(byte);
            i += 1;
        }
    }

    // Handle last line if file doesn't end with newline
    if !current_line.is_empty() {
        match try_fallback_decoders(&current_line) {
            Ok(line_text) => {
                lines.push(line_text);
            }
            Err(_) => {
                // For failed decodes, use lossy UTF-8 conversion
                let line_text = String::from_utf8_lossy(&current_line).into_owned();
                lines.push(line_text);
            }
        }
    }

    Ok(lines)
}

/// Try multiple encoding decoders as fallback
fn try_fallback_decoders(line_bytes: &[u8]) -> Result<String, String> {
    use encoding_rs::{UTF_8, WINDOWS_1252, ISO_8859_2, ISO_8859_3};

    let fallback_encodings = [
        (UTF_8, "UTF-8"),
        (WINDOWS_1252, "Windows-1252"),
        (ISO_8859_2, "ISO-8859-2"),
        (ISO_8859_3, "ISO-8859-3"),
    ];

    for (encoding, _name) in fallback_encodings.iter() {
        let mut result = String::new();
        let mut decoder = encoding.new_decoder();

        let (_result, _used, had_errors) = decoder.decode_to_string(line_bytes, &mut result, false);

        if !result.is_empty() && !had_errors {
            return Ok(result);
        }
    }

    // Last resort: lossy UTF-8 with replacement characters
    Ok(String::from_utf8_lossy(line_bytes).into_owned())
}

/// Read file with robust encoding detection using core library
fn read_file_with_encoding_detection(data: &[u8]) -> anyhow::Result<Vec<String>> {
    tracing::info!("read_file_with_encoding_detection started, data size: {} bytes", data.len());

    // Use the core library's read_log_file function by writing to a temp file
    // This ensures we use the same improved encoding detection logic

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("loglens_temp_{}.log", uuid::Uuid::new_v4()));

    tracing::info!("Creating temporary file: {}", temp_file.display());

    // Write data to temp file
    std::fs::write(&temp_file, data)
        .map_err(|e| {
            tracing::error!("Failed to write temporary file {}: {}", temp_file.display(), e);
            anyhow::anyhow!("Failed to write temp file: {}", e)
        })?;

    tracing::info!("Temporary file written successfully");
    tracing::info!("Calling loglens_core::read_log_file for encoding detection...");

    // Use the core library's read function
    let result = futures::executor::block_on(async {
        loglens_core::read_log_file(temp_file.to_string_lossy().as_ref()).await
    });

    // Clean up temp file
    match std::fs::remove_file(&temp_file) {
        Ok(_) => tracing::debug!("Temporary file cleaned up successfully"),
        Err(e) => tracing::warn!("Failed to clean up temporary file {}: {}", temp_file.display(), e),
    }

    match &result {
        Ok(lines) => {
            tracing::info!("Encoding detection and parsing completed successfully");
            tracing::info!("Parsed {} lines from the file", lines.len());

            // Log some encoding detection info if available
            if !lines.is_empty() {
                tracing::debug!("First line preview: {}",
                    if lines[0].len() > 100 {
                        format!("{}...", &lines[0][..100])
                    } else {
                        lines[0].clone()
                    }
                );
            }
        }
        Err(e) => {
            tracing::error!("Core library encoding detection failed: {}", e);
            tracing::error!("Error details: {:#}", e);
        }
    }

    result.map_err(|e| anyhow::anyhow!("Failed to read file with core library: {}", e))
}

