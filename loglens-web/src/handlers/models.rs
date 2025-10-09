use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use crate::{error_handling::AppError, AppState};
use anyhow::Result;
use loglens_core::ai_provider::{create_provider, ModelInfo, ModelListResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsRequest {
    pub provider: String,
    pub api_key: String,
    pub force_refresh: Option<bool>,
}

pub async fn get_available_models(
    State(state): State<AppState>,
    Json(request): Json<ModelsRequest>,
) -> Result<Json<ModelListResponse>, AppError> {
    // Validate provider
    let valid_providers = ["openai", "claude", "gemini", "openrouter"];
    if !valid_providers.contains(&request.provider.as_str()) {
        return Err(AppError::bad_request("Invalid provider"));
    }

    // Check cache if force_refresh is not set
    if !request.force_refresh.unwrap_or(false) {
        if let Ok(cached_models) = get_cached_models(&state, &request.provider).await {
            return Ok(Json(cached_models));
        }
    }

    // Create provider and fetch models
    let provider = create_provider(&request.provider, &request.api_key)
        .map_err(|e| AppError::internal(format!("Failed to create provider: {}", e)))?;

    let models = provider
        .get_available_models()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch models from {}: {}", request.provider, e);
            AppError::ai_provider(&request.provider, e.to_string())
        })?;

    let response = ModelListResponse {
        models: models.clone(),
        cached_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
    };

    // Cache the results
    let _ = cache_models(&state, &request.provider, &response).await;

    Ok(Json(response))
}

async fn get_cached_models(
    state: &AppState,
    provider: &str,
) -> Result<ModelListResponse, Box<dyn std::error::Error + Send + Sync>> {
    let pool = state.db.pool();

    let row = sqlx::query(
        "SELECT available_models, models_last_fetched FROM settings WHERE id = 1"
    )
    .fetch_one(pool)
    .await?;

    let available_models: Option<String> = row.get("available_models");
    let models_last_fetched: Option<String> = row.get("models_last_fetched");

    if let (Some(models_json), Some(fetched_at_str)) = (available_models, models_last_fetched) {
        let fetched_at = chrono::DateTime::parse_from_rfc3339(&fetched_at_str)
            .map_err(|_| "Invalid datetime format")?
            .with_timezone(&chrono::Utc);

        let expires_at = fetched_at + chrono::Duration::hours(24);

        if chrono::Utc::now() < expires_at {
            let models: Vec<ModelInfo> = serde_json::from_str(&models_json)
                .map_err(|_| "Invalid models JSON format")?;

            // Filter models for the requested provider
            let provider_models: Vec<ModelInfo> = models
                .into_iter()
                .filter(|m| m.provider == provider)
                .collect();

            if !provider_models.is_empty() {
                return Ok(ModelListResponse {
                    models: provider_models,
                    cached_at: fetched_at,
                    expires_at,
                });
            }
        }
    }

    Err("No valid cache found".into())
}

async fn cache_models(
    state: &AppState,
    provider: &str,
    response: &ModelListResponse,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = state.db.pool();

    // Get existing cached models
    let existing_models = get_existing_cached_models(state).await.unwrap_or_default();

    // Merge with new models (replace for this provider)
    let mut all_models: Vec<ModelInfo> = existing_models
        .into_iter()
        .filter(|m| m.provider != provider)
        .collect();

    all_models.extend(response.models.clone());

    let models_json = serde_json::to_string(&all_models)?;
    let fetched_at = response.cached_at.to_rfc3339();

    sqlx::query(
        "UPDATE settings SET available_models = ?, models_last_fetched = ? WHERE id = 1"
    )
    .bind(&models_json)
    .bind(&fetched_at)
    .execute(pool)
    .await?;

    Ok(())
}

async fn get_existing_cached_models(state: &AppState) -> Result<Vec<ModelInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = state.db.pool();

    let row = sqlx::query("SELECT available_models FROM settings WHERE id = 1")
        .fetch_one(pool)
        .await?;

    let available_models: Option<String> = row.get("available_models");

    if let Some(models_json) = available_models {
        let models: Vec<ModelInfo> = serde_json::from_str(&models_json)?;
        Ok(models)
    } else {
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearCacheRequest {
    pub provider: Option<String>,
}

pub async fn clear_models_cache(
    State(state): State<AppState>,
    Json(request): Json<ClearCacheRequest>,
) -> Result<Json<&'static str>, AppError> {
    let pool = state.db.pool();

    if let Some(provider) = &request.provider {
        // Clear cache for specific provider
        let existing_models = get_existing_cached_models(&state).await.unwrap_or_default();
        let filtered_models: Vec<ModelInfo> = existing_models
            .into_iter()
            .filter(|m| m.provider != *provider)
            .collect();

        let models_json = serde_json::to_string(&filtered_models)
            .map_err(|e| AppError::internal(format!("Failed to serialize models: {}", e)))?;

        sqlx::query("UPDATE settings SET available_models = ? WHERE id = 1")
            .bind(&models_json)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e))?;
    } else {
        // Clear all cached models
        sqlx::query("UPDATE settings SET available_models = NULL, models_last_fetched = NULL WHERE id = 1")
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e))?;
    }

    Ok(Json("Cache cleared"))
}