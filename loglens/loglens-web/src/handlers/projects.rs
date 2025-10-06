use axum::{
    extract::{Path, State},
    response::Json,
};
use serde_json::Value;

use crate::{error_handling::AppError, models::*, validation::Validator, AppState};

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, AppError> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(AppError::Database)?;

    Ok(Json(projects))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, AppError> {
    // Validate and sanitize input
    let (sanitized_name, sanitized_description) =
        Validator::validate_project_request(&req.name, req.description.as_ref())
            .map_err(|e: crate::validation::ValidationError| {
                tracing::warn!("Project creation validation failed: {}", e.to_message());
                AppError::from(e)
            })?;

    let project = Project::new(sanitized_name, sanitized_description);

    sqlx::query!(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        project.id,
        project.name,
        project.description,
        project.created_at,
        project.updated_at
    )
    .execute(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to create project: {}", e);
        AppError::Database(e)
    })?;

    tracing::info!("Created project {} with ID {}", project.name, project.id);
    Ok(Json(project))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Project>, AppError> {
    // Validate UUID format
    Validator::validate_uuid(&project_id)
        .map_err(|e: crate::validation::ValidationError| {
            tracing::warn!("Invalid project ID format: {}", e.to_message());
            AppError::from(e)
        })?;

    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to fetch project {}: {}", project_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| AppError::not_found(format!("Project {} not found", project_id)))?;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Validate UUID format
    Validator::validate_uuid(&project_id)
        .map_err(|e: crate::validation::ValidationError| {
            tracing::warn!("Invalid project ID format: {}", e.to_message());
            AppError::from(e)
        })?;

    // Check for existing analyses that would be orphaned
    let analysis_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE project_id = ?",
        project_id
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to check analyses for project {}: {}", project_id, e);
        AppError::Database(e)
    })?;

    if analysis_count.count > 0 {
        tracing::warn!("Cannot delete project {} - has {} analyses", project_id, analysis_count.count);
        return Err(AppError::bad_request(
            format!("Cannot delete project with {} analyses", analysis_count.count)
        ));
    }

    let result = sqlx::query!("DELETE FROM projects WHERE id = ?", project_id)
        .execute(state.db.pool())
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to delete project {}: {}", project_id, e);
            AppError::Database(e)
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Project {} not found", project_id)));
    }

    tracing::info!("Deleted project {}", project_id);
    Ok(Json(serde_json::json!({ "success": true })))
}
