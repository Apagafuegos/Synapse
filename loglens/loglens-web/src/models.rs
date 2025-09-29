use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LogFile {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub file_size: i64,
    pub line_count: i64,
    pub upload_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Analysis {
    pub id: String,
    pub project_id: String,
    pub log_file_id: Option<String>,
    pub analysis_type: String, // "file" or "realtime"
    pub provider: String,
    pub level_filter: String,
    pub status: AnalysisStatus,
    pub result: Option<String>, // JSON serialized AnalysisResponse
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PerformanceMetric {
    pub id: String,
    pub analysis_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub unit: String,
    pub threshold_value: Option<f64>,
    pub is_bottleneck: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorPattern {
    pub id: String,
    pub project_id: String,
    pub pattern: String,
    pub category: String,
    pub description: Option<String>,
    pub frequency: i32,
    pub last_seen: DateTime<Utc>,
    pub suggested_solution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KnowledgeBaseEntry {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub problem_description: String,
    pub solution: String,
    pub tags: Option<String>,
    pub severity: String,
    pub is_public: bool,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorCorrelation {
    pub id: String,
    pub project_id: String,
    pub primary_error_id: String,
    pub correlated_error_id: String,
    pub correlation_strength: f64,
    pub correlation_type: String,
    pub created_at: DateTime<Utc>,
}

impl PerformanceMetric {
    pub fn new(
        analysis_id: String,
        metric_name: String,
        metric_value: f64,
        unit: String,
        threshold_value: Option<f64>,
    ) -> Self {
        let is_bottleneck = threshold_value.map_or(false, |threshold| metric_value > threshold);
        Self {
            id: Uuid::new_v4().to_string(),
            analysis_id,
            metric_name,
            metric_value,
            unit,
            threshold_value,
            is_bottleneck,
            created_at: Utc::now(),
        }
    }
}

impl ErrorPattern {
    pub fn new(
        project_id: String,
        pattern: String,
        category: String,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            pattern,
            category,
            description,
            frequency: 1,
            last_seen: now,
            suggested_solution: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl KnowledgeBaseEntry {
    pub fn new(
        project_id: String,
        title: String,
        problem_description: String,
        solution: String,
        severity: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            title,
            problem_description,
            solution,
            tags: None,
            severity,
            is_public: false,
            usage_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

impl ErrorCorrelation {
    pub fn new(
        project_id: String,
        primary_error_id: String,
        correlated_error_id: String,
        correlation_strength: f64,
        correlation_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            primary_error_id,
            correlated_error_id,
            correlation_strength,
            correlation_type,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum AnalysisStatus {
    #[default]
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
}

impl std::fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisStatus::Pending => write!(f, "pending"),
            AnalysisStatus::Running => write!(f, "running"),
            AnalysisStatus::Completed => write!(f, "completed"),
            AnalysisStatus::Failed => write!(f, "failed"),
        }
    }
}

impl PartialEq<&str> for AnalysisStatus {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub provider: String,
    pub level: String,
    pub user_context: Option<String>,
    pub selected_model: Option<String>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisListResponse {
    pub analyses: Vec<AnalysisWithFile>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWithFile {
    #[serde(flatten)]
    pub analysis: Analysis,
    pub filename: Option<String>,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

impl LogFile {
    pub fn new(
        project_id: String,
        filename: String,
        file_size: i64,
        line_count: i64,
        upload_path: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            filename,
            file_size,
            line_count,
            upload_path,
            created_at: Utc::now(),
        }
    }
}

impl Analysis {
    pub fn new(
        project_id: String,
        log_file_id: Option<String>,
        analysis_type: String,
        provider: String,
        level_filter: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            project_id,
            log_file_id,
            analysis_type,
            provider,
            level_filter,
            status: AnalysisStatus::Pending,
            result: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }
}
