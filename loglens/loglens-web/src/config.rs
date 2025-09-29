use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub database_url: String,
    pub max_upload_size: usize,
    pub max_projects: usize,
    pub analysis_timeout_secs: u64,
    pub cors_origins: Vec<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 3001,
            database_url: "sqlite://./loglens.db".to_string(),
            max_upload_size: 50 * 1024 * 1024, // 50MB
            max_projects: 100,
            analysis_timeout_secs: 300, // 5 minutes
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }
}

impl WebConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        // Load from environment variables
        if let Ok(port) = env::var("LOGLENS_PORT") {
            config.port = port.parse()?;
        }

        if let Ok(db_url) = env::var("LOGLENS_DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(max_size) = env::var("LOGLENS_MAX_UPLOAD_SIZE") {
            config.max_upload_size = max_size.parse()?;
        }

        if let Ok(max_projects) = env::var("LOGLENS_MAX_PROJECTS") {
            config.max_projects = max_projects.parse()?;
        }

        if let Ok(timeout) = env::var("LOGLENS_ANALYSIS_TIMEOUT") {
            config.analysis_timeout_secs = timeout.parse()?;
        }

        if let Ok(origins) = env::var("LOGLENS_CORS_ORIGINS") {
            config.cors_origins = origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        Ok(config)
    }
}
