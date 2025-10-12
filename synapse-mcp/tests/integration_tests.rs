// Integration tests for Synapse MCP server
// These tests verify the MCP protocol implementation and tool functionality

use synapse_mcp::{create_server, Config, Database};
use serde_json::json;
use tempfile::TempDir;

/// Helper to create a test database
async fn setup_test_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Use SQLite create mode to allow database creation
    let db = Database::new(&format!("sqlite://{}?mode=rwc", db_path.display()))
        .await
        .expect("Failed to create test database");

    // Run migrations - create all required tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            root_path TEXT,
            description TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            metadata TEXT
        )"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS analyses (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            log_file_path TEXT NOT NULL,
            provider TEXT NOT NULL,
            level TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            started_at DATETIME,
            completed_at DATETIME,
            metadata TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS analysis_results (
            analysis_id TEXT PRIMARY KEY,
            summary TEXT,
            full_report TEXT,
            patterns_detected TEXT,
            issues_found INTEGER,
            metadata TEXT,
            FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
        )"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    (db, temp_dir)
}

#[tokio::test]
async fn test_server_creation() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();

    let server = create_server(db, config).await;
    assert!(server.is_ok(), "Server creation should succeed");
}

#[tokio::test]
async fn test_list_projects_empty() {
    let (db, _temp) = setup_test_db().await;

    // Test tool function directly
    use synapse_mcp::tools::projects::list_projects;
    let result = list_projects(&db, json!({})).await;
    assert!(result.is_ok(), "list_projects should succeed even with no projects");

    let value = result.unwrap();
    assert!(value.is_array(), "Result should be an array");
    assert_eq!(value.as_array().unwrap().len(), 0, "Should have 0 projects");
}

#[tokio::test]
async fn test_list_projects_with_data() {
    let (db, _temp) = setup_test_db().await;

    // Insert test project
    sqlx::query(
        "INSERT INTO projects (id, name, root_path, created_at, updated_at)
         VALUES ('test-id', 'test-project', '/test/path', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    // Test tool function directly
    use synapse_mcp::tools::projects::list_projects;
    let result = list_projects(&db, json!({})).await;
    assert!(result.is_ok(), "list_projects should succeed");

    let value = result.unwrap();
    assert!(value.is_array(), "Result should be an array");
    assert_eq!(value.as_array().unwrap().len(), 1, "Should have 1 project");
}

#[tokio::test]
async fn test_get_project_not_found() {
    let (db, _temp) = setup_test_db().await;

    // Test tool function directly
    use synapse_mcp::tools::projects::get_project;
    let result = get_project(&db, json!({"project_id": "nonexistent"})).await;

    // Should return error
    assert!(result.is_err(), "Should fail for nonexistent project");
}

#[tokio::test]
async fn test_server_config() {
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();

    // Verify server configuration
    assert_eq!(server.config().server_name, "synapse-mcp");
}

#[tokio::test]
async fn test_database_path_resolution() {
    // Test that database path is properly resolved
    use synapse_core::db_path::get_database_path;

    let db_path = get_database_path();
    assert!(db_path.to_str().unwrap().ends_with("synapse.db"));

    // Test with environment variable override
    std::env::set_var("SYNAPSE_DATABASE_PATH", "/custom/test.db");
    let custom_path = get_database_path();
    assert_eq!(custom_path.to_str().unwrap(), "/custom/test.db");
    std::env::remove_var("SYNAPSE_DATABASE_PATH");
}

#[tokio::test]
async fn test_query_analyses_with_filters() {
    let (db, _temp) = setup_test_db().await;

    // Insert test project
    sqlx::query(
        "INSERT INTO projects (id, name, root_path, created_at, updated_at)
         VALUES ('proj-1', 'test-project', '/test', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    // Insert test analyses
    sqlx::query(
        "INSERT INTO analyses (id, project_id, log_file_path, provider, level, status, created_at)
         VALUES
         ('analysis-1', 'proj-1', '/test/log1.log', 'openrouter', 'ERROR', 'completed', CURRENT_TIMESTAMP),
         ('analysis-2', 'proj-1', '/test/log2.log', 'openai', 'WARN', 'pending', CURRENT_TIMESTAMP)"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    // Test querying analyses
    use synapse_core::project::queries::query_analyses;

    let analyses = query_analyses(&db.pool, Some("proj-1"), None, None, None)
        .await
        .unwrap();

    assert_eq!(analyses.len(), 2, "Should find 2 analyses");
}

#[tokio::test]
async fn test_get_analysis_by_id() {
    let (db, _temp) = setup_test_db().await;

    // Insert test project
    sqlx::query(
        "INSERT INTO projects (id, name, root_path, created_at, updated_at)
         VALUES ('proj-1', 'test-project', '/test', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    // Insert test analysis
    sqlx::query(
        "INSERT INTO analyses (id, project_id, log_file_path, provider, level, status, created_at)
         VALUES ('analysis-1', 'proj-1', '/test/log.log', 'openrouter', 'ERROR', 'completed', CURRENT_TIMESTAMP)"
    )
    .execute(&db.pool)
    .await
    .unwrap();

    // Test retrieving analysis
    use synapse_core::project::queries::get_analysis_by_id;

    let result = get_analysis_by_id(&db.pool, "analysis-1")
        .await
        .unwrap();

    assert!(result.is_some(), "Should find the analysis");
    let (analysis, _result) = result.unwrap();
    assert_eq!(analysis.id, "analysis-1");
    assert_eq!(analysis.project_id, "proj-1");
}

#[tokio::test]
async fn test_mcp_schema_definitions() {
    // Test that MCP schemas are properly defined
    use synapse_mcp::schema;

    let list_projects_schema = schema::list_projects_schema();
    assert_eq!(list_projects_schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert!(list_projects_schema.contains_key("properties"));

    let get_project_schema = schema::get_project_schema();
    assert_eq!(get_project_schema.get("type").and_then(|v| v.as_str()), Some("object"));
    assert!(get_project_schema.contains_key("required"));
}
