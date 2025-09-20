//! AI Processors
//! 
//! High-level processors that coordinate AI operations and provide
/// user-friendly interfaces for common AI tasks.

use crate::ai::interface::*;
use crate::ai::models::*;
use crate::model::LogEntry;
use anyhow::Result;
use std::collections::HashMap;

/// AI processor for log analysis operations
pub struct AIProcessor {
    engine: AIEngine,
    cache: std::collections::HashMap<String, serde_json::Value>,
}

impl AIProcessor {
    /// Create a new AI processor
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self {
            engine: AIEngine::new(provider),
            cache: std::collections::HashMap::new(),
        }
    }
    
    /// Analyze logs for anomalies with intelligent fallback
    pub fn analyze_anomalies(
        &mut self,
        entries: &[LogEntry],
        sensitivity: f64,
        algorithm: Option<AnomalyAlgorithm>,
    ) -> Result<AnomalyAnalysis> {
        let request = AnomalyDetectionRequest {
            algorithm: algorithm.unwrap_or(AnomalyAlgorithm::Statistical),
            sensitivity,
            window_size: Some(100),
            features: vec!["timestamp".to_string(), "level".to_string(), "message".to_string()],
            threshold: Some(sensitivity),
            custom_params: HashMap::new(),
        };
        
        let response = self.engine.hybrid_anomaly_detection(entries, request)?;
        
        Ok(AnomalyAnalysis {
            total_entries: entries.len(),
            anomaly_count: response.anomalies.len(),
            anomalies: response.anomalies,
            processing_time_ms: response.processing_time_ms,
            model_used: response.model_info.model_name,
            confidence_scores: response.confidence_scores,
        })
    }
    
    /// Cluster log patterns with intelligent fallback
    pub fn cluster_log_patterns(
        &mut self,
        entries: &[LogEntry],
        num_clusters: usize,
        algorithm: Option<ClusteringAlgorithm>,
    ) -> Result<PatternAnalysis> {
        let request = PatternClusteringRequest {
            algorithm: algorithm.unwrap_or(ClusteringAlgorithm::KMeans),
            num_clusters: Some(num_clusters),
            distance_metric: "cosine".to_string(),
            min_cluster_size: Some(5),
            features: vec!["message".to_string()],
            custom_params: HashMap::new(),
        };
        
        let response = self.engine.enhanced_pattern_clustering(entries, request)?;
        
        Ok(PatternAnalysis {
            total_entries: entries.len(),
            cluster_count: response.clusters.len(),
            clusters: response.clusters,
            cluster_labels: response.cluster_labels,
            silhouette_score: response.silhouette_score,
            processing_time_ms: response.processing_time_ms,
            model_used: response.model_info.model_name,
        })
    }
    
    /// Analyze text content of logs
    pub fn analyze_text_content(
        &mut self,
        entries: &[LogEntry],
        analysis_type: TextAnalysisType,
        include_embeddings: bool,
    ) -> Result<TextAnalysis> {
        let request = TextAnalysisRequest {
            analysis_type,
            language: Some("en".to_string()),
            include_embeddings,
            custom_params: HashMap::new(),
        };
        
        let response = self.engine.provider().analyze_text(entries, request)?;
        
        Ok(TextAnalysis {
            total_entries: entries.len(),
            results: response.results,
            embeddings: response.embeddings,
            processing_time_ms: response.processing_time_ms,
            model_used: response.model_info.model_name,
        })
    }
    
    /// Perform time series analysis
    pub fn analyze_time_series(
        &mut self,
        entries: &[LogEntry],
        analysis_type: TimeSeriesAnalysisType,
        forecast_steps: Option<usize>,
    ) -> Result<TimeSeriesAnalysis> {
        let request = TimeSeriesRequest {
            analysis_type,
            window_size: Some(100),
            forecast_steps,
            features: vec!["timestamp".to_string(), "level".to_string()],
            custom_params: HashMap::new(),
        };
        
        let response = self.engine.provider().analyze_time_series(entries, request)?;
        
        Ok(TimeSeriesAnalysis {
            total_entries: entries.len(),
            results: response.results,
            forecast: response.forecast,
            processing_time_ms: response.processing_time_ms,
            model_used: response.model_info.model_name,
        })
    }
    
    /// Get AI provider health status
    pub fn get_health_status(&mut self) -> Result<AiProviderHealth> {
        self.engine.provider().health_check().map_err(|e| anyhow::anyhow!(e))
    }
    
    /// Get AI provider capabilities
    pub fn get_capabilities(&mut self) -> AiProviderCapabilities {
        self.engine.provider().get_capabilities()
    }
    
    /// Clear the internal cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.len(),
            memory_usage_bytes: std::mem::size_of_val(&self.cache) as u64,
        }
    }
}

