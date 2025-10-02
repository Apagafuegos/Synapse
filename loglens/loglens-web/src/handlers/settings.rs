use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use anyhow::Result;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: String,
    pub api_key: String,
    pub max_lines: u32,
    pub default_level: String,
    pub show_timestamps: bool,
    pub show_line_numbers: bool,
    pub selected_model: Option<String>,
    pub available_models: Option<String>, // JSON array cache
    pub models_last_fetched: Option<String>, // ISO datetime
    pub analysis_timeout_seconds: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_provider: "openrouter".to_string(),
            api_key: String::new(),
            max_lines: 1000,
            default_level: "ERROR".to_string(),
            show_timestamps: true,
            show_line_numbers: true,
            selected_model: None,
            available_models: None,
            models_last_fetched: None,
            analysis_timeout_seconds: Some(300), // 5 minutes default
        }
    }
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Settings>, StatusCode> {
    let pool = state.db.pool();
    
    // Query settings from database
    let result = sqlx::query(
        "SELECT default_provider, api_key, max_lines, default_level, show_timestamps, show_line_numbers,
                selected_model, available_models, models_last_fetched, analysis_timeout_seconds
         FROM settings WHERE id = 1"
    )
    .fetch_one(pool)
    .await;
    
    match result {
        Ok(row) => {
            let settings = Settings {
                default_provider: row.get("default_provider"),
                api_key: row.get("api_key"),
                max_lines: row.get("max_lines"),
                default_level: row.get("default_level"),
                show_timestamps: row.get("show_timestamps"),
                show_line_numbers: row.get("show_line_numbers"),
                selected_model: row.get("selected_model"),
                available_models: row.get("available_models"),
                models_last_fetched: row.get("models_last_fetched"),
                analysis_timeout_seconds: row.get("analysis_timeout_seconds"),
            };
            Ok(Json(settings))
        }
        Err(e) => {
            tracing::error!("Failed to fetch settings: {}", e);
            // Return default settings if database query fails
            Ok(Json(Settings::default()))
        }
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<Settings>,
) -> Result<Json<Settings>, StatusCode> {
    let pool = state.db.pool();
    
    // Validate the provider
    let valid_providers = ["openai", "claude", "gemini", "openrouter", "mock"];
    if !valid_providers.contains(&settings.default_provider.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate log level
    let valid_levels = ["ERROR", "WARN", "INFO", "DEBUG"];
    if !valid_levels.contains(&settings.default_level.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate max_lines
    if settings.max_lines < 100 || settings.max_lines > 10000 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate timeout if provided
    if let Some(timeout) = settings.analysis_timeout_seconds {
        if !(60..=1800).contains(&timeout) { // 1 minute to 30 minutes
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Update settings in database
    let result = sqlx::query(
        "UPDATE settings
         SET default_provider = ?, api_key = ?, max_lines = ?, default_level = ?,
             show_timestamps = ?, show_line_numbers = ?, selected_model = ?,
             available_models = ?, models_last_fetched = ?, analysis_timeout_seconds = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = 1"
    )
    .bind(&settings.default_provider)
    .bind(&settings.api_key)
    .bind(settings.max_lines)
    .bind(&settings.default_level)
    .bind(settings.show_timestamps)
    .bind(settings.show_line_numbers)
    .bind(&settings.selected_model)
    .bind(&settings.available_models)
    .bind(&settings.models_last_fetched)
    .bind(settings.analysis_timeout_seconds)
    .execute(pool)
    .await;
    
    match result {
        Ok(_) => {
            // Return the updated settings
            Ok(Json(settings))
        }
        Err(e) => {
            tracing::error!("Failed to update settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}