use axum::{
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("File processing error: {message}")]
    FileProcessing { message: String },
    
    #[error("AI provider error: {provider} - {message}")]
    AIProvider { provider: String, message: String },
    
    #[error("Cache error: {message}")]
    Cache { message: String },
    
    #[error("Circuit breaker is open: {service}")]
    CircuitBreakerOpen { service: String },
    
    #[error("Internal server error: {message}")]
    Internal { message: String },
    
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    
    #[error("Service temporarily unavailable: {service}")]
    ServiceUnavailable { service: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    pub timestamp: String,
    pub trace_id: Option<String>,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ErrorResponse {
    pub fn new(error_type: &str, message: String, code: String) -> Self {
        Self {
            error: error_type.to_string(),
            message,
            code,
            timestamp: chrono::Utc::now().to_rfc3339(),
            trace_id: None,
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            AppError::Database(ref e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new(
                        "database_error",
                        "A database error occurred".to_string(),
                        "DB_ERROR".to_string(),
                    ),
                )
            }
            
            AppError::Validation { ref message } => {
                warn!("Validation error: {}", message);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(
                        "validation_error",
                        message.clone(),
                        "VALIDATION_FAILED".to_string(),
                    ),
                )
            }
            
            AppError::NotFound { ref resource } => {
                warn!("Resource not found: {}", resource);
                (
                    StatusCode::NOT_FOUND,
                    ErrorResponse::new(
                        "not_found",
                        format!("Resource not found: {}", resource),
                        "NOT_FOUND".to_string(),
                    ),
                )
            }
            
            AppError::Unauthorized => {
                warn!("Unauthorized access attempt");
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse::new(
                        "unauthorized",
                        "Authentication required".to_string(),
                        "AUTH_REQUIRED".to_string(),
                    ),
                )
            }
            
            AppError::RateLimitExceeded => {
                warn!("Rate limit exceeded");
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorResponse::new(
                        "rate_limit_exceeded",
                        "Too many requests. Please try again later".to_string(),
                        "RATE_LIMIT".to_string(),
                    ),
                )
            }
            
            AppError::FileProcessing { ref message } => {
                error!("File processing error: {}", message);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorResponse::new(
                        "file_processing_error",
                        message.clone(),
                        "FILE_PROCESSING".to_string(),
                    ),
                )
            }
            
            AppError::AIProvider { ref provider, ref message } => {
                error!("AI provider error - {}: {}", provider, message);
                let mut details = HashMap::new();
                details.insert("provider".to_string(), serde_json::Value::String(provider.clone()));
                
                (
                    StatusCode::BAD_GATEWAY,
                    ErrorResponse::new(
                        "ai_provider_error",
                        format!("AI service error: {}", message),
                        "AI_SERVICE_ERROR".to_string(),
                    ).with_details(details),
                )
            }
            
            AppError::Cache { ref message } => {
                error!("Cache error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new(
                        "cache_error",
                        "Cache service error".to_string(),
                        "CACHE_ERROR".to_string(),
                    ),
                )
            }
            
            AppError::CircuitBreakerOpen { ref service } => {
                warn!("Circuit breaker open for service: {}", service);
                let mut details = HashMap::new();
                details.insert("service".to_string(), serde_json::Value::String(service.clone()));
                
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorResponse::new(
                        "service_unavailable",
                        format!("Service temporarily unavailable: {}", service),
                        "CIRCUIT_BREAKER_OPEN".to_string(),
                    ).with_details(details),
                )
            }
            
            AppError::ServiceUnavailable { ref service } => {
                error!("Service unavailable: {}", service);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorResponse::new(
                        "service_unavailable",
                        format!("Service unavailable: {}", service),
                        "SERVICE_DOWN".to_string(),
                    ),
                )
            }
            
            AppError::BadRequest { ref message } => {
                warn!("Bad request: {}", message);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(
                        "bad_request",
                        message.clone(),
                        "BAD_REQUEST".to_string(),
                    ),
                )
            }
            
            AppError::Internal { ref message } => {
                error!("Internal error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new(
                        "internal_error",
                        "An internal error occurred".to_string(),
                        "INTERNAL_ERROR".to_string(),
                    ),
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

// Helper functions for creating specific errors
impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    pub fn file_processing(message: impl Into<String>) -> Self {
        Self::FileProcessing {
            message: message.into(),
        }
    }

    pub fn ai_provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AIProvider {
            provider: provider.into(),
            message: message.into(),
        }
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    pub fn circuit_breaker_open(service: impl Into<String>) -> Self {
        Self::CircuitBreakerOpen {
            service: service.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            service: service.into(),
        }
    }
}

// 404 handler
pub async fn handle_404(uri: Uri) -> impl IntoResponse {
    let error_response = ErrorResponse::new(
        "not_found",
        format!("No route found for {}", uri.path()),
        "ROUTE_NOT_FOUND".to_string(),
    );

    (StatusCode::NOT_FOUND, Json(error_response))
}

// Generic error handler for unhandled errors
pub async fn handle_error(error: Box<dyn std::error::Error>) -> impl IntoResponse {
    error!("Unhandled error: {}", error);

    let error_response = ErrorResponse::new(
        "internal_error",
        "An unexpected error occurred".to_string(),
        "UNHANDLED_ERROR".to_string(),
    );

    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
}

// Result type alias
pub type AppResult<T> = Result<T, AppError>;

// Validation helpers
pub mod validation {
    use super::*;
    use uuid::Uuid;

