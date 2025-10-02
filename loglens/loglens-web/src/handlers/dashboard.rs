use axum::{extract::State, http::StatusCode, response::Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{models::AnalysisStatus, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_projects: i64,
    pub analyses_this_week: i64,
    pub avg_processing_time_minutes: Option<f64>,
    pub critical_errors: i64,
}

pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, StatusCode> {
    // Get total projects
    let total_projects = sqlx::query!("SELECT COUNT(*) as count FROM projects")
        .fetch_one(state.db.pool())
        .await
        .map_err(|e: sqlx::Error| {
            tracing::error!("Failed to count projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .count;

    // Get analyses from the last week
    let one_week_ago = Utc::now() - Duration::weeks(1);
    let analyses_this_week = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE started_at >= ?",
        one_week_ago
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to count weekly analyses: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .count;

    // Get average processing time for completed analyses (in the last 30 days)
    let thirty_days_ago = Utc::now() - Duration::days(30);
    let avg_processing_time_minutes = sqlx::query!(
        r#"
        SELECT AVG(
            CAST(
                (julianday(completed_at) - julianday(started_at)) * 24 * 60
                as REAL
            )
        ) as avg_minutes
        FROM analyses
        WHERE status = ? AND completed_at IS NOT NULL AND started_at >= ?
        "#,
        AnalysisStatus::Completed as i32,
        thirty_days_ago
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to calculate average processing time: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .avg_minutes;

    // Get critical errors count (failed analyses in the last 7 days)
    let critical_errors = sqlx::query!(
        "SELECT COUNT(*) as count FROM analyses WHERE status = ? AND started_at >= ?",
        AnalysisStatus::Failed as i32,
        one_week_ago
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to count critical errors: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .count;

    let stats = DashboardStats {
        total_projects: total_projects.into(),
        analyses_this_week: analyses_this_week.into(),
        avg_processing_time_minutes,
        critical_errors: critical_errors.into(),
    };

    Ok(Json(stats))
}