// Core Module Integration Tests
// Basic unit tests to ensure core functionality works

use loglens_core::*;

#[test]
fn test_parse_log_lines_basic() {
    let lines = vec![
        "2023-12-01 10:30:45 INFO Application started".to_string(),
        "2023-12-01 10:30:46 ERROR Database connection failed".to_string(),
    ];
    
    let result = parse_log_lines(&lines);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].level, Some("INFO".to_string()));
    assert_eq!(result[1].level, Some("ERROR".to_string()));
}

#[test]
fn test_filter_logs_by_level() {
    let entries = vec![
        LogEntry {
            timestamp: Some("2023-12-01 10:30:45".to_string()),
            level: Some("INFO".to_string()),
            message: "Application started".to_string(),
            line_number: Some(1),
        },
        LogEntry {
            timestamp: Some("2023-12-01 10:30:46".to_string()),
            level: Some("ERROR".to_string()),
            message: "Database connection failed".to_string(),
            line_number: Some(2),
        },
    ];
    
    let result = filter_logs_by_level(entries, "ERROR").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].level, Some("ERROR".to_string()));
}

#[test]
fn test_slim_logs() {
    let entries = vec![
        LogEntry {
            timestamp: Some("2023-12-01 10:30:45".to_string()),
            level: Some("INFO".to_string()),
            message: "Test message 1".to_string(),
            line_number: Some(1),
        },
        LogEntry {
            timestamp: Some("2023-12-01 10:30:46".to_string()),
            level: Some("INFO".to_string()),
            message: "Test message 2".to_string(),
            line_number: Some(2),
        },
    ];
    
    let result = slim_logs(entries);
    assert!(!result.is_empty());
}

#[test]
fn test_error_classifier() {
    let classifier = ErrorClassifier::new();
    let result = classifier.classify_error("Database connection failed", None);
    assert!(result.confidence > 0.0);
    assert!(!result.reason.is_empty());
}

#[test]
fn test_config_loading() {
    let config = Config::load();
    assert!(config.is_ok());
}

#[test]
fn test_log_entry_creation() {
    let entry = LogEntry {
        timestamp: Some("2023-12-01 10:30:45".to_string()),
        level: Some("INFO".to_string()),
        message: "Test message".to_string(),
        line_number: Some(1),
    };
    
    assert_eq!(entry.timestamp, Some("2023-12-01 10:30:45".to_string()));
    assert_eq!(entry.level, Some("INFO".to_string()));
    assert_eq!(entry.message, "Test message");
    assert_eq!(entry.line_number, Some(1));
}

#[tokio::test]
async fn test_read_log_file_integration() {
    use std::fs;
    use tempfile::tempdir;
    
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.log");
    let content = "2023-12-01 10:30:45 INFO Test log message\n";
    fs::write(&file_path, content).unwrap();

    let result = read_log_file(file_path.to_str().unwrap()).await;
    assert!(result.is_ok());
    
    let lines = result.unwrap();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Test log message"));
}

#[tokio::test]
async fn test_execute_and_capture_integration() {
    let result = execute_and_capture("echo 'test output'").await;
    assert!(result.is_ok());
    
    let lines = result.unwrap();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("test output"));
}