// Data models for Synapse project management and analysis tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a Synapse-initialized project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "project-management", derive(sqlx::FromRow))]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

impl Project {
    pub fn new(name: String, root_path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            root_path,
            description: None,
            created_at: now,
            updated_at: now,
            metadata: None,
        }
    }
}

/// Status of a log analysis operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Pending,
    Completed,
    Failed,
}

impl std::fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisStatus::Pending => write!(f, "pending"),
            AnalysisStatus::Completed => write!(f, "completed"),
            AnalysisStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for AnalysisStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(AnalysisStatus::Pending),
            "completed" => Ok(AnalysisStatus::Completed),
            "failed" => Ok(AnalysisStatus::Failed),
            _ => Err(format!("Invalid analysis status: {}", s)),
        }
    }
}

/// Represents a log analysis operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub id: String,
    pub project_id: String,
    pub log_file_path: String,
    pub provider: String,
    pub level: String,
    pub status: AnalysisStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<String>,
}

#[cfg(feature = "project-management")]
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Analysis {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let status_str: String = row.try_get("status")?;
        let status = status_str
            .parse::<AnalysisStatus>()
            .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            ))))?;

        Ok(Analysis {
            id: row.try_get("id")?,
            project_id: row.try_get("project_id")?,
            log_file_path: row.try_get("log_file_path")?,
            provider: row.try_get("provider")?,
            level: row.try_get("level")?,
            status,
            created_at: row.try_get("created_at")?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

impl Analysis {
    pub fn new(
        project_id: String,
        log_file_path: String,
        provider: String,
        level: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            log_file_path,
            provider,
            level,
            status: AnalysisStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            metadata: None,
        }
    }
}

/// Pattern detected in log analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern: String,
    pub count: usize,
    pub examples: Vec<String>,
    pub severity: String,
    pub confidence: f64,
}

/// Results of a completed log analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub analysis_id: String,
    pub summary: Option<String>,
    pub full_report: Option<String>,
    pub patterns_detected: serde_json::Value,
    pub issues_found: Option<i64>,
    pub metadata: Option<String>,
}

#[cfg(feature = "project-management")]
impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for AnalysisResult {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let patterns_str: String = row.try_get("patterns_detected")?;
        let patterns_detected = serde_json::from_str(&patterns_str)
            .map_err(|e| sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))))?;

        Ok(AnalysisResult {
            analysis_id: row.try_get("analysis_id")?,
            summary: row.try_get("summary")?,
            full_report: row.try_get("full_report")?,
            patterns_detected,
            issues_found: row.try_get("issues_found")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

impl AnalysisResult {
    pub fn new(analysis_id: String) -> Self {
        Self {
            analysis_id,
            summary: None,
            full_report: None,
            patterns_detected: serde_json::json!([]),
            issues_found: None,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new(
            "test-project".to_string(),
            "/path/to/project".to_string(),
        );

        assert!(!project.id.is_empty());
        assert_eq!(project.name, "test-project");
        assert_eq!(project.root_path, "/path/to/project");
        assert!(project.description.is_none());
    }

    #[test]
    fn test_analysis_status_serialization() {
        assert_eq!(AnalysisStatus::Pending.to_string(), "pending");
        assert_eq!(AnalysisStatus::Completed.to_string(), "completed");
        assert_eq!(AnalysisStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_analysis_status_parsing() {
        use std::str::FromStr;

        assert_eq!(
            AnalysisStatus::from_str("pending").unwrap(),
            AnalysisStatus::Pending
        );
        assert_eq!(
            AnalysisStatus::from_str("COMPLETED").unwrap(),
            AnalysisStatus::Completed
        );
        assert!(AnalysisStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_analysis_creation() {
        let analysis = Analysis::new(
            "project-id".to_string(),
            "/path/to/log.log".to_string(),
            "openrouter".to_string(),
            "ERROR".to_string(),
        );

        assert!(!analysis.id.is_empty());
        assert_eq!(analysis.status, AnalysisStatus::Pending);
        assert!(analysis.completed_at.is_none());
    }

    #[test]
    fn test_pattern_serialization() {
        let pattern = Pattern {
            pattern: "NullPointerException".to_string(),
            count: 5,
            examples: vec!["Error example 1".to_string()],
            severity: "high".to_string(),
            confidence: 0.9,
        };

        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("NullPointerException"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_analysis_result_creation() {
        let result = AnalysisResult::new("analysis-id".to_string());

        assert_eq!(result.analysis_id, "analysis-id");
        assert!(result.summary.is_none());
        assert_eq!(result.patterns_detected, serde_json::json!([]));
    }
}
