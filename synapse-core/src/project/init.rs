// Project initialization logic for Synapse

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use super::config::ProjectConfig;
use super::detect::{detect_project_type, ProjectType};
use super::metadata::ProjectMetadata;
use super::registry::ProjectRegistry;

#[cfg(feature = "project-management")]
use super::database::initialize_database;

const SYNAPSE_DIR: &str = ".synapse";
const CONFIG_FILE: &str = "config.toml";
const METADATA_FILE: &str = "metadata.json";
const DATABASE_FILE: &str = "index.db";
const ANALYSES_DIR: &str = "analyses";
const LOGS_DIR: &str = "logs";

/// Result of project initialization
#[derive(Debug)]
pub struct InitializationResult {
    pub project_path: PathBuf,
    pub synapse_dir: PathBuf,
    pub project_type: ProjectType,
    pub project_id: String,
}

/// Initialize Synapse in a project directory
///
/// Creates .synapse/ directory with:
/// - config.toml (project configuration)
/// - metadata.json (project metadata with UUID)
/// - index.db (SQLite database)
/// - analyses/ (analysis results storage)
/// - logs/ (optional log file cache)
pub async fn initialize_project<P: AsRef<Path>>(project_path: Option<P>) -> Result<InitializationResult> {
    // Resolve project path (default to current directory)
    let project_path = if let Some(path) = project_path {
        path.as_ref().canonicalize()
            .context("Failed to resolve project path")?
    } else {
        std::env::current_dir()
            .context("Failed to get current directory")?
    };

    info!("Initializing Synapse in: {:?}", project_path);

    // Detect project type
    let project_type = detect_project_type(&project_path).await?;
    info!("Detected project type: {}", project_type);

    if project_type == ProjectType::Unknown {
        warn!("Could not detect project type - using 'unknown'");
    }

    // Create .synapse directory
    let synapse_dir = project_path.join(SYNAPSE_DIR);
    if synapse_dir.exists() {
        anyhow::bail!(
            "Synapse directory already exists at: {:?}\nProject may already be initialized.",
            synapse_dir
        );
    }

    tokio::fs::create_dir(&synapse_dir)
        .await
        .context("Failed to create .synapse directory")?;
    debug!("Created directory: {:?}", synapse_dir);

    // Create subdirectories
    let analyses_dir = synapse_dir.join(ANALYSES_DIR);
    let logs_dir = synapse_dir.join(LOGS_DIR);

    tokio::fs::create_dir(&analyses_dir).await
        .context("Failed to create analyses directory")?;
    tokio::fs::create_dir(&logs_dir).await
        .context("Failed to create logs directory")?;
    debug!("Created subdirectories: analyses/, logs/");

    // Generate project name from directory name
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project")
        .to_string();

    // Get absolute path as string
    let root_path = project_path
        .to_str()
        .context("Project path contains invalid UTF-8")?
        .to_string();

    // Create metadata
    let git_remote = detect_git_remote(&project_path).await;
    let metadata = ProjectMetadata::new(
        project_name.clone(),
        project_type.to_string(),
        root_path.clone(),
    )
    .with_git_remote(git_remote);

    let project_id = metadata.project_id.clone();

    // Save metadata.json
    let metadata_path = synapse_dir.join(METADATA_FILE);
    metadata.save(&metadata_path).await
        .context("Failed to save metadata.json")?;
    info!("Created metadata.json");

    // Create configuration
    let config = ProjectConfig::new(
        project_name,
        project_type.to_string(),
        root_path,
    );

    // Save config.toml
    let config_path = synapse_dir.join(CONFIG_FILE);
    config.save(&config_path).await
        .context("Failed to save config.toml")?;
    info!("Created config.toml");

    // Initialize database
    #[cfg(feature = "project-management")]
    {
        let db_path = synapse_dir.join(DATABASE_FILE);
        initialize_database(&db_path).await
            .context("Failed to initialize database")?;
        info!("Initialized database at index.db");
    }

    // Register in global registry
    let mut registry = ProjectRegistry::load()
        .context("Failed to load global registry")?;
    registry.register_project(
        project_id.clone(),
        metadata.project_name.clone(),
        project_path.clone(),
        synapse_dir.clone(),
    )
    .context("Failed to register project in global registry")?;
    info!("Registered project in global registry");

    info!("Synapse initialization complete!");
    info!("Project ID: {}", project_id);

    Ok(InitializationResult {
        project_path,
        synapse_dir,
        project_type,
        project_id,
    })
}