/// High-level anomaly analysis result
#[derive(Debug, Clone)]
pub struct AnomalyAnalysis {
    pub total_entries: usize,
    pub anomaly_count: usize,
    pub anomalies: Vec<AnomalyResult>,
    pub processing_time_ms: u64,
    pub model_used: String,
    pub confidence_scores: Vec<f64>,
}

/// High-level pattern analysis result
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    pub total_entries: usize,
    pub cluster_count: usize,
    pub clusters: Vec<PatternCluster>,
    pub cluster_labels: Vec<usize>,
    pub silhouette_score: Option<f64>,
    pub processing_time_ms: u64,
    pub model_used: String,
}

/// High-level text analysis result
#[derive(Debug, Clone)]
pub struct TextAnalysis {
    pub total_entries: usize,
    pub results: Vec<TextAnalysisResult>,
    pub embeddings: Option<Vec<Vec<f64>>>,
    pub processing_time_ms: u64,
    pub model_used: String,
}

/// High-level time series analysis result
#[derive(Debug, Clone)]
pub struct TimeSeriesAnalysis {
    pub total_entries: usize,
    pub results: Vec<TimeSeriesResult>,
    pub forecast: Option<Vec<f64>>,
    pub processing_time_ms: u64,
    pub model_used: String,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub memory_usage_bytes: u64,
}

/// AI Analysis coordinator that combines multiple analysis types
pub struct AIAnalysisCoordinator {
    processor: AIProcessor,
    analysis_history: Vec<AnalysisRecord>,
}

