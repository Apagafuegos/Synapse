// Project link validation and auto-repair
//
// This module provides validation logic for the hard link system,
// ensuring consistency between the global registry and project metadata,
// with automatic repair capabilities for common issues.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tracing::info;

use crate::project::metadata::ProjectMetadata;
use crate::project::registry::{IssueType, ProjectRegistry, RepairReport, ValidationIssue};

/// Comprehensive validation report
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// Total projects in registry
    pub total_projects: usize,
    /// Projects that passed validation
    pub valid_projects: usize,
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
    /// Timestamp of validation
    pub timestamp: DateTime<Utc>,
}

/// Validate all links in the global registry
pub async fn validate_links() -> Result<ValidationReport> {
    info!("Starting link validation");

    let registry = ProjectRegistry::load()
        .context("Failed to load global registry")?;

    let issues = registry.validate().await;
    let total_projects = registry.projects.len();
    let valid_projects = total_projects - count_unique_projects(&issues);

    let report = ValidationReport {
        total_projects,
        valid_projects,
        issues,
        timestamp: Utc::now(),
    };

    info!(
        "Validation complete: {}/{} projects valid, {} issues found",
        report.valid_projects,
        report.total_projects,
        report.issues.len()
    );

    Ok(report)
}

/// Validate links and automatically repair issues where possible
pub async fn validate_and_repair() -> Result<(ValidationReport, RepairReport)> {
    info!("Starting validation and auto-repair");

    let mut registry = ProjectRegistry::load()
        .context("Failed to load global registry")?;

    // First validation pass
    let total_projects = registry.projects.len();

    // Auto-repair
    let repair_report = registry.auto_repair().await
        .context("Auto-repair failed")?;

    // Second validation pass after repair
    let remaining_issues = registry.validate().await;
    let valid_projects = registry.projects.len() - count_unique_projects(&remaining_issues);

    let validation_report = ValidationReport {
        total_projects,
        valid_projects,
        issues: remaining_issues,
        timestamp: Utc::now(),
    };

    info!(
        "Repair complete: removed {} projects, {} issues remain requiring manual intervention",
        repair_report.removed.len(),
        repair_report.manual_intervention.len()
    );

    Ok((validation_report, repair_report))
}

/// Validate a specific project
pub async fn validate_project(project_path: &PathBuf) -> Result<ProjectValidation> {
    let project_path = project_path
        .canonicalize()
        .context("Failed to resolve project path")?;

    let mut validation = ProjectValidation {
        project_path: project_path.clone(),
        ..Default::default()
    };

    // Check if .loglens exists
    let loglens_dir = project_path.join(".loglens");
    if !loglens_dir.exists() {
        validation.loglens_exists = false;
        validation.errors.push("No .loglens directory found".to_string());
        return Ok(validation);
    }
    validation.loglens_exists = true;

    // Check metadata
    let metadata_path = loglens_dir.join("metadata.json");
    if !metadata_path.exists() {
        validation.metadata_valid = false;
        validation.errors.push("No metadata.json found".to_string());
        return Ok(validation);
    }

    match ProjectMetadata::load(&metadata_path).await {
        Ok(metadata) => {
            validation.metadata_valid = true;
            validation.project_id = Some(metadata.project_id.clone());

            // Check registry link
            let registry = ProjectRegistry::load()
                .context("Failed to load global registry")?;

            if let Some(entry) = registry.get_project(&metadata.project_id) {
                validation.registry_linked = true;

                // Verify bidirectional consistency
                if entry.root_path != project_path {
                    validation.bidirectional_valid = false;
                    validation.warnings.push(format!(
                        "Registry path mismatch: registry has {}, project is at {}",
                        entry.root_path.display(),
                        project_path.display()
                    ));
                } else {
                    validation.bidirectional_valid = true;
                }
            } else {
                validation.registry_linked = false;
                validation.warnings.push("Project not linked in global registry".to_string());
            }
        }
        Err(e) => {
            validation.metadata_valid = false;
            validation.errors.push(format!("Invalid metadata: {}", e));
        }
    }

    Ok(validation)
}

