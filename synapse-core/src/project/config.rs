// TOML configuration for Synapse projects

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Project configuration stored in .synapse/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectSection,
    pub synapse: SynapseSection,
    #[serde(default)]
    pub mcp: McpSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub root_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseSection {
    #[serde(default = "default_auto_analyze")]
    pub auto_analyze: bool,
    #[serde(default = "default_provider")]
    pub default_provider: String,
    #[serde(default = "default_level")]
    pub default_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpSection {
    #[serde(default = "default_mcp_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
}

fn default_auto_analyze() -> bool {
    true
}

fn default_provider() -> String {
    "openrouter".to_string()
}

fn default_level() -> String {
    "ERROR".to_string()
}

fn default_mcp_enabled() -> bool {
    true
}

impl ProjectConfig {
    /// Create a new project configuration with sensible defaults
    pub fn new(name: String, project_type: String, root_path: String) -> Self {
        let created_at = chrono::Utc::now().to_rfc3339();

        Self {
            project: ProjectSection {
                name,
                project_type,
                root_path,
                created_at,
            },
            synapse: SynapseSection {
                auto_analyze: default_auto_analyze(),
                default_provider: default_provider(),
                default_level: default_level(),
            },
            mcp: McpSection {
                enabled: default_mcp_enabled(),
                server_port: None,
            },
        }
    }

    /// Serialize to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))
    }

    /// Deserialize from TOML string
    pub fn from_toml_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))
    }

    /// Load configuration from file
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_toml_str(&content)
    }

    /// Save configuration to file
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_string = self.to_toml_string()?;
        tokio::fs::write(path, toml_string).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ProjectConfig::new(
            "my-project".to_string(),
            "rust".to_string(),
            "/path/to/project".to_string(),
        );

        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.project.project_type, "rust");
        assert_eq!(config.synapse.auto_analyze, true);
        assert_eq!(config.synapse.default_provider, "openrouter");
        assert_eq!(config.mcp.enabled, true);
    }

    #[test]
    fn test_config_serialization() {
        let config = ProjectConfig::new(
            "test-project".to_string(),
            "java".to_string(),
            "/test/path".to_string(),
        );

        let toml = config.to_toml_string().unwrap();

        assert!(toml.contains("name = \"test-project\""));
        assert!(toml.contains("type = \"java\""));
        assert!(toml.contains("auto_analyze = true"));
        assert!(toml.contains("[project]"));
        assert!(toml.contains("[synapse]"));
        assert!(toml.contains("[mcp]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[project]
name = "test-project"
type = "python"
root_path = "/path/to/project"
created_at = "2025-10-06T12:00:00Z"

[synapse]
auto_analyze = false
default_provider = "claude"
default_level = "WARN"

[mcp]
enabled = true
server_port = 3000
"#;

        let config = ProjectConfig::from_toml_str(toml_str).unwrap();

        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.project_type, "python");
        assert_eq!(config.synapse.auto_analyze, false);
        assert_eq!(config.synapse.default_provider, "claude");
        assert_eq!(config.synapse.default_level, "WARN");
        assert_eq!(config.mcp.server_port, Some(3000));
    }

    #[test]
    fn test_config_roundtrip() {
        let original = ProjectConfig::new(
            "roundtrip-test".to_string(),
            "node".to_string(),
            "/test".to_string(),
        );

        let toml = original.to_toml_string().unwrap();
        let parsed = ProjectConfig::from_toml_str(&toml).unwrap();

        assert_eq!(original.project.name, parsed.project.name);
        assert_eq!(original.project.project_type, parsed.project.project_type);
        assert_eq!(original.synapse.auto_analyze, parsed.synapse.auto_analyze);
    }

    #[tokio::test]
    async fn test_config_file_operations() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = ProjectConfig::new(
            "file-test".to_string(),
            "rust".to_string(),
            "/test/path".to_string(),
        );

        // Save
        config.save(&config_path).await.unwrap();
        assert!(config_path.exists());

        // Load
        let loaded = ProjectConfig::load(&config_path).await.unwrap();
        assert_eq!(config.project.name, loaded.project.name);
    }
}