impl AIAnalysisCoordinator {
    /// Create a new AI analysis coordinator
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self {
            processor: AIProcessor::new(provider),
            analysis_history: Vec::new(),
        }
    }
    
    /// Perform comprehensive log analysis
    pub fn comprehensive_analysis(
        &mut self,
        entries: &[LogEntry],
        options: AnalysisOptions,
    ) -> Result<ComprehensiveAnalysisResult> {
        let mut result = ComprehensiveAnalysisResult {
            total_entries: entries.len(),
            timestamp: chrono::Utc::now(),
            anomaly_analysis: None,
            pattern_analysis: None,
            text_analysis: None,
            time_series_analysis: None,
            health_status: None,
        };
        
        // Perform anomaly detection if requested
        if options.anomaly_detection {
            match self.processor.analyze_anomalies(
                entries,
                options.anomaly_sensitivity.unwrap_or(2.0),
                options.anomaly_algorithm.clone(),
            ) {
                Ok(analysis) => {
                    result.anomaly_analysis = Some(analysis);
                }
                Err(e) => {
                    eprintln!("Anomaly detection failed: {}", e);
                }
            }
        }
        
        // Perform pattern clustering if requested
        if options.pattern_clustering {
            match self.processor.cluster_log_patterns(
                entries,
                options.num_clusters.unwrap_or(10),
                options.clustering_algorithm.clone(),
            ) {
                Ok(analysis) => {
                    result.pattern_analysis = Some(analysis);
                }
                Err(e) => {
                    eprintln!("Pattern clustering failed: {}", e);
                }
            }
        }
        
        // Perform text analysis if requested
        if options.text_analysis {
            match self.processor.analyze_text_content(
                entries,
                options.text_analysis_type.as_ref().unwrap_or(&TextAnalysisType::KeywordExtraction).clone(),
                options.include_embeddings.unwrap_or(false),
            ) {
                Ok(analysis) => {
                    result.text_analysis = Some(analysis);
                }
                Err(e) => {
                    eprintln!("Text analysis failed: {}", e);
                }
            }
        }
        
        // Perform time series analysis if requested
        if options.time_series_analysis {
            match self.processor.analyze_time_series(
                entries,
                options.time_series_type.as_ref().unwrap_or(&TimeSeriesAnalysisType::TrendAnalysis).clone(),
                options.forecast_steps,
            ) {
                Ok(analysis) => {
                    result.time_series_analysis = Some(analysis);
                }
                Err(e) => {
                    eprintln!("Time series analysis failed: {}", e);
                }
            }
        }
        
        // Get health status
        if let Ok(health) = self.processor.get_health_status() {
            result.health_status = Some(health);
        }
        
        // Record this analysis
        self.analysis_history.push(AnalysisRecord {
            timestamp: result.timestamp,
            entries_analyzed: entries.len(),
            options: options.clone(),
            success: result.anomaly_analysis.is_some() || 
                     result.pattern_analysis.is_some() || 
                     result.text_analysis.is_some() ||
                     result.time_series_analysis.is_some(),
        });
        
        Ok(result)
    }
    
    /// Get analysis history
    pub fn get_analysis_history(&self) -> &Vec<AnalysisRecord> {
        &self.analysis_history
    }
    
    /// Clear analysis history
    pub fn clear_history(&mut self) {
        self.analysis_history.clear();
    }
    
    /// Get a reference to the underlying processor
    pub fn processor(&self) -> &AIProcessor {
        &self.processor
    }
    
    /// Get a mutable reference to the underlying processor
    pub fn processor_mut(&mut self) -> &mut AIProcessor {
        &mut self.processor
    }
}

/// Comprehensive analysis options
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub anomaly_detection: bool,
    pub pattern_clustering: bool,
    pub text_analysis: bool,
    pub time_series_analysis: bool,
    pub anomaly_sensitivity: Option<f64>,
    pub anomaly_algorithm: Option<AnomalyAlgorithm>,
    pub num_clusters: Option<usize>,
    pub clustering_algorithm: Option<ClusteringAlgorithm>,
    pub text_analysis_type: Option<TextAnalysisType>,
    pub include_embeddings: Option<bool>,
    pub time_series_type: Option<TimeSeriesAnalysisType>,
    pub forecast_steps: Option<usize>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            anomaly_detection: true,
            pattern_clustering: true,
            text_analysis: false,
            time_series_analysis: false,
            anomaly_sensitivity: Some(2.0),
            anomaly_algorithm: Some(AnomalyAlgorithm::Statistical),
            num_clusters: Some(10),
            clustering_algorithm: Some(ClusteringAlgorithm::KMeans),
            text_analysis_type: Some(TextAnalysisType::KeywordExtraction),
            include_embeddings: Some(false),
            time_series_type: Some(TimeSeriesAnalysisType::TrendAnalysis),
            forecast_steps: None,
        }
    }
}

/// Comprehensive analysis result
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisResult {
    pub total_entries: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub anomaly_analysis: Option<AnomalyAnalysis>,
    pub pattern_analysis: Option<PatternAnalysis>,
    pub text_analysis: Option<TextAnalysis>,
    pub time_series_analysis: Option<TimeSeriesAnalysis>,
    pub health_status: Option<AiProviderHealth>,
}

/// Analysis record for history tracking
#[derive(Debug, Clone)]
pub struct AnalysisRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub entries_analyzed: usize,
    pub options: AnalysisOptions,
    pub success: bool,
}