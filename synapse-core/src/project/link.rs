// Project linking and unlinking operations
//
// This module handles creating and removing bidirectional links between
// software projects and their Synapse configurations in the global registry.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::project::metadata::ProjectMetadata;
use crate::project::registry::ProjectRegistry;

/// Link an existing Synapse project to the global registry
///
/// This creates a bidirectional reference:
/// - Project's .synapse/metadata.json contains project_id
/// - Global registry maps project_id -> project paths
pub async fn link_project(project_path: Option<&Path>) -> Result<LinkResult> {
    let project_path = project_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let project_path = project_path
        .canonicalize()
        .context("Failed to resolve project path")?;

    info!("Linking project at {}", project_path.display());

    // Verify .synapse directory exists
    let synapse_dir = project_path.join(".synapse");
    if !synapse_dir.exists() {
        bail!(
            "No .synapse directory found at {}. Run 'synapse init' first.",
            project_path.display()
        );
    }

    // Load project metadata
    let metadata_path = synapse_dir.join("metadata.json");
    let metadata = ProjectMetadata::load(&metadata_path).await
        .context("Failed to load project metadata")?;

    // Load global registry
    let mut registry = ProjectRegistry::load()
        .context("Failed to load global registry")?;

    // Check if already linked
    if let Some(existing) = registry.get_project(&metadata.project_id) {
        if existing.root_path == project_path {
            info!("Project already linked");
            return Ok(LinkResult {
                project_id: metadata.project_id,
                project_name: metadata.project_name,
                root_path: project_path,
                already_linked: true,
            });
        } else {
            bail!(
                "Project ID {} is already linked to a different path: {}",
                metadata.project_id,
                existing.root_path.display()
            );
        }
    }

    // Check if path is already linked under different ID
    if let Some((existing_id, _)) = registry.find_by_path(&project_path) {
        bail!(
            "This path is already linked with project ID: {}",
            existing_id
        );
    }

    // Register in global registry (JSON file for CLI)
    registry.register_project(
        metadata.project_id.clone(),
        metadata.project_name.clone(),
        project_path.clone(),
        synapse_dir.clone(),
    )?;

    // ALSO register in SQLite database for web dashboard visibility
    #[cfg(feature = "project-management")]
    if let Err(e) = super::database::register_in_database(&metadata, &project_path, &synapse_dir).await {
        // Log warning but don't fail - CLI users may not have web database yet
        debug!("Failed to register project in web database: {}", e);
    }

    info!(
        "Successfully linked project {} ({})",
        metadata.project_name, metadata.project_id
    );

    Ok(LinkResult {
        project_id: metadata.project_id,
        project_name: metadata.project_name,
        root_path: project_path,
        already_linked: false,
    })
}

/// Unlink a project from the global registry
///
/// This removes the registry entry but preserves the .synapse/ directory
/// and all project data.
pub async fn unlink_project(project_path: Option<&Path>) -> Result<UnlinkResult> {
    let project_path = project_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let project_path = project_path
        .canonicalize()
        .context("Failed to resolve project path")?;

    info!("Unlinking project at {}", project_path.display());

    // Load project metadata to get project_id
    let synapse_dir = project_path.join(".synapse");
    let metadata_path = synapse_dir.join("metadata.json");

    if !metadata_path.exists() {
        bail!(
            "No Synapse metadata found at {}. Project may not be initialized.",
            project_path.display()
        );
    }

    let metadata = ProjectMetadata::load(&metadata_path).await
        .context("Failed to load project metadata")?;

    // Load global registry
    let mut registry = ProjectRegistry::load()
        .context("Failed to load global registry")?;

    // Unregister from global registry (JSON file)
    let was_linked = registry.unregister_project(&metadata.project_id)?;

    if !was_linked {
        debug!("Project was not linked in registry");
    }

    // ALSO remove from SQLite database
    #[cfg(feature = "project-management")]
    if let Err(e) = super::database::unregister_from_database(&metadata.project_id).await {
        debug!("Failed to unregister project from web database: {}", e);
    }

    info!(
        "Successfully unlinked project {} ({})",
        metadata.project_name, metadata.project_id
    );

    Ok(UnlinkResult {
        project_id: metadata.project_id,
        project_name: metadata.project_name,
        root_path: project_path,
        was_linked,
    })
}

/// Result of link operation
#[derive(Debug)]
pub struct LinkResult {
    pub project_id: String,
    pub project_name: String,
    pub root_path: PathBuf,
    pub already_linked: bool,
}

/// Result of unlink operation
#[derive(Debug)]
pub struct UnlinkResult {
    pub project_id: String,
    pub project_name: String,
    pub root_path: PathBuf,
    pub was_linked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::initialize_project;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_link_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Initialize project first (this now auto-links)
        let init_result = initialize_project(Some(project_path))
            .await
            .unwrap();

        // Linking again should report already linked
        let result = link_project(Some(project_path)).await.unwrap();
        assert!(result.already_linked); // Changed: init now auto-links
        assert_eq!(result.root_path, project_path.canonicalize().unwrap());
        assert_eq!(result.project_id, init_result.project_id);

        // Verify it's in the registry
        let registry = ProjectRegistry::load().unwrap();
        assert!(registry.get_project(&result.project_id).is_some());

        // Cleanup
        let mut registry = ProjectRegistry::load().unwrap();
        registry.unregister_project(&result.project_id).unwrap();
    }

    #[tokio::test]
    async fn test_link_already_linked() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let init_result = initialize_project(Some(project_path))
            .await
            .unwrap();

        // Link after init (already linked via init)
        let result1 = link_project(Some(project_path)).await.unwrap();
        let result2 = link_project(Some(project_path)).await.unwrap();

        assert!(result1.already_linked); // Changed: init auto-links
        assert!(result2.already_linked);
        assert_eq!(result1.project_id, result2.project_id);
        assert_eq!(result1.project_id, init_result.project_id);

        // Cleanup
        let mut registry = ProjectRegistry::load().unwrap();
        registry.unregister_project(&init_result.project_id).unwrap();
    }

    #[tokio::test]
    async fn test_link_without_init() {
        let temp_dir = TempDir::new().unwrap();
        let result = link_project(Some(temp_dir.path())).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Run 'synapse init' first"));
    }

    #[tokio::test]
    async fn test_unlink_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let init_result = initialize_project(Some(project_path))
            .await
            .unwrap();

        // Unlink the project (it was auto-linked during init)
        let unlink_result = unlink_project(Some(project_path)).await.unwrap();
        assert!(unlink_result.was_linked);
        assert_eq!(unlink_result.project_id, init_result.project_id);

        // Verify it's removed from registry
        let registry = ProjectRegistry::load().unwrap();
        assert!(registry.get_project(&init_result.project_id).is_none());

        // Verify .synapse directory still exists
        assert!(project_path.join(".synapse").exists());
    }

    #[tokio::test]
    async fn test_unlink_not_linked() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let init_result = initialize_project(Some(project_path))
            .await
            .unwrap();

        // First unlink (it was linked during init)
        let result1 = unlink_project(Some(project_path)).await.unwrap();
        assert!(result1.was_linked);

        // Second unlink should show not linked
        let result2 = unlink_project(Some(project_path)).await.unwrap();
        assert!(!result2.was_linked);
        assert_eq!(result2.project_id, init_result.project_id);
    }
}
