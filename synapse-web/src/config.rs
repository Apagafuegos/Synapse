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
        // Use unified database path from synapse-core
        let db_path = synapse_core::db_path::get_database_path();
        let db_path_str = db_path.to_string_lossy().to_string();

        Self {
            port: 3000,
            database_url: format!("sqlite://{}", db_path_str),
            max_upload_size: 50 * 1024 * 1024, // 50MB
            max_projects: 100,
            analysis_timeout_secs: 300, // 5 minutes
            cors_origins: vec!["http://localhost:3000".to_string()],
            // Frontend directory - check multiple locations
            // CRITICAL: Always use absolute paths for Windows compatibility
            frontend_dir: {
                if let Ok(exe_path) = std::env::current_exe() {
                    let install_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
                    let frontend_path = install_dir.join("frontend");

                    // 1. Check if frontend exists next to executable (installed location)
                    if frontend_path.exists() && frontend_path.join("index.html").exists() {
                        frontend_path.canonicalize()
                            .unwrap_or(frontend_path)
                            .to_string_lossy()
                            .to_string()
                    }
                    // 2. Check if running from target/release - go up to workspace root
                    else if install_dir.ends_with("target/release") || install_dir.ends_with("target\\release") {
                        // Go up two levels: target/release -> target -> workspace root
                        if let Some(target_dir) = install_dir.parent() {
                            if let Some(workspace_root) = target_dir.parent() {
                                let workspace_frontend = workspace_root.join("synapse-web/frontend-react/dist");
                                if workspace_frontend.exists() && workspace_frontend.join("index.html").exists() {
                                    workspace_frontend.canonicalize()
                                        .unwrap_or(workspace_frontend)
                                        .to_string_lossy()
                                        .to_string()
                                } else {
                                    // Fallback to install path
                                    frontend_path.to_string_lossy().to_string()
                                }
                            } else {
                                // Fallback to install path
                                frontend_path.to_string_lossy().to_string()
                            }
                        } else {
                            // Fallback to install path
                            frontend_path.to_string_lossy().to_string()
                        }
                    }
                    // 3. Check development path relative to current working directory
                    else if Path::new("synapse-web/frontend-react/dist/index.html").exists() {
                        Path::new("synapse-web/frontend-react/dist")
                            .canonicalize()
                            .unwrap_or_else(|_| Path::new("synapse-web/frontend-react/dist").to_path_buf())
                            .to_string_lossy()
                            .to_string()
                    }
                    // 4. Check if we're in the synapse-web directory directly
                    else if Path::new("frontend-react/dist/index.html").exists() {
                        Path::new("frontend-react/dist")
                            .canonicalize()
                            .unwrap_or_else(|_| Path::new("frontend-react/dist").to_path_buf())
                            .to_string_lossy()
                            .to_string()
                    }
                    // Last resort: return the expected install path
                    else {
                        frontend_path.to_string_lossy().to_string()
                    }
                } else {
                    // Fallback to development path
                    "synapse-web/frontend-react/dist".to_string()
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
        if let Ok(port) = env::var("SYNAPSE_PORT").or_else(|_| env::var("PORT")) {
            config.port = port.parse()?;
        }

        // Allow override with DATABASE_URL or SYNAPSE_DATABASE_PATH if needed
        if let Ok(database_url) = env::var("DATABASE_URL") {
            config.database_url = database_url;
        } else if let Ok(db_path) = env::var("SYNAPSE_DATABASE_PATH") {
            config.database_url = format!("sqlite://{}", db_path);
        }

        if let Ok(max_size) = env::var("SYNAPSE_MAX_UPLOAD_SIZE") {
            config.max_upload_size = max_size.parse()?;
        }

        if let Ok(max_projects) = env::var("SYNAPSE_MAX_PROJECTS") {
            config.max_projects = max_projects.parse()?;
        }

        if let Ok(timeout) = env::var("SYNAPSE_ANALYSIS_TIMEOUT") {
            config.analysis_timeout_secs = timeout.parse()?;
        }

        if let Ok(origins) = env::var("SYNAPSE_CORS_ORIGINS") {
            config.cors_origins = origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(frontend_dir) = env::var("SYNAPSE_FRONTEND_DIR") {
            config.frontend_dir = frontend_dir;
        }

        if let Ok(upload_dir) = env::var("SYNAPSE_UPLOAD_DIR") {
            config.upload_dir = upload_dir;
        }

        Ok(config)
    }
}
