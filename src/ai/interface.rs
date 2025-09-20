//! AI Interface Definitions
//! 
//! Defines the core traits and structures for AI/ML integration

use crate::model::LogEntry;
use crate::config::{AnalysisDepth};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// AI Provider interface for external ML integration
pub trait AiProvider {
    /// Initialize the AI provider with configuration
    fn initialize(&mut self, config: AiProviderConfig) -> Result<(), AiError>;
    
    /// Process log entries for anomaly detection
    fn detect_anomalies(&self, entries: &[LogEntry], request: AnomalyDetectionRequest) -> Result<AnomalyDetectionResponse, AiError>;
    
    /// Process log entries for pattern clustering
    fn cluster_patterns(&self, entries: &[LogEntry], request: PatternClusteringRequest) -> Result<PatternClusteringResponse, AiError>;
    
    /// Perform natural language processing on log messages
    fn analyze_text(&self, entries: &[LogEntry], request: TextAnalysisRequest) -> Result<TextAnalysisResponse, AiError>;
    
    /// Perform time series analysis
    fn analyze_time_series(&self, entries: &[LogEntry], request: TimeSeriesRequest) -> Result<TimeSeriesResponse, AiError>;
    
    /// Get provider capabilities
    fn get_capabilities(&self) -> AiProviderCapabilities;
    
    /// Health check for the AI provider
    fn health_check(&self) -> Result<AiProviderHealth, AiError>;
}

/// Configuration for AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub provider_type: AiProviderType,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub batch_size: usize,
    pub custom_params: HashMap<String, String>,
}

/// Types of AI providers supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiProviderType {
    /// Local Python integration
    PythonLocal,
    /// Remote REST API
    RestApi,
    /// gRPC service
    GrpcService,
    /// WebSocket connection
    WebSocket,
    /// Custom implementation
    Custom(String),
}

/// AI provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderCapabilities {
    pub anomaly_detection: bool,
    pub pattern_clustering: bool,
    pub text_analysis: bool,
    pub time_series_analysis: bool,
    pub classification: bool,
    pub embedding_generation: bool,
    pub real_time_processing: bool,
    pub batch_processing: bool,
}

/// Health status of AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderHealth {
    pub is_healthy: bool,
    pub response_time_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
    pub available_models: Vec<String>,
}

/// Request for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionRequest {
    pub algorithm: AnomalyAlgorithm,
    pub sensitivity: f64,
    pub window_size: Option<usize>,
    pub features: Vec<String>,
    pub threshold: Option<f64>,
    pub custom_params: HashMap<String, String>,
}

/// Anomaly detection algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyAlgorithm {
    /// Isolation Forest
    IsolationForest,
    /// One-Class SVM
    OneClassSvm,
    /// Local Outlier Factor
    LocalOutlierFactor,
    /// Statistical methods
    Statistical,
    /// Autoencoder-based
    Autoencoder,
    /// Custom algorithm
    Custom(String),
}

/// Response from anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResponse {
    pub anomalies: Vec<AnomalyResult>,
    pub confidence_scores: Vec<f64>,
    pub processing_time_ms: u64,
    pub model_info: ModelInfo,
}

/// Individual anomaly result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub entry_index: usize,
    pub anomaly_score: f64,
    pub confidence: f64,
    pub anomaly_type: String,
    pub description: String,
    pub features_contributing: Vec<String>,
}

/// Request for pattern clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternClusteringRequest {
    pub algorithm: ClusteringAlgorithm,
    pub num_clusters: Option<usize>,
    pub distance_metric: String,
    pub min_cluster_size: Option<usize>,
    pub features: Vec<String>,
    pub custom_params: HashMap<String, String>,
}

/// Clustering algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusteringAlgorithm {
    /// K-Means
    KMeans,
    /// DBSCAN
    DBSCAN,
    /// Hierarchical clustering
    Hierarchical,
    /// Gaussian Mixture Model
    GaussianMixture,
    /// Spectral clustering
    Spectral,
    /// Custom algorithm
    Custom(String),
}

/// Response from pattern clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternClusteringResponse {
    pub clusters: Vec<PatternCluster>,
    pub cluster_labels: Vec<usize>,
    pub silhouette_score: Option<f64>,
    pub processing_time_ms: u64,
    pub model_info: ModelInfo,
}

/// Pattern cluster information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCluster {
    pub cluster_id: usize,
    pub size: usize,
    pub centroid: Vec<f64>,
    pub patterns: Vec<String>,
    pub representative_entries: Vec<usize>,
    pub confidence: f64,
}

/// Request for text analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnalysisRequest {
    pub analysis_type: TextAnalysisType,
    pub language: Option<String>,
    pub include_embeddings: bool,
    pub custom_params: HashMap<String, String>,
}

