// Global project registry management
//
// This module manages the global registry of LogLens projects at
// ~/.config/loglens/projects.json, enabling persistent bidirectional
// linkage between software projects and their LogLens configurations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::project::metadata::ProjectMetadata;

/// Global registry entry for a linked project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Project name
    pub name: String,
    /// Absolute path to project root
    pub root_path: PathBuf,
    /// Absolute path to .loglens/ directory
    pub loglens_config: PathBuf,
    /// Last time this project was accessed
    pub last_accessed: DateTime<Utc>,
}

/// Global project registry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    /// Map of project_id -> registry entry
    pub projects: HashMap<String, RegistryEntry>,
}

impl ProjectRegistry {
    /// Get the global registry file path (~/.config/loglens/projects.json)
    pub fn registry_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to determine user config directory")?
            .join("loglens");

        // Ensure config directory exists
        fs::create_dir_all(&config_dir)
            .context("Failed to create LogLens config directory")?;

        Ok(config_dir.join("projects.json"))
    }

    /// Load the global registry from disk
    pub fn load() -> Result<Self> {
        let registry_path = Self::registry_path()?;

        if !registry_path.exists() {
            debug!("Registry file does not exist, creating new registry");
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&registry_path)
            .context("Failed to read registry file")?;

        // Handle empty file
        if contents.trim().is_empty() {
            debug!("Registry file is empty, creating new registry");
            return Ok(Self::default());
        }

        let registry: Self = serde_json::from_str(&contents)
            .context("Failed to parse registry JSON")?;

        debug!("Loaded registry with {} projects", registry.projects.len());
        Ok(registry)
    }

    /// Save the registry to disk
    pub fn save(&self) -> Result<()> {
        let registry_path = Self::registry_path()?;

        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize registry")?;

        fs::write(&registry_path, contents)
            .context("Failed to write registry file")?;

        debug!("Saved registry with {} projects", self.projects.len());
        Ok(())
    }

    /// Register a new project in the global registry
    pub fn register_project(
        &mut self,
        project_id: String,
        name: String,
        root_path: PathBuf,
        loglens_config: PathBuf,
    ) -> Result<()> {
        let entry = RegistryEntry {
            name,
            root_path,
            loglens_config,
            last_accessed: Utc::now(),
        };

        self.projects.insert(project_id.clone(), entry);
        self.save()?;

        info!("Registered project {} in global registry", project_id);
        Ok(())
    }

    /// Unregister a project from the global registry
    pub fn unregister_project(&mut self, project_id: &str) -> Result<bool> {
        if let Some(_) = self.projects.remove(project_id) {
            self.save()?;
            info!("Unregistered project {} from global registry", project_id);
            Ok(true)
        } else {
            warn!("Project {} not found in registry", project_id);
            Ok(false)
        }
    }

    /// Update last accessed timestamp for a project
    pub fn touch_project(&mut self, project_id: &str) -> Result<()> {
        if let Some(entry) = self.projects.get_mut(project_id) {
            entry.last_accessed = Utc::now();
            self.save()?;
            debug!("Updated last_accessed for project {}", project_id);
        }
        Ok(())
    }

    /// Get a project entry by ID
    pub fn get_project(&self, project_id: &str) -> Option<&RegistryEntry> {
        self.projects.get(project_id)
    }

    /// Find project by root path
    pub fn find_by_path(&self, path: &Path) -> Option<(&String, &RegistryEntry)> {
        self.projects
            .iter()
            .find(|(_, entry)| entry.root_path == path)
    }

    /// List all registered projects
    pub fn list_projects(&self) -> Vec<(&String, &RegistryEntry)> {
        let mut projects: Vec<_> = self.projects.iter().collect();
        // Sort by last accessed, most recent first
        projects.sort_by(|a, b| b.1.last_accessed.cmp(&a.1.last_accessed));
        projects
    }

    /// Validate all registry entries and return issues
    pub async fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (project_id, entry) in &self.projects {
            // Check if project root exists
            if !entry.root_path.exists() {
                issues.push(ValidationIssue {
                    project_id: project_id.clone(),
                    issue_type: IssueType::ProjectRootMissing,
                    path: entry.root_path.clone(),
                });
            }

            // Check if .loglens directory exists
            if !entry.loglens_config.exists() {
                issues.push(ValidationIssue {
                    project_id: project_id.clone(),
                    issue_type: IssueType::ConfigDirectoryMissing,
                    path: entry.loglens_config.clone(),
                });
            } else {
                // Verify metadata matches
                let metadata_path = entry.loglens_config.join("metadata.json");
                if let Ok(metadata) = ProjectMetadata::load(&metadata_path).await {
                    if metadata.project_id != *project_id {
                        issues.push(ValidationIssue {
                            project_id: project_id.clone(),
                            issue_type: IssueType::ProjectIdMismatch,
                            path: metadata_path,
                        });
                    }
                } else {
                    issues.push(ValidationIssue {
                        project_id: project_id.clone(),
                        issue_type: IssueType::MetadataInvalid,
                        path: metadata_path,
                    });
                }
            }
        }

        issues
    }

    /// Automatically repair registry issues where possible
    pub async fn auto_repair(&mut self) -> Result<RepairReport> {
        let issues = self.validate().await;
        let mut report = RepairReport::default();

        for issue in issues {
            match issue.issue_type {
                IssueType::ProjectRootMissing | IssueType::ConfigDirectoryMissing => {
                    // Remove stale entries for deleted projects
                    if self.unregister_project(&issue.project_id)? {
                        report.removed.push(issue.project_id.clone());
                        info!("Removed stale project {}", issue.project_id);
                    }
                }
                IssueType::ProjectIdMismatch => {
                    // This requires manual intervention
                    report.manual_intervention.push(issue);
                }
                IssueType::MetadataInvalid => {
                    // Try to remove if metadata is corrupted
                    if self.unregister_project(&issue.project_id)? {
                        report.removed.push(issue.project_id.clone());
                        warn!("Removed project with invalid metadata: {}", issue.project_id);
                    }
                }
            }
        }

        Ok(report)
    }
}

