use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: String,
    pub line_number: Option<usize>,
    pub raw_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogFile {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    pub file_size: i64,
    pub line_count: i64,
    pub upload_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Analysis {
    pub id: String,
    pub project_id: String,
    pub log_file_id: Option<String>,
    pub analysis_type: String,
    pub provider: String,
    pub level_filter: String,
    pub status: AnalysisStatus,
    pub result: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisWithFile {
    #[serde(flatten)]
    pub analysis: Analysis,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub provider: String,
    pub level: String,
    pub user_context: Option<String>,
    pub selected_model: Option<String>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: String,
    pub api_key: String,
    pub max_lines: u32,
    pub default_level: String,
    pub show_timestamps: bool,
    pub show_line_numbers: bool,
    pub selected_model: Option<String>,
    pub available_models: Option<String>,
    pub models_last_fetched: Option<String>,
    pub analysis_timeout_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub pricing_tier: Option<String>,
    pub capabilities: Vec<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub models: Vec<ModelInfo>,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsRequest {
    pub provider: String,
    pub api_key: String,
    pub force_refresh: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisListResponse {
    pub analyses: Vec<AnalysisWithFile>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogParsePreview {
    pub total_lines: usize,
    pub error_lines: usize,
    pub warning_lines: usize,
    pub info_lines: usize,
    pub debug_lines: usize,
    pub lines_by_level: Vec<LogLinePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLinePreview {
    pub line_number: usize,
    pub level: Option<String>,
    pub timestamp: Option<String>,
    pub message: String,
    pub is_truncated: bool,
}

impl AnalysisStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalysisStatus::Pending => "pending",
            AnalysisStatus::Running => "running",
            AnalysisStatus::Completed => "completed",
            AnalysisStatus::Failed => "failed",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            AnalysisStatus::Pending => "status-pending",
            AnalysisStatus::Running => "status-running",
            AnalysisStatus::Completed => "status-completed",
            AnalysisStatus::Failed => "status-failed",
        }
    }
}