use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub database_url: String,
    pub max_upload_size: usize,
    pub max_projects: usize,
    pub analysis_timeout_secs: u64,
    pub cors_origins: Vec<String>,
    pub frontend_dir: String,
    pub upload_dir: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        // Use unified database path from loglens-core
        let db_path = loglens_core::db_path::get_database_path();
        let db_path_str = db_path.to_string_lossy().to_string();

        Self {
            port: 3000,
            database_url: format!("sqlite://{}", db_path_str),
            max_upload_size: 50 * 1024 * 1024, // 50MB
            max_projects: 100,
            analysis_timeout_secs: 300, // 5 minutes
            cors_origins: vec!["http://localhost:3000".to_string()],
            // Frontend directory - check multiple locations
            frontend_dir: {
                // Check if running from installed binary
                if let Ok(exe_path) = std::env::current_exe() {
                    let install_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
                    let frontend_path = install_dir.join("frontend");
                    if frontend_path.exists() {
                        frontend_path.to_string_lossy().to_string()
                    } else {
                        "loglens-web/frontend-react/dist".to_string()
                    }
                } else {
                    "loglens-web/frontend-react/dist".to_string()
                }
            },
            upload_dir: "./uploads".to_string(),
        }
    }
}

impl WebConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        // Load from environment variables
        if let Ok(port) = env::var("LOGLENS_PORT").or_else(|_| env::var("PORT")) {
            config.port = port.parse()?;
        }

        // Allow override with DATABASE_URL or LOGLENS_DATABASE_PATH if needed
        if let Ok(database_url) = env::var("DATABASE_URL") {
            config.database_url = database_url;
        } else if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
            config.database_url = format!("sqlite://{}", db_path);
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

        if let Ok(frontend_dir) = env::var("LOGLENS_FRONTEND_DIR") {
            config.frontend_dir = frontend_dir;
        }

        if let Ok(upload_dir) = env::var("LOGLENS_UPLOAD_DIR") {
            config.upload_dir = upload_dir;
        }

        Ok(config)
    }
}