/// Validation issue found in registry
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub project_id: String,
    pub issue_type: IssueType,
    pub path: PathBuf,
}

/// Type of validation issue
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    ProjectRootMissing,
    ConfigDirectoryMissing,
    ProjectIdMismatch,
    MetadataInvalid,
}

/// Report of auto-repair operations
#[derive(Debug, Default)]
pub struct RepairReport {
    /// Projects successfully removed
    pub removed: Vec<String>,
    /// Issues requiring manual intervention
    pub manual_intervention: Vec<ValidationIssue>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_registry_create_and_save() {
        let mut registry = ProjectRegistry::default();
        assert_eq!(registry.projects.len(), 0);

        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        let loglens_config = root_path.join(".loglens");

        registry
            .register_project(
                "test-id".to_string(),
                "test-project".to_string(),
                root_path.clone(),
                loglens_config.clone(),
            )
            .unwrap();

        assert_eq!(registry.projects.len(), 1);
        let entry = registry.get_project("test-id").unwrap();
        assert_eq!(entry.name, "test-project");
        assert_eq!(entry.root_path, root_path);
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = ProjectRegistry::default();
        let temp_dir = TempDir::new().unwrap();

        registry
            .register_project(
                "test-id".to_string(),
                "test-project".to_string(),
                temp_dir.path().to_path_buf(),
                temp_dir.path().join(".loglens"),
            )
            .unwrap();

        assert!(registry.unregister_project("test-id").unwrap());
        assert!(!registry.unregister_project("test-id").unwrap());
        assert_eq!(registry.projects.len(), 0);
    }

    #[test]
    fn test_find_by_path() {
        let mut registry = ProjectRegistry::default();
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();

        registry
            .register_project(
                "test-id".to_string(),
                "test-project".to_string(),
                root_path.clone(),
                root_path.join(".loglens"),
            )
            .unwrap();

        let (id, entry) = registry.find_by_path(&root_path).unwrap();
        assert_eq!(id, "test-id");
        assert_eq!(entry.name, "test-project");
    }

    #[tokio::test]
    async fn test_validation_missing_paths() {
        let mut registry = ProjectRegistry::default();
        let nonexistent = PathBuf::from("/nonexistent/path");

        registry
            .register_project(
                "test-id".to_string(),
                "test-project".to_string(),
                nonexistent.clone(),
                nonexistent.join(".loglens"),
            )
            .unwrap();

        let issues = registry.validate().await;
        assert_eq!(issues.len(), 2); // Both root and config missing

        let has_root_missing = issues
            .iter()
            .any(|i| i.issue_type == IssueType::ProjectRootMissing);
        let has_config_missing = issues
            .iter()
            .any(|i| i.issue_type == IssueType::ConfigDirectoryMissing);

        assert!(has_root_missing);
        assert!(has_config_missing);
    }

    #[tokio::test]
    async fn test_auto_repair_removes_missing() {
        let mut registry = ProjectRegistry::default();
        let nonexistent = PathBuf::from("/nonexistent/path");

        registry
            .register_project(
                "test-id".to_string(),
                "test-project".to_string(),
                nonexistent.clone(),
                nonexistent.join(".loglens"),
            )
            .unwrap();

        let report = registry.auto_repair().await.unwrap();
        assert_eq!(report.removed.len(), 1);
        assert_eq!(report.removed[0], "test-id");
        assert_eq!(registry.projects.len(), 0);
    }
}