/// Text analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAnalysisType {
    /// Named Entity Recognition
    NamedEntityRecognition,
    /// Sentiment Analysis
    SentimentAnalysis,
    /// Topic Modeling
    TopicModeling,
    /// Keyword Extraction
    KeywordExtraction,
    /// Text Classification
    TextClassification,
    /// Custom analysis
    Custom(String),
}

/// Response from text analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnalysisResponse {
    pub results: Vec<TextAnalysisResult>,
    pub embeddings: Option<Vec<Vec<f64>>>,
    pub processing_time_ms: u64,
    pub model_info: ModelInfo,
}

/// Individual text analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnalysisResult {
    pub entry_index: usize,
    pub entities: Vec<Entity>,
    pub sentiment: Option<SentimentResult>,
    pub topics: Vec<Topic>,
    pub keywords: Vec<Keyword>,
    pub classification: Option<ClassificationResult>,
}

/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub label: String,
    pub confidence: f64,
    pub start: usize,
    pub end: usize,
}

/// Sentiment analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub sentiment: String,
    pub confidence: f64,
    pub positive_score: f64,
    pub negative_score: f64,
    pub neutral_score: f64,
}

/// Topic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub topic_id: usize,
    pub keywords: Vec<String>,
    pub probability: f64,
}

/// Keyword information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub word: String,
    pub score: f64,
    pub frequency: usize,
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub label: String,
    pub confidence: f64,
    pub probabilities: HashMap<String, f64>,
}

/// Request for time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesRequest {
    pub analysis_type: TimeSeriesAnalysisType,
    pub window_size: Option<usize>,
    pub forecast_steps: Option<usize>,
    pub features: Vec<String>,
    pub custom_params: HashMap<String, String>,
}

/// Time series analysis types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSeriesAnalysisType {
    /// Trend analysis
    TrendAnalysis,
    /// Seasonality detection
    SeasonalityDetection,
    /// Forecasting
    Forecasting,
    /// Change point detection
    ChangePointDetection,
    /// Decomposition
    Decomposition,
    /// Custom analysis
    Custom(String),
}

/// Response from time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesResponse {
    pub results: Vec<TimeSeriesResult>,
    pub forecast: Option<Vec<f64>>,
    pub processing_time_ms: u64,
    pub model_info: ModelInfo,
}

/// Individual time series result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesResult {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub trend: Option<f64>,
    pub seasonality: Option<f64>,
    pub residual: Option<f64>,
    pub is_anomaly: bool,
    pub confidence_interval: Option<(f64, f64)>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_name: String,
    pub model_version: String,
    pub training_timestamp: Option<DateTime<Utc>>,
    pub parameters: HashMap<String, String>,
    pub performance_metrics: HashMap<String, f64>,
}

/// AI error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiError {
    /// Configuration error
    Configuration(String),
    /// Network error
    Network(String),
    /// Processing error
    Processing(String),
    /// Model error
    Model(String),
    /// Timeout error
    Timeout(String),
    /// Authentication error
    Authentication(String),
    /// Rate limit error
    RateLimit(String),
    /// Invalid input
    InvalidInput(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            AiError::Network(msg) => write!(f, "Network error: {}", msg),
            AiError::Processing(msg) => write!(f, "Processing error: {}", msg),
            AiError::Model(msg) => write!(f, "Model error: {}", msg),
            AiError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            AiError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            AiError::RateLimit(msg) => write!(f, "Rate limit error: {}", msg),
            AiError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AiError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

/// LLM Provider interface for language model integration
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Analyze log entries and generate insights
    async fn analyze_logs(&self, request: LogAnalysisRequest) -> Result<LogAnalysisResponse, AiError>;
    
    /// Generate recommendations based on analysis
    async fn generate_recommendations(&self, analysis: &str) -> Result<String, AiError>;
    
    /// Check provider health and connectivity
    async fn health_check(&self) -> Result<ProviderHealth, AiError>;
    
    /// Get list of available models
    fn available_models(&self) -> Vec<String>;
    
    /// Check if provider supports streaming responses
    fn supports_streaming(&self) -> bool;
    
    /// Get provider capabilities
    fn get_capabilities(&self) -> LlmProviderCapabilities;
}

/// Log analysis request for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisRequest {
    pub log_entries: Vec<LogEntry>,
    pub analysis_depth: AnalysisDepth,
    pub focus_areas: Vec<AnalysisFocus>,
    pub output_format: OutputFormat,
    pub include_context: bool,
    pub max_context_entries: usize,
    pub custom_prompt: Option<String>,
    pub provider_override: Option<String>,
}

/// Analysis focus areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisFocus {
    Errors,
    Performance,
    Security,
    Configuration,
    UserActivity,
    SystemEvents,
    Custom(String),
}

