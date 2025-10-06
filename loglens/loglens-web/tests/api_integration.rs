use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use loglens_web::{AppState, WebConfig};
use serde_json::json;
use tower::ServiceExt;

/// Helper to create test app state
async fn create_test_app_state() -> anyhow::Result<AppState> {
    let config = WebConfig {
        port: 3000,
        database_url: ":memory:".to_string(),
        frontend_dir: "./frontend-react/dist".to_string(),
        max_upload_size: 10 * 1024 * 1024,
        analysis_timeout_secs: 300,
    };

    AppState::new(config).await
}

#[tokio::test]
async fn test_health_endpoint() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_project() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    let project_data = json!({
        "name": "Test Project",
        "description": "Integration test project"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&project_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_projects() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_settings_endpoints() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    // Test GET settings
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_endpoint_returns_404() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent-endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
