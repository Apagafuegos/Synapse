use axum::{
    extract::{Path, State},
    response::Json,
};
use serde_json::Value;
use sqlx::Row;
use std::path::PathBuf;
use tracing::{info, warn, error};

use crate::{error_handling::AppError, models::*, validation::Validator, AppState};

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<Project>>, AppError> {
    info!("Fetching all projects from database");
    
    // Try the new schema first, fallback to old schema if columns don't exist
    let new_schema_result = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, root_path, loglens_config, project_type, last_accessed, created_at, updated_at FROM projects ORDER BY updated_at DESC"
    )
    .fetch_all(state.db.pool())
    .await;
    
    let projects = match new_schema_result {
        Ok(projects) => {
            info!("Successfully fetched {} projects using new schema", projects.len());
            projects
        }
        Err(e) => {
            warn!("New schema query failed, trying fallback: {}", e);
            
            // Fallback to old schema without CLI integration fields
            let old_schema_projects = sqlx::query_as::<_, Project>(
                "SELECT id, name, description, NULL as root_path, NULL as loglens_config, 'web' as project_type, NULL as last_accessed, created_at, updated_at FROM projects ORDER BY updated_at DESC"
            )
            .fetch_all(state.db.pool())
            .await
            .map_err(|db_e| {
                error!("Both schema queries failed. New schema error: {}. Old schema error: {}", e, db_e);
                AppError::Database(e)
            })?;
            
            info!("Successfully fetched {} projects using old schema fallback", old_schema_projects.len());
            old_schema_projects
        }
    };

    Ok(Json(projects))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, AppError> {
    info!("Creating new project: {}", req.name);
    
    // Validate and sanitize input
    let (sanitized_name, sanitized_description) =
        Validator::validate_project_request(&req.name, req.description.as_ref())
            .map_err(|e: crate::validation::ValidationError| {
                warn!("Project creation validation failed: {}", e.to_message());
                AppError::from(e)
            })?;

    let project = Project::new(sanitized_name, sanitized_description);

    info!("Inserting project into database with ID: {}", project.id);
    
    // Try new schema first
    let new_schema_result = sqlx::query(
        "INSERT INTO projects (id, name, description, root_path, loglens_config, project_type, last_accessed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&project.id)
    .bind(&project.name)
    .bind(&project.description)
    .bind(&project.root_path)
    .bind(&project.loglens_config)
    .bind(&project.project_type)
    .bind(&project.last_accessed)
    .bind(&project.created_at)
    .bind(&project.updated_at)
    .execute(state.db.pool())
    .await;

    match new_schema_result {
        Ok(_) => {
            info!("Successfully created project {} with ID {} using new schema", project.name, project.id);
            return Ok(Json(project));
        }
        Err(e) => {
            warn!("New schema insert failed, trying fallback: {}", e);
        }
    }
    
    // Fallback to old schema
    let result = sqlx::query(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&project.id)
    .bind(&project.name)
    .bind(&project.description)
    .bind(&project.created_at)
    .bind(&project.updated_at)
    .execute(state.db.pool())
    .await;
    
    match result {
        Ok(_) => {
            info!("Successfully created project {} with ID {} using old schema fallback", project.name, project.id);
            Ok(Json(project))
        }
        Err(e) => {
            error!("Failed to create project with both schemas: {}", e);
            error!("Database error details: {:?}", e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Project>, AppError> {
    info!("Fetching project with ID: {}", project_id);
    
    // Validate UUID format
    Validator::validate_uuid(&project_id)
        .map_err(|e: crate::validation::ValidationError| {
            warn!("Invalid project ID format: {}", e.to_message());
            AppError::from(e)
        })?;

    // Try new schema first
    let new_schema_result = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, root_path, loglens_config, project_type, last_accessed, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await;
    
    match new_schema_result {
        Ok(Some(project)) => {
            info!("Successfully found project: {} using new schema", project.name);
            return Ok(Json(project));
        }
        Ok(None) => {
            warn!("Project {} not found with new schema, trying fallback", project_id);
        }
        Err(e) => {
            warn!("New schema query failed for project {}, trying fallback: {}", project_id, e);
        }
    }
    
    // Fallback to old schema
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, NULL as root_path, NULL as loglens_config, 'web' as project_type, NULL as last_accessed, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| {
        error!("Both schema queries failed for project {}: {}", project_id, e);
        AppError::Database(e)
    })?
    .ok_or_else(|| {
        warn!("Project {} not found in either schema", project_id);
        AppError::not_found(format!("Project {} not found", project_id))
    })?;

    info!("Successfully found project: {} using old schema fallback", project.name);
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

pub async fn sync_cli_projects(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    info!("Starting CLI project sync");
    
    // First check if CLI integration columns exist, try to create them if not
    let schema_check = sqlx::query(
        "SELECT COUNT(*) as count FROM pragma_table_info('projects') WHERE name = 'root_path'"
    )
    .fetch_one(state.db.pool())
    .await;
    
    match schema_check {
        Ok(row) if row.get::<i64, usize>(0) == 0 => {
            warn!("CLI integration columns not found, attempting to add them");
            
            // Try to add CLI integration columns
            let migration_result = sqlx::query(
                "ALTER TABLE projects ADD COLUMN root_path TEXT;
                 ALTER TABLE projects ADD COLUMN loglens_config TEXT;
                 ALTER TABLE projects ADD COLUMN last_accessed DATETIME;
                 ALTER TABLE projects ADD COLUMN project_type TEXT DEFAULT 'unknown';"
            )
            .execute(state.db.pool())
            .await;
            
            match migration_result {
                Ok(_) => {
                    info!("Successfully added CLI integration columns");
                }
                Err(e) => {
                    warn!("Failed to add CLI integration columns: {}, continuing with existing schema", e);
                }
            }
        }
        Ok(_) => {
            info!("CLI integration columns already exist");
        }
        Err(e) => {
            warn!("Failed to check schema: {}, continuing anyway", e);
        }
    }
    
    let mut synced_count = 0;
    let mut error_count = 0;
    
    // Get current workspace root (where Cargo.toml is located)
    let workspace_root = std::env::current_dir()
        .map_err(|e| {
            error!("Failed to get current directory: {}", e);
            AppError::internal("Cannot determine workspace root")
        })?;
    
    info!("Scanning for CLI projects in: {}", workspace_root.display());
    
    // Look for projects with .loglens directories
    let projects_dir = workspace_root;
    let mut entries = match std::fs::read_dir(&projects_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read projects directory: {}", e);
            return Err(AppError::internal("Cannot scan for CLI projects"));
        }
    };
    
    while let Ok(Some(entry)) = entries.next().transpose() {
        let path = entry.path();
        
        // Check if this is a directory with a .loglens subdirectory
        if path.is_dir() {
            let loglens_dir = path.join(".loglens");
            if loglens_dir.exists() && loglens_dir.is_dir() {
                // Found a CLI project, try to import it
                info!("Found potential CLI project at: {}", path.display());
                match import_cli_project(&state, &path).await {
                    Ok(imported) => {
                        if imported {
                            synced_count += 1;
                            info!("Successfully imported CLI project from: {}", path.display());
                        } else {
                            info!("CLI project already imported or no metadata: {}", path.display());
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        warn!("Failed to import CLI project from {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    info!("CLI project sync completed: {} imported, {} errors", synced_count, error_count);
    
    Ok(Json(serde_json::json!({
        "synced": synced_count,
        "errors": error_count
    })))
}

async fn import_cli_project(state: &AppState, project_path: &PathBuf) -> Result<bool, AppError> {
    let loglens_dir = project_path.join(".loglens");
    let metadata_path = loglens_dir.join("metadata.json");
    
    if !metadata_path.exists() {
        return Ok(false); // No metadata file to import from
    }
    
    // Read project metadata from JSON file
    let metadata_content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| {
            warn!("Failed to read CLI project metadata at {}: {}", metadata_path.display(), e);
            AppError::internal("Cannot read CLI project metadata")
        })?;
    
    let metadata: serde_json::Value = serde_json::from_str(&metadata_content)
        .map_err(|e| {
            warn!("Failed to parse CLI project metadata from {}: {}", metadata_path.display(), e);
            AppError::internal("Cannot parse CLI project metadata")
        })?;
    
    let project_id = metadata.get("project_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::internal("Missing project_id in CLI metadata"))?
        .to_string();
    
    let project_name = metadata.get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            project_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown-project")
        })
        .to_string();
    
    let description = metadata.get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let root_path = metadata.get("root_path")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| project_path.to_str().unwrap_or(""))
        .to_string();
    
    let project_type = metadata.get("project_type")
        .and_then(|v| v.as_str())
        .unwrap_or("cli")
        .to_string();
    
    let loglens_config = metadata.to_string();
    
    // Check if project already exists in web database
    let exists = sqlx::query(
        "SELECT COUNT(*) as count FROM projects WHERE id = ? OR root_path = ?"
    )
    .bind(&project_id)
    .bind(&root_path)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?
    .get::<i64, usize>(0);
    
    if exists > 0 {
        return Ok(false); // Project already exists, skip
    }
    
    // Create new project in web database
    let project = Project::new_cli_project(
        project_id.clone(),
        project_name,
        description,
        root_path,
        Some(loglens_config),
        project_type,
    );
    
    sqlx::query(
        "INSERT INTO projects (id, name, description, root_path, loglens_config, project_type, last_accessed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&project.id)
    .bind(&project.name)
    .bind(&project.description)
    .bind(&project.root_path)
    .bind(&project.loglens_config)
    .bind(&project.project_type)
    .bind(&project.last_accessed)
    .bind(&project.created_at)
    .bind(&project.updated_at)
    .execute(state.db.pool())
    .await
    .map_err(|e| {
        warn!("Failed to insert CLI project {} into web database: {}", project.id, e);
        AppError::Database(e)
    })?;
    
    info!("Imported CLI project '{}' from {}", project.name, project_path.display());
    Ok(true)
}
