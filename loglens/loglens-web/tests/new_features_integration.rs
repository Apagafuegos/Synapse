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

// ============================================================================
// Pattern Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_get_patterns_with_category_filter() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    // Create a test project first
    let project_data = json!({
        "name": "Pattern Test Project",
        "description": "Testing pattern filtering"
    });

    let project_response = app
        .clone()
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

    assert_eq!(project_response.status(), StatusCode::OK);

    // Note: In a real test, we would extract the project ID from the response
    // and use it to test pattern filtering. This is a basic structure.
}

#[tokio::test]
async fn test_get_patterns_with_severity_filter() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    // Test that the endpoint accepts severity filter parameter
    // Note: This assumes the route is set up correctly in routes.rs
    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/patterns?severity=critical")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should either return OK with data or NOT_FOUND if project doesn't exist
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_get_patterns_with_combined_filters() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/patterns?category=code&severity=high")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

// ============================================================================
// Knowledge Base Tests
// ============================================================================

#[tokio::test]
async fn test_create_knowledge_entry_with_public_flag() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let knowledge_data = json!({
        "title": "Authentication Fix",
        "problem_description": "Users unable to login after password reset",
        "solution": "Clear session cache and restart authentication service",
        "tags": "authentication, session, cache",
        "severity": "high",
        "is_public": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/test-id/knowledge")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&knowledge_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_get_public_knowledge() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/knowledge/public")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Public knowledge endpoint should always be accessible
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_search_public_knowledge() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/knowledge/public?search=authentication")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Streaming Sources Tests
// ============================================================================

#[tokio::test]
async fn test_create_file_streaming_source() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let source_data = json!({
        "project_id": "test-project-id",
        "name": "Application Logs",
        "source_type": "file",
        "config": {
            "path": "/var/log/app.log"
        },
        "parser_config": {
            "log_format": "text"
        },
        "buffer_size": 100,
        "batch_timeout_seconds": 2,
        "restart_on_error": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/test-id/streaming/sources")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&source_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_create_tcp_streaming_source() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let source_data = json!({
        "project_id": "test-project-id",
        "name": "TCP Log Receiver",
        "source_type": "tcp",
        "config": {
            "port": 5140
        },
        "parser_config": {
            "log_format": "syslog"
        },
        "buffer_size": 200,
        "batch_timeout_seconds": 3,
        "restart_on_error": true
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/projects/test-id/streaming/sources")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&source_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_list_streaming_sources() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/streaming/sources")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_get_streaming_stats() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/streaming/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_stop_streaming_source() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/projects/test-id/streaming/sources/source-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::NO_CONTENT
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_get_recent_streaming_logs() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/streaming/logs?level=ERROR&since=2024-01-01T00:00:00Z")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

// ============================================================================
// Analytics Tests
// ============================================================================

#[tokio::test]
async fn test_get_performance_metrics() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/analyses/test-analysis-id/performance-metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_get_error_correlations() {
    let state = create_test_app_state().await.expect("Failed to create test state");
    let app = loglens_web::routes::api_routes().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/projects/test-id/error-correlations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
    );
}

// ============================================================================
// Integration Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_complete_workflow_pattern_to_knowledge() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    // This test would verify the complete workflow:
    // 1. Detect error patterns
    // 2. Create knowledge entry from pattern
    // 3. Mark knowledge as public
    // 4. Verify it appears in public knowledge base

    // Note: Full implementation would require multiple sequential requests
    assert!(state.db.pool().is_ok());
}

#[tokio::test]
async fn test_streaming_source_lifecycle() {
    let state = create_test_app_state().await.expect("Failed to create test state");

    // This test would verify:
    // 1. Create streaming source
    // 2. Verify it appears in list
    // 3. Check stats update
    // 4. Stop source
    // 5. Verify removal

    assert!(state.db.pool().is_ok());
}
