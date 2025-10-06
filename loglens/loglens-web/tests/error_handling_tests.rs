use loglens_web::error_handling::AppError;
use axum::{http::StatusCode, response::IntoResponse};

#[test]
fn test_app_error_validation() {
    let error = AppError::validation("Invalid input");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_app_error_not_found() {
    let error = AppError::not_found("Resource not found");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_app_error_bad_request() {
    let error = AppError::bad_request("Bad request");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_app_error_internal() {
    let error = AppError::internal("Internal error");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_ai_provider() {
    let error = AppError::ai_provider("openai", "API error");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_app_error_file_processing() {
    let error = AppError::file_processing("Failed to process file");

    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[test]
fn test_app_error_from_sqlx() {
    use sqlx::Error as SqlxError;

    let sqlx_error = SqlxError::RowNotFound;
    let app_error: AppError = sqlx_error.into();

    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[cfg(test)]
mod validation_tests {
    use loglens_web::error_handling::validation::*;

    #[test]
    fn test_validate_non_empty_success() {
        let result = validate_non_empty("test", "field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_non_empty_failure() {
        let result = validate_non_empty("", "field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_non_empty_whitespace() {
        let result = validate_non_empty("   ", "field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_max_length_success() {
        let result = validate_max_length("test", 10, "field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_max_length_failure() {
        let result = validate_max_length("test string that is too long", 10, "field");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_level_valid() {
        assert!(validate_log_level("ERROR").is_ok());
        assert!(validate_log_level("WARN").is_ok());
        assert!(validate_log_level("INFO").is_ok());
        assert!(validate_log_level("DEBUG").is_ok());
    }

    #[test]
    fn test_validate_log_level_invalid() {
        let result = validate_log_level("INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_level_case_insensitive() {
        assert!(validate_log_level("error").is_ok());
        assert!(validate_log_level("WaRn").is_ok());
    }

    #[test]
    fn test_validate_ai_provider_valid() {
        assert!(validate_ai_provider("openrouter").is_ok());
        assert!(validate_ai_provider("openai").is_ok());
        assert!(validate_ai_provider("claude").is_ok());
        assert!(validate_ai_provider("gemini").is_ok());
    }

    #[test]
    fn test_validate_ai_provider_invalid() {
        let result = validate_ai_provider("invalid_provider");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_size_success() {
        let result = validate_file_size(1000, 2000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_failure() {
        let result = validate_file_size(3000, 2000);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pagination_defaults() {
        let result = validate_pagination(None, None);
        assert!(result.is_ok());

        let (limit, offset) = result.unwrap();
        assert_eq!(limit, 50);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_validate_pagination_custom() {
        let result = validate_pagination(Some(100), Some(50));
        assert!(result.is_ok());

        let (limit, offset) = result.unwrap();
        assert_eq!(limit, 100);
        assert_eq!(offset, 50);
    }

    #[test]
    fn test_validate_pagination_limit_too_high() {
        let result = validate_pagination(Some(2000), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pagination_limit_too_low() {
        let result = validate_pagination(Some(0), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_pagination_negative_offset() {
        let result = validate_pagination(Some(50), Some(-1));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uuid_valid() {
        let result = validate_uuid("550e8400-e29b-41d4-a716-446655440000", "id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        let result = validate_uuid("not-a-uuid", "id");
        assert!(result.is_err());
    }
}
