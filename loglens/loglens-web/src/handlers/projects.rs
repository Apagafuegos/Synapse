use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::Value;

use crate::{models::*, validation::Validator, AppState};

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(projects))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, StatusCode> {
    // Validate and sanitize input
    let (sanitized_name, sanitized_description) = 
        Validator::validate_project_request(&req.name, req.description.as_ref())
            .map_err(|e: crate::validation::ValidationError| {
                tracing::warn!("Project creation validation failed: {}", e.to_message());
                e.to_status_code()
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
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Created project {} with ID {}", project.name, project.id);
    Ok(Json(project))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    // Validate UUID format
    Validator::validate_uuid(&project_id)
        .map_err(|e: crate::validation::ValidationError| {
            tracing::warn!("Invalid project ID format: {}", e.to_message());
            StatusCode::BAD_REQUEST
        })?;

    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to fetch project {}: {}", project_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(project))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // Validate UUID format
    Validator::validate_uuid(&project_id)
        .map_err(|e: crate::validation::ValidationError| {
            tracing::warn!("Invalid project ID format: {}", e.to_message());
            StatusCode::BAD_REQUEST
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
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if analysis_count.count > 0 {
        tracing::warn!("Cannot delete project {} - has {} analyses", project_id, analysis_count.count);
        return Err(StatusCode::CONFLICT);
    }

    let result = sqlx::query!("DELETE FROM projects WHERE id = ?", project_id)
        .execute(state.db.pool())
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to delete project {}: {}", project_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    tracing::info!("Deleted project {}", project_id);
    Ok(Json(serde_json::json!({ "success": true })))
}