    pub fn validate_uuid(id: &str, field_name: &str) -> AppResult<Uuid> {
        Uuid::parse_str(id).map_err(|_| {
            AppError::validation(format!("Invalid {}: must be a valid UUID", field_name))
        })
    }

    pub fn validate_non_empty(value: &str, field_name: &str) -> AppResult<()> {
        if value.trim().is_empty() {
            Err(AppError::validation(format!("{} cannot be empty", field_name)))
        } else {
            Ok(())
        }
    }

    pub fn validate_max_length(value: &str, max_length: usize, field_name: &str) -> AppResult<()> {
        if value.len() > max_length {
            Err(AppError::validation(format!(
                "{} must be {} characters or less",
                field_name, max_length
            )))
        } else {
            Ok(())
        }
    }

    pub fn validate_log_level(level: &str) -> AppResult<()> {
        match level.to_uppercase().as_str() {
            "DEBUG" | "INFO" | "WARN" | "ERROR" => Ok(()),
            _ => Err(AppError::validation(
                "Log level must be one of: DEBUG, INFO, WARN, ERROR".to_string(),
            )),
        }
    }

    pub fn validate_ai_provider(provider: &str) -> AppResult<()> {
        match provider.to_lowercase().as_str() {
            "openrouter" | "openai" | "claude" | "gemini" => Ok(()),
            _ => Err(AppError::validation(
                "AI provider must be one of: openrouter, openai, claude, gemini".to_string(),
            )),
        }
    }

    pub fn validate_file_size(size: u64, max_size: u64) -> AppResult<()> {
        if size > max_size {
            Err(AppError::validation(format!(
                "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                size, max_size
            )))
        } else {
            Ok(())
        }
    }

    pub fn validate_pagination(limit: Option<i64>, offset: Option<i64>) -> AppResult<(i64, i64)> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        if !(1..=1000).contains(&limit) {
            return Err(AppError::validation(
                "Limit must be between 1 and 1000".to_string(),
            ));
        }

        if offset < 0 {
            return Err(AppError::validation(
                "Offset must be non-negative".to_string(),
            ));
        }

        Ok((limit, offset))
    }
}

// Health check types and functions
#[derive(Serialize, Deserialize, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<f64>,
    pub error: Option<String>,
    pub last_check: String,
}

pub async fn check_database_health(pool: &sqlx::Pool<sqlx::Sqlite>) -> ServiceHealth {
    let start = std::time::Instant::now();
    
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as f64),
            error: None,
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as f64),
            error: Some(e.to_string()),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    }
}

pub async fn check_cache_health(cache_manager: &crate::cache::CacheManager) -> ServiceHealth {
    let start = std::time::Instant::now();

    // Test cache operation with a minimal test analysis
    let test_key = "health_check_test".to_string();
    let test_analysis = crate::models::Analysis {
        id: "health-test".to_string(),
        project_id: "health-test".to_string(),
        log_file_id: None,
        analysis_type: "test".to_string(),
        provider: "test".to_string(),
        level_filter: "INFO".to_string(),
        status: crate::models::AnalysisStatus::Pending,
        result: None,
        error_message: None,
        started_at: chrono::Utc::now(),
        completed_at: None,
    };

    cache_manager.analysis_cache.put(test_key.clone(), test_analysis, None);

    match cache_manager.analysis_cache.get(&test_key) {
        Some(_) => {
            cache_manager.analysis_cache.remove(&test_key);
            ServiceHealth {
                status: "healthy".to_string(),
                response_time_ms: Some(start.elapsed().as_millis() as f64),
                error: None,
                last_check: chrono::Utc::now().to_rfc3339(),
            }
        },
        None => ServiceHealth {
            status: "unhealthy".to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as f64),
            error: Some("Cache test failed".to_string()),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    }
}

// Middleware for request tracing
use axum::{extract::Request, middleware::Next};

pub async fn trace_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let trace_id = uuid::Uuid::new_v4().to_string();
    tracing::info!(
        trace_id = %trace_id,
        method = %method,
        uri = %uri,
        "Request started"
    );

    let response = next.run(request).await;
    
    let status = response.status();
    let duration = start.elapsed();

    tracing::info!(
        trace_id = %trace_id,
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_helpers() {
        // Test UUID validation
        assert!(validation::validate_uuid("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
        assert!(validation::validate_uuid("invalid-uuid", "id").is_err());

        // Test empty string validation
        assert!(validation::validate_non_empty("valid", "name").is_ok());
        assert!(validation::validate_non_empty("", "name").is_err());
        assert!(validation::validate_non_empty("  ", "name").is_err());

        // Test max length validation
        assert!(validation::validate_max_length("short", 10, "field").is_ok());
        assert!(validation::validate_max_length("this is too long", 5, "field").is_err());

        // Test log level validation
        assert!(validation::validate_log_level("ERROR").is_ok());
        assert!(validation::validate_log_level("info").is_ok());
        assert!(validation::validate_log_level("INVALID").is_err());

        // Test pagination validation
        assert!(validation::validate_pagination(Some(50), Some(0)).is_ok());
        assert!(validation::validate_pagination(Some(2000), Some(0)).is_err());
        assert!(validation::validate_pagination(Some(50), Some(-1)).is_err());
    }

    #[test]
    fn test_error_response_creation() {
        let error = AppError::validation("Test validation error");
        let response = error.into_response();
        // This would test the response in a real test environment
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}