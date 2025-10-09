use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use bytes::Bytes;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

use crate::{error_handling::AppError, models::*, validation::Validator, AppState};

/// Helper function to process file upload with transaction support
async fn process_file_upload(
    state: &AppState,
    project_id: String,
    sanitized_filename: String,
    data: Bytes,
    file_path: PathBuf,
) -> Result<LogFile, AppError> {
    // Start a transaction to ensure atomicity
    let mut tx = state.db.pool().begin().await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to start transaction for file upload: {}", e);
        AppError::Database(e)
    })?;

    // Write file to disk asynchronously
    if let Err(e) = tokio::fs::write(&file_path, &data).await {
        tracing::error!("Failed to write file {:?}: {}", file_path, e);
        if let Err(rollback_err) = tx.rollback().await {
            tracing::error!("Failed to rollback transaction: {}", rollback_err);
        }
        return Err(AppError::file_processing(format!("Failed to write file: {}", e)));
    }

    // Calculate actual line count from the uploaded data
    let line_count = count_lines_in_data(&data);

    // Create log file record with actual line count
    let log_file = LogFile::new(
        project_id,
        sanitized_filename,
        data.len() as i64,
        line_count,
        file_path.to_string_lossy().to_string(),
    );

    // Save to database
    if let Err(e) = sqlx::query!(
        "INSERT INTO log_files (id, project_id, filename, file_size, line_count, upload_path, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        log_file.id,
        log_file.project_id,
        log_file.filename,
        log_file.file_size,
        log_file.line_count,
        log_file.upload_path,
        log_file.created_at
    )
    .execute(&mut *tx)
    .await
    {
        tracing::error!("Failed to create file record: {}", e);
        if let Err(rollback_err) = tx.rollback().await {
            tracing::error!("Failed to rollback transaction: {}", rollback_err);
        }
        // Clean up the file
        if let Err(file_err) = tokio::fs::remove_file(&file_path).await {
            tracing::error!("Failed to remove file after DB error: {}", file_err);
        }
        return Err(AppError::Database(e));
    }

    // Commit only if everything succeeded
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit file upload transaction: {}", e);
        // Clean up the file
        if let Err(file_err) = tokio::fs::remove_file(&file_path).await {
            tracing::error!("Failed to remove file after commit error: {}", file_err);
        }
        return Err(AppError::Database(e));
    }

    Ok(log_file)
}

pub async fn upload_log_file(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<LogFile>, AppError> {
    // Validate project ID format
    Validator::validate_uuid(&project_id).map_err(|e: crate::validation::ValidationError| {
        tracing::warn!("Invalid project ID: {}", e.to_message());
        AppError::from(e)
    })?;

    // Verify project exists
    let _project = sqlx::query!("SELECT id FROM projects WHERE id = ?", project_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::not_found(format!("Project {} not found", project_id)))?;

    // Create uploads directory if it doesn't exist
    let upload_dir = PathBuf::from("uploads").join(&project_id);
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::file_processing(format!("Failed to create upload directory: {}", e)))?;

    let mut uploaded_file: Option<LogFile> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let filename = field
                .file_name()
                .ok_or_else(|| AppError::bad_request("No filename provided"))?
                .to_string();

            tracing::info!("Processing uploaded file: '{}'", filename);

            let data = field.bytes().await.map_err(|e: axum::extract::multipart::MultipartError| {
                tracing::error!("Failed to read file bytes: {}", e);
                AppError::bad_request(format!("Failed to read file: {}", e))
            })?;

            // Check for empty file
            if data.is_empty() {
                tracing::warn!("Empty file uploaded: {}", filename);
                return Err(AppError::validation("File cannot be empty"));
            }

            tracing::info!("File size: {} bytes, validating...", data.len());

            // Validate filename and file size
            let sanitized_filename = Validator::validate_file_upload(
                &filename,
                data.len(),
                state.config.max_upload_size,
            )
            .map_err(|e: crate::validation::ValidationError| {
                tracing::error!("File upload validation failed for '{}': {}", filename, e.to_message());
                AppError::from(e)
            })?;

            // Generate unique filename using sanitized filename
            let file_id = uuid::Uuid::new_v4().to_string();
            let file_extension = std::path::Path::new(&sanitized_filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("log");
            let unique_filename = format!("{}.{}", file_id, file_extension);
            let file_path = upload_dir.join(&unique_filename);

            // Process the upload with transaction
            let log_file = process_file_upload(
                &state,
                project_id.clone(),
                sanitized_filename,
                data,
                file_path,
            )
            .await?;

            uploaded_file = Some(log_file);
            break;
        }
    }

    uploaded_file.map(Json).ok_or_else(|| AppError::bad_request("No file uploaded"))
}

