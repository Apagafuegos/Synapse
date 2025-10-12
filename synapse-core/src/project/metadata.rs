// JSON metadata for Synapse projects

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// Project metadata stored in .synapse/metadata.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_id: String,
    pub project_name: String,
    pub project_type: String,
    pub root_path: String,
    pub synapse_version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    #[serde(default)]
    pub linked_analyses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_remote: Option<String>,
}

impl ProjectMetadata {
    /// Create new project metadata with generated UUID
    pub fn new(project_name: String, project_type: String, root_path: String) -> Self {
        let now = Utc::now();

        Self {
            project_id: Uuid::new_v4().to_string(),
            project_name,
            project_type,
            root_path,
            synapse_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            last_updated: now,
            linked_analyses: Vec::new(),
            git_remote: None,
        }
    }

    /// Attempt to detect git remote URL
    pub fn with_git_remote(mut self, remote: Option<String>) -> Self {
        self.git_remote = remote;
        self
    }

    /// Add an analysis ID to linked analyses
    pub fn add_linked_analysis(&mut self, analysis_id: String) {
        if !self.linked_analyses.contains(&analysis_id) {
            self.linked_analyses.push(analysis_id);
            self.last_updated = Utc::now();
        }
    }

    /// Update last_updated timestamp
    pub fn touch(&mut self) {
        self.last_updated = Utc::now();
    }

    /// Serialize to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize metadata: {}", e))
    }

    /// Deserialize from JSON string
    pub fn from_json_str(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata: {}", e))
    }

    /// Load metadata from file
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_json_str(&content)
    }

    /// Save metadata to file
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json_string = self.to_json_string()?;
        tokio::fs::write(path, json_string).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let metadata = ProjectMetadata::new(
            "my-project".to_string(),
            "rust".to_string(),
            "/path/to/project".to_string(),
        );

        assert!(!metadata.project_id.is_empty());
        assert!(Uuid::parse_str(&metadata.project_id).is_ok());
        assert_eq!(metadata.project_name, "my-project");
        assert_eq!(metadata.project_type, "rust");
        assert!(metadata.linked_analyses.is_empty());
        assert!(metadata.git_remote.is_none());
    }

    #[test]
    fn test_metadata_with_git_remote() {
        let metadata = ProjectMetadata::new(
            "test".to_string(),
            "rust".to_string(),
            "/test".to_string(),
        )
        .with_git_remote(Some("https://github.com/user/repo.git".to_string()));

        assert_eq!(
            metadata.git_remote,
            Some("https://github.com/user/repo.git".to_string())
        );
    }

    #[test]
    fn test_add_linked_analysis() {
        let mut metadata = ProjectMetadata::new(
            "test".to_string(),
            "rust".to_string(),
            "/test".to_string(),
        );

        let analysis_id = "analysis-123".to_string();
        metadata.add_linked_analysis(analysis_id.clone());

        assert_eq!(metadata.linked_analyses.len(), 1);
        assert_eq!(metadata.linked_analyses[0], analysis_id);

        // Adding same ID again shouldn't duplicate
        metadata.add_linked_analysis(analysis_id.clone());
        assert_eq!(metadata.linked_analyses.len(), 1);
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "python".to_string(),
            "/test/path".to_string(),
        );

        let json = metadata.to_json_string().unwrap();

        assert!(json.contains("project_id"));
        assert!(json.contains("test-project"));
        assert!(json.contains("python"));
        assert!(json.contains("synapse_version"));
    }

    #[test]
    fn test_metadata_deserialization() {
        let json_str = r#"{
            "project_id": "550e8400-e29b-41d4-a716-446655440000",
            "project_name": "test-project",
            "project_type": "node",
            "root_path": "/path/to/project",
            "synapse_version": "0.1.0",
            "created_at": "2025-10-06T12:00:00Z",
            "last_updated": "2025-10-06T12:00:00Z",
            "linked_analyses": ["analysis-1", "analysis-2"],
            "git_remote": "https://github.com/user/repo.git"
        }"#;

        let metadata = ProjectMetadata::from_json_str(json_str).unwrap();

        assert_eq!(metadata.project_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(metadata.project_name, "test-project");
        assert_eq!(metadata.linked_analyses.len(), 2);
        assert_eq!(metadata.git_remote, Some("https://github.com/user/repo.git".to_string()));
    }

    #[test]
    fn test_metadata_roundtrip() {
        let original = ProjectMetadata::new(
            "roundtrip-test".to_string(),
            "java".to_string(),
            "/test".to_string(),
        );

        let json = original.to_json_string().unwrap();
        let parsed = ProjectMetadata::from_json_str(&json).unwrap();

        assert_eq!(original.project_id, parsed.project_id);
        assert_eq!(original.project_name, parsed.project_name);
        assert_eq!(original.project_type, parsed.project_type);
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut metadata = ProjectMetadata::new(
            "test".to_string(),
            "rust".to_string(),
            "/test".to_string(),
        );

        let original_timestamp = metadata.last_updated;

        std::thread::sleep(std::time::Duration::from_millis(10));
        metadata.touch();

        assert!(metadata.last_updated > original_timestamp);
    }

    #[tokio::test]
    async fn test_metadata_file_operations() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let metadata_path = temp_dir.path().join("metadata.json");

        let metadata = ProjectMetadata::new(
            "file-test".to_string(),
            "rust".to_string(),
            "/test/path".to_string(),
        );

        // Save
        metadata.save(&metadata_path).await.unwrap();
        assert!(metadata_path.exists());

        // Load
        let loaded = ProjectMetadata::load(&metadata_path).await.unwrap();
        assert_eq!(metadata.project_id, loaded.project_id);
        assert_eq!(metadata.project_name, loaded.project_name);
    }
}
