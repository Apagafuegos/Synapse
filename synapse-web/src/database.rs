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

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
