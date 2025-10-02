use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::{models::*, AppState};

#[derive(Deserialize)]
pub struct KnowledgeBaseQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateKnowledgeBaseRequest {
    pub title: String,
    pub problem_description: String,
    pub solution: String,
    pub tags: Option<Vec<String>>,
    pub severity: Option<String>,
    pub is_public: Option<bool>,
}

impl CreateKnowledgeBaseRequest {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if self.title.len() > 255 {
            return Err("Title must be 255 characters or less".to_string());
        }
        if self.problem_description.trim().is_empty() {
            return Err("Problem description cannot be empty".to_string());
        }
        if self.solution.trim().is_empty() {
            return Err("Solution cannot be empty".to_string());
        }
        if let Some(ref severity) = self.severity {
            if !matches!(severity.as_str(), "low" | "medium" | "high" | "critical") {
                return Err("Severity must be one of: low, medium, high, critical".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct PatternQuery {
    pub category: Option<String>,
    pub min_frequency: Option<i32>,
    pub limit: Option<i64>,
}

// Knowledge Base Endpoints
pub async fn create_knowledge_entry(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateKnowledgeBaseRequest>,
) -> Result<Json<KnowledgeBaseEntry>, StatusCode> {
    // Validate the request
    if let Err(error_msg) = req.validate() {
        tracing::warn!("Invalid knowledge base entry request: {}", error_msg);
        return Err(StatusCode::BAD_REQUEST);
    }

    let entry = KnowledgeBaseEntry::new(
        project_id,
        req.title,
        req.problem_description,
        req.solution,
        req.severity.unwrap_or_else(|| "medium".to_string()),
    );

    let tags_json = match req.tags {
        Some(tags) => {
            Some(serde_json::to_string(&tags)
                .map_err(|e| {
                    tracing::error!("Failed to serialize knowledge base tags: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?)
        }
        None => None
    };

    sqlx::query!(
        "INSERT INTO knowledge_base (id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        entry.id,
        entry.project_id,
        entry.title,
        entry.problem_description,
        entry.solution,
        tags_json,
        entry.severity,
        entry.is_public,
        entry.usage_count,
        entry.created_at,
        entry.updated_at
    )
    .execute(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        tracing::error!("Failed to create knowledge base entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(entry))
}

pub async fn get_knowledge_entries(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<KnowledgeBaseQuery>,
) -> Result<Json<Vec<KnowledgeBaseEntry>>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    if let Some(search) = &params.search {
        let search_pattern = format!("%{}%", search);
        let entries = sqlx::query_as::<_, KnowledgeBaseEntry>(
            "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
             FROM knowledge_base 
             WHERE project_id = $1 AND (title LIKE $2 OR problem_description LIKE $3 OR solution LIKE $4)
             ORDER BY usage_count DESC, created_at DESC LIMIT $5 OFFSET $6"
        )
        .bind(&project_id)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(entries));
    }

    let entries = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
         FROM knowledge_base 
         WHERE project_id = $1 
         ORDER BY usage_count DESC, created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(&project_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries))
}

pub async fn get_knowledge_entry(
    State(state): State<AppState>,
    Path((project_id, entry_id)): Path<(String, String)>,
) -> Result<Json<KnowledgeBaseEntry>, StatusCode> {
    let entry = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
         FROM knowledge_base WHERE id = $1 AND project_id = $2"
    )
    .bind(&entry_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Increment usage count
    sqlx::query!(
        "UPDATE knowledge_base SET usage_count = usage_count + 1, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        entry_id
    )
    .execute(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entry))
}

// Error Pattern Endpoints
pub async fn get_error_patterns(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<PatternQuery>,
) -> Result<Json<Vec<ErrorPattern>>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100);

    if let Some(category) = &params.category {
        let patterns = sqlx::query_as::<_, ErrorPattern>(
            "SELECT id, project_id, pattern, category, description, frequency, last_seen, suggested_solution, created_at, updated_at
             FROM error_patterns 
             WHERE project_id = $1 AND category = $2
             ORDER BY frequency DESC, last_seen DESC LIMIT $3"
        )
        .bind(&project_id)
        .bind(category)
        .bind(limit)
        .fetch_all(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(patterns));
    }

    if let Some(min_freq) = &params.min_frequency {
        let patterns = sqlx::query_as::<_, ErrorPattern>(
            "SELECT id, project_id, pattern, category, description, frequency, last_seen, suggested_solution, created_at, updated_at
             FROM error_patterns 
             WHERE project_id = $1 AND frequency >= $2
             ORDER BY frequency DESC, last_seen DESC LIMIT $3"
        )
        .bind(&project_id)
        .bind(min_freq)
        .bind(limit)
        .fetch_all(state.db.pool())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(patterns));
    }

    let patterns = sqlx::query_as::<_, ErrorPattern>(
        "SELECT id, project_id, pattern, category, description, frequency, last_seen, suggested_solution, created_at, updated_at
         FROM error_patterns 
         WHERE project_id = $1 
         ORDER BY frequency DESC, last_seen DESC LIMIT $2"
    )
    .bind(&project_id)
    .bind(limit)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(patterns))
}

pub async fn update_pattern_frequency(
    State(state): State<AppState>,
    Path(pattern_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!(
        "UPDATE error_patterns SET frequency = frequency + 1, last_seen = CURRENT_TIMESTAMP WHERE id = $1",
        pattern_id
    )
    .execute(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

// Error Correlation Endpoints
pub async fn get_error_correlations(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Path(analysis_id): Path<String>,
) -> Result<Json<Vec<ErrorCorrelation>>, StatusCode> {
    let correlations = sqlx::query_as::<_, ErrorCorrelation>(
        "SELECT id, project_id, primary_error_id, correlated_error_id, correlation_strength, correlation_type, created_at
         FROM error_correlations 
         WHERE project_id = $1 AND (primary_error_id = $2 OR correlated_error_id = $2)
         ORDER BY correlation_strength DESC"
    )
    .bind(&project_id)
    .bind(&analysis_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(correlations))
}

// Performance Metrics Endpoints
pub async fn get_performance_metrics(
    State(state): State<AppState>,
    Path(analysis_id): Path<String>,
) -> Result<Json<Vec<PerformanceMetric>>, StatusCode> {
    let metrics = sqlx::query_as::<_, PerformanceMetric>(
        "SELECT id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at
         FROM performance_metrics 
         WHERE analysis_id = $1
         ORDER BY is_bottleneck DESC, metric_name"
    )
    .bind(&analysis_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}

pub async fn create_performance_metric(
    State(state): State<AppState>,
    Path(_analysis_id): Path<String>,
    Json(metric): Json<PerformanceMetric>,
) -> Result<Json<PerformanceMetric>, StatusCode> {
    sqlx::query!(
        "INSERT INTO performance_metrics (id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        metric.id,
        metric.analysis_id,
        metric.metric_name,
        metric.metric_value,
        metric.unit,
        metric.threshold_value,
        metric.is_bottleneck,
        metric.created_at
    )
    .execute(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metric))
}

// Pattern Recognition Service
pub async fn recognize_patterns(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(error_messages): Json<Vec<String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get existing patterns for this project
    let existing_patterns = sqlx::query_as::<_, ErrorPattern>(
        "SELECT id, project_id, pattern, category, description, frequency, last_seen, suggested_solution, created_at, updated_at
         FROM error_patterns WHERE project_id = $1"
    )
    .bind(&project_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut patterns_by_category = serde_json::Map::new();

    // Match error messages against patterns
    for error_msg in error_messages {
        for pattern in &existing_patterns {
            if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                if regex.is_match(&error_msg) {
                    let category = &pattern.category;
                    let pattern_data = serde_json::json!({
                        "id": pattern.id,
                        "pattern": pattern.pattern,
                        "description": pattern.description,
                        "frequency": pattern.frequency,
                        "suggested_solution": pattern.suggested_solution
                    });

                    if let Some(category_patterns) = patterns_by_category.get_mut(category) {
                        if let Some(array) = category_patterns.as_array_mut() {
                            array.push(pattern_data);
                        }
                    } else {
                        patterns_by_category.insert(
                            category.clone(),
                            serde_json::Value::Array(vec![pattern_data]),
                        );
                    }

                    // Update pattern frequency
                    let _ =
                        update_pattern_frequency(State(state.clone()), Path(pattern.id.clone()))
                            .await;
                }
            }
        }
    }

    Ok(Json(serde_json::Value::Object(patterns_by_category)))
}
