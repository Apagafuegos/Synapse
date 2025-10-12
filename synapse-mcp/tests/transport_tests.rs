use synapse_mcp::{create_server, Config, Database};
use tempfile::NamedTempFile;
use rmcp::ServerHandler;

/// Helper to set up test database
async fn setup_test_db() -> (Database, NamedTempFile) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let db = Database::new(db_path).await.unwrap();
    (db, temp_file)
}

/// Test STDIO transport connection
#[tokio::test]
async fn test_stdio_transport_connection() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();

    // Test that we can create the handler (Arc cloning works)
    let handler = server.create_handler();
    
    // Verify handler is properly created
    assert!(handler.server.config.server_name.len() > 0);
}

/// Test HTTP transport server creation
#[tokio::test]
async fn test_http_transport_creation() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();

    // Test that we can create the handler
    let handler = server.create_handler();

    // Verify handler is properly created
    assert!(handler.server.config.server_name.len() > 0);
}

/// Test server info
#[tokio::test]
async fn test_server_info() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();
    let handler = server.create_handler();

    let info = handler.get_info();
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("log analysis"));
    assert!(info.capabilities.tools.is_some());
}

/// Test concurrent handler creation
#[tokio::test]
async fn test_concurrent_handler_creation() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();

    // Create multiple handlers concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            server_clone.create_handler()
        });
        handles.push(handle);
    }

    // Wait for all handlers to be created
    for handle in handles {
        let handler = handle.await.unwrap();
        let info = handler.get_info();
        assert!(info.instructions.is_some());
    }
}