/// Attempt to detect git remote URL
async fn detect_git_remote<P: AsRef<Path>>(project_path: P) -> Option<String> {
    let git_config = project_path.as_ref().join(".git").join("config");

    if !git_config.exists() {
        return None;
    }

    // Read git config and extract remote URL
    if let Ok(content) = tokio::fs::read_to_string(&git_config).await {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("url = ") {
                return Some(trimmed.strip_prefix("url = ")?.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_initialize_rust_project() {
        let temp_dir = tempdir().unwrap();
        // Create Cargo.toml to mark as Rust project
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"")
            .await
            .unwrap();

        let result = initialize_project(Some(temp_dir.path())).await.unwrap();

        assert_eq!(result.project_type, ProjectType::Rust);
        assert!(result.synapse_dir.exists());
        assert!(result.synapse_dir.join("config.toml").exists());
        assert!(result.synapse_dir.join("metadata.json").exists());
        assert!(result.synapse_dir.join("analyses").exists());
        assert!(result.synapse_dir.join("logs").exists());

        #[cfg(feature = "project-management")]
        assert!(result.synapse_dir.join("index.db").exists());
    }

    #[tokio::test]
    async fn test_initialize_python_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("requirements.txt"), "")
            .await
            .unwrap();

        let result = initialize_project(Some(temp_dir.path())).await.unwrap();

        assert_eq!(result.project_type, ProjectType::Python);
        assert!(!result.project_id.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_already_initialized() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "")
            .await
            .unwrap();

        // First initialization should succeed
        initialize_project(Some(temp_dir.path())).await.unwrap();

        // Second initialization should fail
        let result = initialize_project(Some(temp_dir.path())).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_config_toml_generated() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}")
            .await
            .unwrap();

        let result = initialize_project(Some(temp_dir.path())).await.unwrap();

        let config_path = result.synapse_dir.join("config.toml");
        let config_content = fs::read_to_string(&config_path).await.unwrap();

        assert!(config_content.contains("[project]"));
        assert!(config_content.contains("type = \"node\""));
        assert!(config_content.contains("[synapse]"));
        assert!(config_content.contains("auto_analyze = true"));
    }

    #[tokio::test]
    async fn test_metadata_json_generated() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("pom.xml"), "")
            .await
            .unwrap();

        let result = initialize_project(Some(temp_dir.path())).await.unwrap();

        let metadata_path = result.synapse_dir.join("metadata.json");
        let metadata_content = fs::read_to_string(&metadata_path).await.unwrap();

        assert!(metadata_content.contains("project_id"));
        assert!(metadata_content.contains("project_type"));
        assert!(metadata_content.contains("java"));
        assert!(metadata_content.contains("synapse_version"));
    }

    #[tokio::test]
    async fn test_detect_git_remote_no_git() {
        let temp_dir = tempdir().unwrap();
        let remote = detect_git_remote(temp_dir.path()).await;
        assert!(remote.is_none());
    }

    #[tokio::test]
    async fn test_detect_git_remote_with_git() {
        let temp_dir = tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).await.unwrap();

        let git_config = git_dir.join("config");
        fs::write(
            &git_config,
            "[remote \"origin\"]\n\turl = https://github.com/user/repo.git\n",
        )
        .await
        .unwrap();

        let remote = detect_git_remote(temp_dir.path()).await;
        assert_eq!(remote, Some("https://github.com/user/repo.git".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "project-management")]
    async fn test_database_initialized() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "")
            .await
            .unwrap();

        let result = initialize_project(Some(temp_dir.path())).await.unwrap();

        let db_path = result.synapse_dir.join("index.db");
        assert!(db_path.exists());

        // Verify database can be opened and has correct schema
        use sqlx::sqlite::SqlitePool;
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();

        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
        assert!(table_names.contains(&"projects".to_string()));
        assert!(table_names.contains(&"analyses".to_string()));
        assert!(table_names.contains(&"analysis_results".to_string()));

        // Cleanup
        let mut registry = crate::project::registry::ProjectRegistry::load().unwrap();
        registry.unregister_project(&result.project_id).unwrap();
    }
}
