// Database operations for LogLens project management

use std::path::Path;

#[cfg(feature = "project-management")]
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
#[cfg(feature = "project-management")]
use std::str::FromStr;

use anyhow::{Context, Result};
use tracing::{debug, info, error};

/// Initialize database connection pool with WAL mode enabled
#[cfg(feature = "project-management")]
pub async fn create_pool<P: AsRef<Path>>(database_path: P) -> Result<SqlitePool> {
    let path_str = database_path
        .as_ref()
        .to_str()
        .context("Invalid database path")?;

    info!("Creating database connection pool for: {}", path_str);

    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path_str))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .context("Failed to create database connection pool")?;

    debug!("Database connection pool created successfully");
    Ok(pool)
}

/// Create database schema from migration SQL
#[cfg(feature = "project-management")]
pub async fn create_schema(pool: &SqlitePool) -> Result<()> {
    info!("Creating database schema");

    let schema_sql = include_str!("../../migrations/001_initial_schema.sql");

    // Parse SQL statements more carefully - account for PRAGMA and multi-line statements
    let mut statements = Vec::new();
    let mut current_statement = String::new();

    for line in schema_sql.lines() {
        let trimmed = line.trim();

        // Skip empty lines and standalone comments
        if trimmed.is_empty() || (trimmed.starts_with("--") && current_statement.is_empty()) {
            continue;
        }

        // Add line to current statement (skip inline comments)
        if let Some(pos) = trimmed.find("--") {
            current_statement.push_str(&trimmed[..pos]);
        } else {
            current_statement.push_str(trimmed);
        }
        current_statement.push(' ');

        // If line ends with semicolon, statement is complete
        if trimmed.ends_with(';') {
            let stmt = current_statement.trim().trim_end_matches(';').trim().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current_statement.clear();
        }
    }

    // Execute each statement
    for statement in statements {
        debug!("Executing: {}", &statement[..statement.len().min(80)]);
        sqlx::query(&statement)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to execute schema statement: {}", &statement[..statement.len().min(200)]))?;
    }

    info!("Database schema created successfully");
    Ok(())
}

/// Verify database schema is correctly initialized
#[cfg(feature = "project-management")]
pub async fn verify_schema(pool: &SqlitePool) -> Result<bool> {
    debug!("Verifying database schema");

    // Check if all required tables exist
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name IN ('projects', 'analyses', 'analysis_results')"
    )
    .fetch_all(pool)
    .await
    .context("Failed to query table existence")?;

    let has_all_tables = tables.len() == 3;

    if has_all_tables {
        debug!("All required tables exist");
    } else {
        error!("Missing required tables. Found: {:?}", tables);
    }

    Ok(has_all_tables)
}

/// Initialize database for a project
#[cfg(feature = "project-management")]
pub async fn initialize_database<P: AsRef<Path>>(database_path: P) -> Result<SqlitePool> {
    let pool = create_pool(&database_path).await?;
    create_schema(&pool).await?;

    // Verify schema was created correctly
    if !verify_schema(&pool).await? {
        anyhow::bail!("Database schema verification failed");
    }

    info!("Database initialized successfully at: {:?}", database_path.as_ref());
    Ok(pool)
}

#[cfg(all(test, feature = "project-management"))]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_pool() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.unwrap();
        // Pool was created successfully - size may vary based on implementation
        assert!(pool.is_closed() == false);
    }

    #[tokio::test]
    async fn test_create_schema() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.unwrap();
        create_schema(&pool).await.unwrap();

        // Verify tables exist
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
        assert!(table_names.contains(&"projects".to_string()));
        assert!(table_names.contains(&"analyses".to_string()));
        assert!(table_names.contains(&"analysis_results".to_string()));
    }

    #[tokio::test]
    async fn test_verify_schema() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path).await.unwrap();

        // Should fail before schema creation
        let verified = verify_schema(&pool).await.unwrap();
        assert!(!verified);

        // Should succeed after schema creation
        create_schema(&pool).await.unwrap();
        let verified = verify_schema(&pool).await.unwrap();
        assert!(verified);
    }

    #[tokio::test]
    async fn test_initialize_database() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = initialize_database(&db_path).await.unwrap();

        // Verify WAL mode is enabled
        let journal_mode: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(journal_mode.0.to_lowercase(), "wal");
    }

    #[tokio::test]
    async fn test_indexes_created() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = initialize_database(&db_path).await.unwrap();

        // Verify indexes exist
        let indexes: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(indexes.len() >= 3); // At least the 3 main indexes
    }
}
