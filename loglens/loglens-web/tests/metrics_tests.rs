use loglens_web::middleware::metrics::MetricsCollector;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_metrics_collector_initialization() {
    let collector = MetricsCollector::new();

    let summary = collector.get_metrics_summary();

    assert_eq!(summary.request_count, 0);
    assert_eq!(summary.error_count, 0);
    assert_eq!(summary.error_rate, 0.0);
}

#[tokio::test]
async fn test_metrics_record_request() {
    let collector = Arc::new(MetricsCollector::new());

    // Record a successful request
    collector.record_request("/api/test", Duration::from_millis(100), 200);

    let summary = collector.get_metrics_summary();

    assert_eq!(summary.request_count, 1);
    assert_eq!(summary.error_count, 0);
    assert_eq!(summary.error_rate, 0.0);
}

#[tokio::test]
async fn test_metrics_record_error() {
    let collector = Arc::new(MetricsCollector::new());

    // Record an error request
    collector.record_request("/api/test", Duration::from_millis(100), 500);

    let summary = collector.get_metrics_summary();

    assert_eq!(summary.request_count, 1);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.error_rate, 1.0);
}

#[tokio::test]
async fn test_metrics_error_rate_calculation() {
    let collector = Arc::new(MetricsCollector::new());

    // Record 3 successful and 1 error request
    collector.record_request("/api/test1", Duration::from_millis(100), 200);
    collector.record_request("/api/test2", Duration::from_millis(150), 200);
    collector.record_request("/api/test3", Duration::from_millis(120), 200);
    collector.record_request("/api/test4", Duration::from_millis(200), 500);

    let summary = collector.get_metrics_summary();

    assert_eq!(summary.request_count, 4);
    assert_eq!(summary.error_count, 1);
    assert_eq!(summary.error_rate, 0.25);
}

#[tokio::test]
async fn test_metrics_analysis_result_recording() {
    let collector = Arc::new(MetricsCollector::new());

    // Record successful analysis
    collector.record_analysis_result(0.95, 0.90, true);

    let summary = collector.get_metrics_summary();

    assert!(summary.quality_metrics.analysis_accuracy > 0.0);
    assert!(summary.quality_metrics.average_confidence_score > 0.0);
}

#[tokio::test]
async fn test_metrics_average_response_time() {
    let collector = Arc::new(MetricsCollector::new());

    collector.record_request("/api/test1", Duration::from_millis(100), 200);
    collector.record_request("/api/test2", Duration::from_millis(200), 200);
    collector.record_request("/api/test3", Duration::from_millis(300), 200);

    let avg = collector.get_average_response_time();

    // Average should be 200ms
    assert_eq!(avg.as_millis(), 200);
}

#[tokio::test]
async fn test_quality_score_calculation() {
    use loglens_web::middleware::metrics::{calculate_quality_score, QualityMetrics};

    let metrics = QualityMetrics {
        analysis_accuracy: 0.95,
        analysis_completion_rate: 0.98,
        average_confidence_score: 0.90,
        error_classification_accuracy: 0.92,
        system_availability: 0.999,
        memory_usage_mb: 500,
        cpu_usage_percent: 25.0,
        active_connections: 10,
        database_query_avg_time: Duration::from_millis(50),
        cache_hit_rate: 0.85,
    };

    let score = calculate_quality_score(&metrics);

    assert!(score > 0.0 && score <= 1.0);
    assert!(score > 0.8); // Should be high quality with these metrics
}

#[tokio::test]
async fn test_metrics_alert_subscription() {
    let collector = Arc::new(MetricsCollector::new());

    let mut _rx = collector.subscribe_to_alerts();

    // Just verify we can subscribe without panic
    assert!(true);
}