/// Output formats for analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Structured,
    HumanReadable,
    Json,
    Markdown,
    Custom(String),
}

/// Log analysis response from LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisResponse {
    pub provider: String,
    pub model: String,
    pub analysis_id: String,
    pub timestamp: DateTime<Utc>,
    pub summary: LogAnalysisSummary,
    pub detailed_analysis: Option<DetailedAnalysis>,
    pub recommendations: Option<Vec<Recommendation>>,
    pub processing_time_ms: u64,
    pub token_usage: Option<TokenUsage>,
    pub metadata: HashMap<String, String>,
}

/// Summary of log analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisSummary {
    pub overall_status: AnalysisStatus,
    pub key_findings: Vec<String>,
    pub error_count: usize,
    pub warning_count: usize,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub affected_systems: Vec<String>,
}

/// Detailed analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAnalysis {
    pub failure_analysis: Option<FailureAnalysis>,
    pub trigger_analysis: Option<TriggerAnalysis>,
    pub pattern_analysis: Option<PatternAnalysis>,
    pub timeline_analysis: Option<TimelineAnalysis>,
}

/// Failure analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    pub root_causes: Vec<RootCause>,
    pub impact_assessment: ImpactAssessment,
    pub failure_timeline: Vec<FailureEvent>,
}

/// Root cause identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub cause_type: String,
    pub description: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub severity: ImpactSeverity,
    pub affected_components: Vec<String>,
    pub user_impact: String,
    pub business_impact: String,
}

/// Impact severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Failure event in timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub severity: ImpactSeverity,
    pub related_entries: Vec<usize>, // indices of related log entries
}

/// Trigger analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAnalysis {
    pub triggers_identified: Vec<Trigger>,
    pub causal_chains: Vec<CausalChain>,
    pub contributing_factors: Vec<ContributingFactor>,
}

/// Identified trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub trigger_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub immediate_effects: Vec<String>,
    pub confidence: f64,
}

/// Causal chain of events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalChain {
    pub chain_id: String,
    pub events: Vec<CausalEvent>,
    pub root_cause: String,
    pub final_outcome: String,
}

/// Event in causal chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub causes: Vec<String>,
    pub effects: Vec<String>,
}

/// Contributing factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributingFactor {
    pub factor_type: String,
    pub description: String,
    pub importance: f64,
    pub mitigation_suggestions: Vec<String>,
}

/// Pattern analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub patterns_identified: Vec<LogPattern>,
    pub anomaly_patterns: Vec<AnomalyPattern>,
    pub trend_analysis: Option<TrendAnalysis>,
}

/// Identified log pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub description: String,
    pub frequency: usize,
    pub sample_entries: Vec<usize>,
    pub time_distribution: Option<String>,
}

/// Anomaly pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPattern {
    pub anomaly_type: String,
    pub severity: ImpactSeverity,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub affected_entries: Vec<usize>,
    pub anomaly_score: f64,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trends_identified: Vec<Trend>,
    pub predictions: Option<Vec<Prediction>>,
    pub statistical_significance: f64,
}

/// Identified trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub trend_type: String,
    pub direction: TrendDirection,
    pub description: String,
    pub time_period: String,
    pub confidence: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Fluctuating,
}

/// Prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_type: String,
    pub predicted_value: String,
    pub timeframe: String,
    pub confidence: f64,
}

/// Timeline analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineAnalysis {
    pub timeline_events: Vec<TimelineEvent>,
    pub critical_periods: Vec<TimePeriod>,
    pub activity_patterns: Vec<ActivityPattern>,
}

/// Timeline event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub description: String,
    pub significance: f64,
    pub related_events: Vec<String>,
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub period_type: String,
    pub description: String,
    pub metrics: HashMap<String, f64>,
}

/// Activity pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPattern {
    pub pattern_type: String,
    pub description: String,
    pub typical_times: Vec<String>,
    pub duration: String,
    pub frequency: String,
}

/// Recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_id: String,
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub expected_outcome: String,
    pub estimated_effort: String,
    pub related_issues: Vec<String>,
    pub confidence: f64,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Fix,
    Optimization,
    Configuration,
    Monitoring,
    Security,
    Performance,
    Custom(String),
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_usd: Option<f64>,
}

/// Analysis status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Provider health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub response_time_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
    pub available_models: Vec<String>,
}

/// LLM provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderCapabilities {
    pub log_analysis: bool,
    pub recommendation_generation: bool,
    pub streaming: bool,
    pub custom_prompts: bool,
    pub context_window: usize,
    pub supported_formats: Vec<OutputFormat>,
    pub max_input_tokens: Option<usize>,
    pub max_output_tokens: Option<usize>,
}