pub async fn list_log_files(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<LogFile>>, AppError> {
    let log_files = sqlx::query_as::<_, LogFile>(
        "SELECT id, project_id, filename, file_size, line_count, upload_path, created_at
         FROM log_files WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(&project_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(AppError::Database)?;

    Ok(Json(log_files))
}

pub async fn delete_log_file(
    State(state): State<AppState>,
    Path((project_id, file_id)): Path<(String, String)>,
) -> Result<Json<Value>, AppError> {
    // Get file info first
    let log_file = sqlx::query_as::<_, LogFile>(
        "SELECT id, project_id, filename, file_size, line_count, upload_path, created_at
         FROM log_files WHERE id = ? AND project_id = ?",
    )
    .bind(&file_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to fetch log file {}: {}", file_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Log file {} not found", file_id)))?;

    // Check for active analyses using this file
    let active_analyses = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE log_file_id = ? AND status IN (0, 1)",
        file_id
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!(
            "Failed to check active analyses for file {}: {}",
            file_id,
            e
        );
        AppError::Database(e)
    })?;

    if active_analyses.count > 0 {
        tracing::warn!(
            "Cannot delete file {} - has {} active analyses",
            file_id,
            active_analyses.count
        );
        return Err(AppError::bad_request(
            format!("Cannot delete file with {} active analyses", active_analyses.count)
        ));
    }

    // Check for any completed analyses - warn but allow deletion
    let completed_analyses = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE log_file_id = ?",
        file_id
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to check all analyses for file {}: {}", file_id, e);
        AppError::Database(e)
    })?;

    if completed_analyses.count > 0 {
        tracing::info!(
            "Deleting file {} that has {} analysis records - records will be orphaned",
            file_id,
            completed_analyses.count
        );
    }

    // Start a transaction to ensure atomicity
    let mut tx = state.db.pool().begin().await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to start transaction for file deletion: {}", e);
        AppError::Database(e)
    })?;

    // Delete from database first
    let result = sqlx::query!("DELETE FROM log_files WHERE id = ?", file_id)
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to delete log file {} from database: {}", file_id, e);
            AppError::Database(e)
        })?;

    if result.rows_affected() == 0 {
        tx.rollback().await.ok();
        return Err(AppError::not_found(format!("Log file {} not found", file_id)));
    }

    // Commit the transaction
    tx.commit().await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to commit file deletion transaction: {}", e);
        AppError::Database(e)
    })?;

    // Only delete file from filesystem after successful database deletion
    if let Err(e) = fs::remove_file(&log_file.upload_path).await {
        tracing::warn!(
            "File {} deleted from database but failed to delete from filesystem {}: {}",
            file_id,
            log_file.upload_path,
            e
        );
        // Continue - database deletion succeeded, filesystem cleanup failed
        // This is better than the reverse situation
    } else {
        tracing::info!(
            "Successfully deleted file {} both from database and filesystem",
            file_id
        );
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Count the number of lines in the uploaded file data
/// Handles different line ending types (LF, CRLF, CR)
fn count_lines_in_data(data: &Bytes) -> i64 {
    if data.is_empty() {
        return 0;
    }

    let mut line_count = 0;
    let mut i = 0;
    let bytes = data.as_ref();

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                // LF ending
                line_count += 1;
                i += 1;
            }
            b'\r' => {
                // Check for CRLF or CR ending
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    // CRLF ending - skip both bytes
                    line_count += 1;
                    i += 2;
                } else {
                    // CR only ending
                    line_count += 1;
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    // If the file doesn't end with a newline but has content, count it as a line
    if !bytes.is_empty() && bytes[bytes.len() - 1] != b'\n' && bytes[bytes.len() - 1] != b'\r' {
        line_count += 1;
    }

    line_count
}
