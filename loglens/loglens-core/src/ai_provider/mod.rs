use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;
use crate::context_manager::AIAnalysisPayload;
use crate::classification::ErrorCategory;

pub mod openrouter;
pub mod openai;
pub mod claude;
pub mod gemini;
pub mod prompts;

pub use openrouter::OpenRouterProvider;
pub use openai::OpenAIProvider;
pub use claude::ClaudeProvider;
pub use gemini::GeminiProvider;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Authentication failed")]
    AuthenticationError,
    #[error("Rate limited")]
    RateLimited,
    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub payload: AIAnalysisPayload,
    pub user_context: Option<String>,
    pub analysis_focus: AnalysisFocus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisFocus {
    RootCause,
    Performance,
    Security,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub sequence_of_events: String,
    pub root_cause: RootCauseAnalysis,
    pub recommendations: Vec<String>,
    pub confidence: f32,
    pub related_errors: Vec<String>,
    pub unrelated_errors: Vec<String>,
    // Advanced analytics for frontend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors_found: Option<Vec<ErrorAnalysis>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<PatternAnalysisSimple>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceAnalysisSimple>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomalies: Option<Vec<AnomalyAnalysisSimple>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub category: ErrorCategory,
    pub description: String,
    pub file_location: Option<String>,
    pub line_numbers: Vec<usize>,
    pub frequency: usize,
    pub severity: String,
    pub context: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisSimple {
    pub pattern: String,
    pub frequency: usize,
    pub first_occurrence: usize,
    pub last_occurrence: usize,
    pub trend: String, // "increasing", "decreasing", "stable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisSimple {
    pub total_processing_time: f64,
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAnalysisSimple {
    pub description: String,
    pub confidence: f32,
    pub line_numbers: Vec<usize>,
    pub anomaly_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub category: ErrorCategory,
    pub description: String,
    pub file_location: Option<String>,
    pub line_number: Option<u32>,
    pub function_name: Option<String>,
    pub confidence: f32,
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

#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResponse, AIError>;
    async fn get_available_models(&self) -> Result<Vec<ModelInfo>, AIError>;
    fn get_provider_name(&self) -> &str;
}

pub fn create_provider(provider_name: &str, api_key: &str) -> Result<Box<dyn AIProvider>> {
    match provider_name.to_lowercase().as_str() {
        "openrouter" => Ok(Box::new(OpenRouterProvider::new(api_key.to_string()))),
        "openai" => Ok(Box::new(OpenAIProvider::new(api_key.to_string()))),
        "claude" | "anthropic" => Ok(Box::new(ClaudeProvider::new(api_key.to_string()))),
        "gemini" => Ok(Box::new(GeminiProvider::new(api_key.to_string()))),
        _ => Err(AIError::UnsupportedProvider(provider_name.to_string()).into()),
    }
}
