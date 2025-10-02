use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite};

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(db_path) = database_url.strip_prefix("sqlite://") {
            if let Some(parent) = std::path::Path::new(db_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Create database if it doesn't exist
        if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
            println!("Creating database at {}", database_url);
            Sqlite::create_database(database_url).await?;
        }

        // Configure connection pool with proper limits
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .max_lifetime(Some(std::time::Duration::from_secs(30 * 60))) // 30 minutes
            .idle_timeout(Some(std::time::Duration::from_secs(10 * 60))) // 10 minutes
            .acquire_timeout(std::time::Duration::from_secs(30)) // 30 seconds
            .test_before_acquire(true)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        // Run migrations from the migrations folder
        sqlx::migrate!("./migrations").run(self.pool()).await?;
        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        let pool = self.pool();
        
        // Create projects table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"
        ).execute(pool).await?;

        // Create log_files table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS log_files (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                line_count INTEGER NOT NULL,
                upload_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            )"
        ).execute(pool).await?;

        // Create analyses table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS analyses (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                log_file_id TEXT,
                analysis_type TEXT NOT NULL,
                provider TEXT NOT NULL,
                level_filter TEXT NOT NULL,
                status INTEGER NOT NULL DEFAULT 0,
                result TEXT,
                error_message TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                FOREIGN KEY (project_id) REFERENCES projects(id),
                FOREIGN KEY (log_file_id) REFERENCES log_files(id)
            )"
        ).execute(pool).await?;

        tracing::info!("Database tables created successfully");
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