/// Validation result for a specific project
#[derive(Debug, Default)]
pub struct ProjectValidation {
    pub project_path: PathBuf,
    pub loglens_exists: bool,
    pub metadata_valid: bool,
    pub registry_linked: bool,
    pub bidirectional_valid: bool,
    pub project_id: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ProjectValidation {
    /// Check if project is fully valid
    pub fn is_valid(&self) -> bool {
        self.loglens_exists
            && self.metadata_valid
            && self.registry_linked
            && self.bidirectional_valid
            && self.errors.is_empty()
    }
}

/// Count unique projects that have issues
fn count_unique_projects(issues: &[ValidationIssue]) -> usize {
    use std::collections::HashSet;
    let mut project_ids = HashSet::new();
    for issue in issues {
        project_ids.insert(&issue.project_id);
    }
    project_ids.len()
}

/// Format issue type as human-readable string
pub fn format_issue_type(issue_type: &IssueType) -> &'static str {
    match issue_type {
        IssueType::ProjectRootMissing => "Project root directory not found",
        IssueType::ConfigDirectoryMissing => ".loglens directory not found",
        IssueType::ProjectIdMismatch => "Project ID does not match metadata",
        IssueType::MetadataInvalid => "Metadata file is invalid or corrupted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::initialize_project;
    use crate::project::link::link_project;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_valid_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Initialize and link
        initialize_project(Some(&project_path))
            .await
            .unwrap();
        link_project(Some(&project_path))
            .await
            .unwrap();

        // Validate
        let validation = validate_project(&project_path).await.unwrap();
        assert!(validation.is_valid());
        assert!(validation.loglens_exists);
        assert!(validation.metadata_valid);
        assert!(validation.registry_linked);
        assert!(validation.bidirectional_valid);
        assert_eq!(validation.errors.len(), 0);
    }

    #[tokio::test]
    async fn test_validate_missing_loglens() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let validation = validate_project(&project_path).await.unwrap();
        assert!(!validation.is_valid());
        assert!(!validation.loglens_exists);
        assert_eq!(validation.errors.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_not_linked() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Initialize (auto-links), then unlink
        let init_result = initialize_project(Some(&project_path))
            .await
            .unwrap();

        // Unlink to test validation of unlinked project
        let mut registry = ProjectRegistry::load().unwrap();
        registry.unregister_project(&init_result.project_id).unwrap();

        let validation = validate_project(&project_path).await.unwrap();
        assert!(!validation.is_valid());
        assert!(validation.loglens_exists);
        assert!(validation.metadata_valid);
        assert!(!validation.registry_linked);
        assert_eq!(validation.warnings.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_links_with_issues() {
        // Create a project with missing directory
        let mut registry = ProjectRegistry::default();
        registry.register_project(
            "test-id".to_string(),
            "test-project".to_string(),
            PathBuf::from("/nonexistent"),
            PathBuf::from("/nonexistent/.loglens"),
        ).unwrap();

        let report = validate_links().await.unwrap();
        assert_eq!(report.total_projects, 1);
        assert_eq!(report.valid_projects, 0);
        assert!(report.issues.len() > 0);
    }

    #[tokio::test]
    async fn test_validate_and_repair() {
        // Create a project with missing directory
        let mut registry = ProjectRegistry::default();
        registry.register_project(
            "test-id".to_string(),
            "test-project".to_string(),
            PathBuf::from("/nonexistent"),
            PathBuf::from("/nonexistent/.loglens"),
        ).unwrap();

        let (validation_report, repair_report) = validate_and_repair().await.unwrap();

        // Should have removed the broken project
        assert_eq!(repair_report.removed.len(), 1);
        assert_eq!(repair_report.removed[0], "test-id");

        // Remaining issues should be empty
        assert_eq!(validation_report.issues.len(), 0);
    }
}
