// Project type detection for Synapse

use anyhow::Result;
use std::path::Path;

/// Supported project types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Java,
    Python,
    Node,
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "rust"),
            ProjectType::Java => write!(f, "java"),
            ProjectType::Python => write!(f, "python"),
            ProjectType::Node => write!(f, "node"),
            ProjectType::Unknown => write!(f, "unknown"),
        }
    }
}

impl ProjectType {
    /// Get project type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Rust => "rust",
            ProjectType::Java => "java",
            ProjectType::Python => "python",
            ProjectType::Node => "node",
            ProjectType::Unknown => "unknown",
        }
    }
}

/// Detect project type by scanning for marker files
pub async fn detect_project_type<P: AsRef<Path>>(project_path: P) -> Result<ProjectType> {
    let path = project_path.as_ref();

    // Rust: Cargo.toml
    if path.join("Cargo.toml").exists() {
        return Ok(ProjectType::Rust);
    }

    // Java: pom.xml (Maven) or build.gradle (Gradle)
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
        return Ok(ProjectType::Java);
    }

    // Python: requirements.txt, setup.py, pyproject.toml, or Pipfile
    if path.join("requirements.txt").exists()
        || path.join("setup.py").exists()
        || path.join("pyproject.toml").exists()
        || path.join("Pipfile").exists()
    {
        return Ok(ProjectType::Python);
    }

    // Node: package.json
    if path.join("package.json").exists() {
        return Ok(ProjectType::Node);
    }

    // Unknown type
    Ok(ProjectType::Unknown)
}

/// Get suggested log file paths for a project type
pub fn get_suggested_log_paths(project_type: ProjectType) -> Vec<&'static str> {
    match project_type {
        ProjectType::Rust => vec!["target/debug/*.log", "target/release/*.log", "*.log"],
        ProjectType::Java => vec!["logs/*.log", "target/*.log", "*.log"],
        ProjectType::Python => vec!["*.log", "logs/*.log", "__pycache__/*.log"],
        ProjectType::Node => vec!["*.log", "logs/*.log", "npm-debug.log", "yarn-error.log"],
        ProjectType::Unknown => vec!["*.log", "logs/*.log"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Rust.to_string(), "rust");
        assert_eq!(ProjectType::Java.to_string(), "java");
        assert_eq!(ProjectType::Python.to_string(), "python");
        assert_eq!(ProjectType::Node.to_string(), "node");
        assert_eq!(ProjectType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_project_type_as_str() {
        assert_eq!(ProjectType::Rust.as_str(), "rust");
        assert_eq!(ProjectType::Java.as_str(), "java");
        assert_eq!(ProjectType::Python.as_str(), "python");
        assert_eq!(ProjectType::Node.as_str(), "node");
        assert_eq!(ProjectType::Unknown.as_str(), "unknown");
    }

    #[tokio::test]
    async fn test_detect_rust_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Rust);
    }

    #[tokio::test]
    async fn test_detect_java_maven_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("pom.xml"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Java);
    }

    #[tokio::test]
    async fn test_detect_java_gradle_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("build.gradle"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Java);
    }

    #[tokio::test]
    async fn test_detect_python_requirements_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("requirements.txt"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Python);
    }

    #[tokio::test]
    async fn test_detect_python_setuppy_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("setup.py"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Python);
    }

    #[tokio::test]
    async fn test_detect_python_pyproject_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("pyproject.toml"), "").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Python);
    }

    #[tokio::test]
    async fn test_detect_node_project() {
        let temp_dir = tempdir().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Node);
    }

    #[tokio::test]
    async fn test_detect_unknown_project() {
        let temp_dir = tempdir().unwrap();
        // No marker files

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Unknown);
    }

    #[tokio::test]
    async fn test_priority_rust_over_others() {
        let temp_dir = tempdir().unwrap();
        // Multiple markers, Rust should take priority
        fs::write(temp_dir.path().join("Cargo.toml"), "").await.unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").await.unwrap();

        let project_type = detect_project_type(temp_dir.path()).await.unwrap();
        assert_eq!(project_type, ProjectType::Rust);
    }

    #[test]
    fn test_suggested_log_paths() {
        let rust_paths = get_suggested_log_paths(ProjectType::Rust);
        assert!(rust_paths.contains(&"target/debug/*.log"));

        let java_paths = get_suggested_log_paths(ProjectType::Java);
        assert!(java_paths.contains(&"logs/*.log"));

        let python_paths = get_suggested_log_paths(ProjectType::Python);
        assert!(python_paths.contains(&"*.log"));

        let node_paths = get_suggested_log_paths(ProjectType::Node);
        assert!(node_paths.contains(&"npm-debug.log"));

        let unknown_paths = get_suggested_log_paths(ProjectType::Unknown);
        assert!(unknown_paths.contains(&"*.log"));
    }
}
