use crate::ai_provider::{
    AIProvider, AnalysisFocus, AnalysisRequest, AnalysisResponse, AnomalyAnalysisSimple,
    ErrorAnalysis, PatternAnalysisSimple, PerformanceAnalysisSimple, RootCauseAnalysis,
};
use crate::classification::ErrorCategory;
use crate::context_manager::ContextManager;
use crate::input::LogEntry;
use crate::slimmer::{slim_logs_with_mode, SlimmingMode};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, PartialEq)]
enum ProcessingStrategy {
    /// Process all logs in a single request (small logs)
    Single,
    /// Apply aggressive slimming and process in single request (medium logs)
    AggressiveSlimming,
    /// Split into chunks and process separately (large logs)
    Chunked,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum tokens per chunk for AI analysis
    pub max_tokens_per_chunk: usize,
    /// Threshold for enabling chunking (number of entries)
    pub chunking_threshold: usize,
    /// Slimming mode to use for large logs
    pub slimming_mode: SlimmingMode,
    /// Maximum number of parallel chunks to process
    pub max_parallel_chunks: usize,
    /// Whether to provide progress feedback
    pub progress_feedback: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_chunk: 8000,
            chunking_threshold: 1000,
            slimming_mode: SlimmingMode::Light,
            max_parallel_chunks: 4,
            progress_feedback: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogChunk {
    pub entries: Vec<LogEntry>,
    pub chunk_id: usize,
    pub total_chunks: usize,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct ChunkAnalysisResult {
    pub chunk_id: usize,
    pub response: AnalysisResponse,
    pub chunk_size: usize,
}

#[derive(Debug, Clone)]
pub struct AnalysisProgress {
    pub current_chunk: usize,
    pub total_chunks: usize,
    pub chunks_completed: usize,
    pub estimated_tokens_processed: usize,
    pub phase: String,
}

pub struct Analyzer {
    provider: Box<dyn AIProvider>,
    config: AnalysisConfig,
}

impl Analyzer {
    pub fn new(provider: Box<dyn AIProvider>) -> Self {
        Self {
            provider,
            config: AnalysisConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnalysisConfig) -> Self {
        self.config = config;
        self
    }

    /// Estimate the token count for a collection of log entries
    fn estimate_tokens(&self, entries: &[LogEntry]) -> usize {
        entries
            .iter()
            .map(|entry| {
                // Rough estimation: 4 chars per token
                let message_tokens = entry.message.len() / 4;
                let timestamp_tokens = entry.timestamp.as_ref().map(|t| t.len() / 4).unwrap_or(0);
                let level_tokens = entry.level.as_ref().map(|l| l.len() / 4).unwrap_or(0);
                message_tokens + timestamp_tokens + level_tokens + 5 // overhead
            })
            .sum()
    }

    /// Determine the best strategy for processing given the log size
    fn determine_processing_strategy(&self, entries: &[LogEntry]) -> ProcessingStrategy {
        let estimated_tokens = self.estimate_tokens(entries);
        let entry_count = entries.len();

        if entry_count <= self.config.chunking_threshold
            && estimated_tokens <= self.config.max_tokens_per_chunk
        {
            ProcessingStrategy::Single
        } else if estimated_tokens <= self.config.max_tokens_per_chunk * 2 {
            ProcessingStrategy::AggressiveSlimming
        } else {
            ProcessingStrategy::Chunked
        }
    }

    /// Split logs into manageable chunks for processing
    fn create_chunks(&self, entries: Vec<LogEntry>) -> Result<Vec<LogChunk>> {
        if entries.is_empty() {
            return Ok(vec![]);
        }

        let target_tokens_per_chunk = self.config.max_tokens_per_chunk;
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0;

        for entry in entries {
            let entry_tokens = self.estimate_tokens(&[entry.clone()]);

            // If adding this entry exceeds the limit and we have entries, start new chunk
            if current_tokens + entry_tokens > target_tokens_per_chunk && !current_chunk.is_empty()
            {
                let chunk_id = chunks.len();
                chunks.push(LogChunk {
                    entries: current_chunk.clone(),
                    chunk_id,
                    total_chunks: 0, // Will be set later
                    estimated_tokens: current_tokens,
                });

                current_chunk.clear();
                current_tokens = 0;
            }

            current_chunk.push(entry);
            current_tokens += entry_tokens;
        }

        // Add the last chunk if it has any entries
        if !current_chunk.is_empty() {
            let chunk_id = chunks.len();
            chunks.push(LogChunk {
                entries: current_chunk,
                chunk_id,
                total_chunks: 0,
                estimated_tokens: current_tokens,
            });
        }

        // Update total_chunks count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    pub async fn analyze_logs(&mut self, entries: Vec<LogEntry>) -> Result<AnalysisResponse> {
        self.analyze_logs_with_context(entries, None).await
    }

    /// Enhanced analysis with adaptive processing strategy
    pub async fn analyze_logs_enhanced(
        &mut self,
        entries: Vec<LogEntry>,
        user_context: Option<String>,
        progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>,
    ) -> Result<AnalysisResponse> {
        info!(
            "Starting enhanced analysis of {} log entries",
            entries.len()
        );
        //DELETE
        for entry in &entries {
            info!("Log Entry: {:?}", entry);
        }

        if entries.is_empty() {
            warn!("No log entries provided for analysis");
            return Ok(self.create_empty_response());
        }

        if self.config.progress_feedback {
            debug!("Initializing progress feedback");
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 0,
                    total_chunks: 1,
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: "Analyzing log size and determining strategy".to_string(),
                });
            }
        }

        let strategy = self.determine_processing_strategy(&entries);
        info!("Selected processing strategy: {:?}", strategy);

        match strategy {
            ProcessingStrategy::Single => {
                info!("Using single chunk processing strategy");
                self.analyze_single_chunk(entries, user_context, progress_callback)
                    .await
            }
            ProcessingStrategy::AggressiveSlimming => {
                info!("Using aggressive slimming strategy");
                self.analyze_with_aggressive_slimming(entries, user_context, progress_callback)
                    .await
            }
            ProcessingStrategy::Chunked => {
                info!("Using chunked processing strategy");
                self.analyze_chunked(entries, user_context, progress_callback)
                    .await
            }
        }
    }

    /// Analyze logs using single request (original behavior)
    async fn analyze_single_chunk(
        &mut self,
        entries: Vec<LogEntry>,
        user_context: Option<String>,
        progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>,
    ) -> Result<AnalysisResponse> {
        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 1,
                    total_chunks: 1,
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: "Processing logs in single request".to_string(),
                });
            }
        }

        // Use light slimming by default
        let slimmed_entries = slim_logs_with_mode(entries.clone(), SlimmingMode::Light);

        let mut context_manager =
            ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

        for (i, entry) in slimmed_entries.iter().enumerate() {
            context_manager.add_log_entry(entry.clone(), i, slimmed_entries.len())?;
        }

        let payload = context_manager.create_ai_payload();
        let analysis_request = AnalysisRequest {
            payload,
            user_context,
            analysis_focus: AnalysisFocus::RootCause,
        };

        let response = self.provider.analyze(analysis_request).await?;

        // Enhance with analytics
        let enhanced_response = self.enhance_with_analytics(response, &entries);

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 1,
                    total_chunks: 1,
                    chunks_completed: 1,
                    estimated_tokens_processed: self.estimate_tokens(&slimmed_entries),
                    phase: "Analysis complete".to_string(),
                });
            }
        }

        Ok(enhanced_response)
    }

    /// Analyze logs with aggressive slimming
    async fn analyze_with_aggressive_slimming(
        &mut self,
        entries: Vec<LogEntry>,
        user_context: Option<String>,
        progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>,
    ) -> Result<AnalysisResponse> {
        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 1,
                    total_chunks: 1,
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: "Applying aggressive slimming to large log set".to_string(),
                });
            }
        }

        // Use the configured slimming mode (likely Aggressive or Ultra)
        let slimmed_entries = slim_logs_with_mode(entries.clone(), self.config.slimming_mode);

        let mut context_manager =
            ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

        for (i, entry) in slimmed_entries.iter().enumerate() {
            context_manager.add_log_entry(entry.clone(), i, slimmed_entries.len())?;
        }

        let payload = context_manager.create_ai_payload();
        let analysis_request = AnalysisRequest {
            payload,
            user_context: user_context.clone(),
            analysis_focus: AnalysisFocus::RootCause,
        };

        let mut response = self.provider.analyze(analysis_request).await?;

        // Add note about aggressive slimming
        response.sequence_of_events = format!(
            "Note: Large log set processed with aggressive slimming. Original {} entries reduced to {} entries.\n\n{}",
            entries.len(),
            slimmed_entries.len(),
            response.sequence_of_events
        );

        // Enhance with analytics using original entries
        let enhanced_response = self.enhance_with_analytics(response, &entries);

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 1,
                    total_chunks: 1,
                    chunks_completed: 1,
                    estimated_tokens_processed: self.estimate_tokens(&slimmed_entries),
                    phase: "Aggressive slimming analysis complete".to_string(),
                });
            }
        }

        Ok(enhanced_response)
    }

    /// Analyze logs using chunking strategy for very large logs
    async fn analyze_chunked(
        &mut self,
        entries: Vec<LogEntry>,
        user_context: Option<String>,
        progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>,
    ) -> Result<AnalysisResponse> {
        // Apply slimming first
        let slimmed_entries = slim_logs_with_mode(entries.clone(), self.config.slimming_mode);

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 0,
                    total_chunks: 1,
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: format!(
                        "Slimmed {} entries to {} entries, creating chunks",
                        entries.len(),
                        slimmed_entries.len()
                    ),
                });
            }
        }

        // Create chunks from slimmed entries
        let chunks = self.create_chunks(slimmed_entries)?;

        if chunks.is_empty() {
            return Ok(self.create_empty_response());
        }

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 0,
                    total_chunks: chunks.len(),
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: format!(
                        "Created {} chunks, starting parallel processing",
                        chunks.len()
                    ),
                });
            }
        }

        // Process chunks in parallel
        let chunk_results = self
            .process_chunks_parallel(chunks, user_context.clone(), &progress_callback)
            .await?;

        // Synthesize results
        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: chunk_results.len(),
                    total_chunks: chunk_results.len(),
                    chunks_completed: chunk_results.len(),
                    estimated_tokens_processed: chunk_results.iter().map(|r| r.chunk_size).sum(),
                    phase: "Synthesizing results from all chunks".to_string(),
                });
            }
        }

        let synthesized_response = self.synthesize_chunk_results(chunk_results, entries.len())?;

        // Enhance with analytics using all original entries
        let enhanced_response = self.enhance_with_analytics(synthesized_response, &entries);

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: enhanced_response.related_errors.len(),
                    total_chunks: enhanced_response.related_errors.len(),
                    chunks_completed: enhanced_response.related_errors.len(),
                    estimated_tokens_processed: self.estimate_tokens(&entries),
                    phase: "Chunked analysis complete".to_string(),
                });
            }
        }

        Ok(enhanced_response)
    }

    /// Process multiple chunks in parallel
    async fn process_chunks_parallel(
        &self,
        chunks: Vec<LogChunk>,
        user_context: Option<String>,
        progress_callback: &Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>,
    ) -> Result<Vec<ChunkAnalysisResult>> {
        let max_parallel = self.config.max_parallel_chunks.min(chunks.len());
        let mut results = Vec::with_capacity(chunks.len());

        // Process chunks in batches to respect parallelism limits
        for chunk_batch in chunks.chunks(max_parallel) {
            for chunk in chunk_batch {
                let chunk_clone = chunk.clone();
                let user_context_clone = user_context.clone();

                // For now, process sequentially to avoid provider sharing issues
                let chunk_result = self
                    .process_single_chunk(chunk_clone, user_context_clone)
                    .await?;
                results.push(chunk_result);

                if self.config.progress_feedback {
                    if let Some(ref callback) = progress_callback {
                        callback(AnalysisProgress {
                            current_chunk: chunk.chunk_id + 1,
                            total_chunks: chunk.total_chunks,
                            chunks_completed: results.len(),
                            estimated_tokens_processed: results.iter().map(|r| r.chunk_size).sum(),
                            phase: format!(
                                "Processed chunk {} of {}",
                                chunk.chunk_id + 1,
                                chunk.total_chunks
                            ),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Process a single chunk
    async fn process_single_chunk(
        &self,
        chunk: LogChunk,
        user_context: Option<String>,
    ) -> Result<ChunkAnalysisResult> {
        let mut context_manager =
            ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

        for (i, entry) in chunk.entries.iter().enumerate() {
            context_manager.add_log_entry(entry.clone(), i, chunk.entries.len())?;
        }

        let payload = context_manager.create_ai_payload();
        let analysis_request = AnalysisRequest {
            payload,
            user_context,
            analysis_focus: AnalysisFocus::RootCause,
        };

        let response = self.provider.analyze(analysis_request).await?;

        Ok(ChunkAnalysisResult {
            chunk_id: chunk.chunk_id,
            response,
            chunk_size: chunk.estimated_tokens,
        })
    }

    /// Synthesize results from multiple chunks into a single response
    fn synthesize_chunk_results(
        &self,
        chunk_results: Vec<ChunkAnalysisResult>,
        original_entry_count: usize,
    ) -> Result<AnalysisResponse> {
        if chunk_results.is_empty() {
            return Ok(self.create_empty_response());
        }

        // Collect all information from chunks
        let mut all_sequences = Vec::new();
        let mut all_recommendations = Vec::new();
        let mut all_related_errors = Vec::new();
        let mut all_unrelated_errors = Vec::new();
        let mut confidence_scores = Vec::new();
        let mut root_causes = Vec::new();

        for (i, chunk_result) in chunk_results.iter().enumerate() {
            let response = &chunk_result.response;

            // Collect sequence of events
            if !response.sequence_of_events.trim().is_empty() {
                all_sequences.push(format!(
                    "Chunk {} ({}): {}",
                    i + 1,
                    chunk_result.chunk_id + 1,
                    response.sequence_of_events
                ));
            }

            // Collect recommendations
            for rec in &response.recommendations {
                if !all_recommendations.contains(rec) {
                    all_recommendations.push(rec.clone());
                }
            }

            // Collect related and unrelated errors
            all_related_errors.extend(response.related_errors.clone());
            all_unrelated_errors.extend(response.unrelated_errors.clone());

            // Collect confidence scores and root causes
            confidence_scores.push(response.confidence);
            root_causes.push(response.root_cause.clone());
        }

        // Synthesize final response
        let synthesized_sequence = if all_sequences.is_empty() {
            format!(
                "Analysis of {} log entries across {} chunks found no significant patterns.",
                original_entry_count,
                chunk_results.len()
            )
        } else {
            format!(
                "Large log analysis across {} chunks (original {} entries):\n\n{}",
                chunk_results.len(),
                original_entry_count,
                all_sequences.join("\n\n")
            )
        };

        // Find the most confident root cause
        let best_root_cause = root_causes
            .into_iter()
            .zip(confidence_scores.iter())
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(rc, _)| rc)
            .unwrap_or_else(|| RootCauseAnalysis {
                category: ErrorCategory::UnknownRelated,
                description: "Multiple chunks analyzed but no clear root cause identified"
                    .to_string(),
                file_location: None,
                line_number: None,
                function_name: None,
                confidence: 0.5,
            });

        // Calculate average confidence
        let avg_confidence = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32
        };

        Ok(AnalysisResponse {
            sequence_of_events: synthesized_sequence,
            root_cause: best_root_cause,
            recommendations: all_recommendations,
            confidence: avg_confidence,
            related_errors: all_related_errors,
            unrelated_errors: all_unrelated_errors,
            errors_found: None,
            patterns: None,
            performance: None,
            anomalies: None,
        })
    }

    /// Enhance analysis response with advanced analytics
    fn enhance_with_analytics(
        &self,
        mut response: AnalysisResponse,
        entries: &[LogEntry],
    ) -> AnalysisResponse {
        // Generate error analytics
        let errors_found = self.generate_error_analytics(entries, &response);

        // Generate pattern analytics
        let patterns = self.generate_pattern_analytics(entries);

        // Generate performance analytics
        let performance = self.generate_performance_analytics(entries);

        // Generate anomaly analytics
        let anomalies = self.generate_anomaly_analytics(entries);

        response.errors_found = Some(errors_found);
        response.patterns = Some(patterns);
        response.performance = Some(performance);
        response.anomalies = Some(anomalies);

        response
    }

    fn generate_error_analytics(
        &self,
        entries: &[LogEntry],
        response: &AnalysisResponse,
    ) -> Vec<ErrorAnalysis> {
        let mut error_map: HashMap<String, (Vec<usize>, Vec<String>)> = HashMap::new();

        for (idx, entry) in entries.iter().enumerate() {
            if entry.level.as_ref().is_some_and(|l| l == "ERROR") {
                let key = entry.message.chars().take(100).collect::<String>();
                error_map
                    .entry(key.clone())
                    .or_insert_with(|| (Vec::new(), Vec::new()))
                    .0
                    .push(idx);
                error_map
                    .get_mut(&key)
                    .unwrap()
                    .1
                    .push(entry.message.clone());
            }
        }

        let mut errors: Vec<ErrorAnalysis> = error_map
            .into_iter()
            .map(|(desc, (lines, contexts))| {
                let frequency = lines.len();
                let severity = if frequency > 10 {
                    "critical"
                } else if frequency > 5 {
                    "high"
                } else if frequency > 2 {
                    "medium"
                } else {
                    "low"
                };

                ErrorAnalysis {
                    category: response.root_cause.category.clone(),
                    description: desc,
                    file_location: response.root_cause.file_location.clone(),
                    line_numbers: lines,
                    frequency,
                    severity: severity.to_string(),
                    context: contexts.into_iter().take(3).collect(),
                    recommendations: vec![],
                }
            })
            .collect();

        errors.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        errors.truncate(10);
        errors
    }

    fn generate_pattern_analytics(&self, entries: &[LogEntry]) -> Vec<PatternAnalysisSimple> {
        let mut pattern_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, entry) in entries.iter().enumerate() {
            let pattern = entry
                .message
                .split_whitespace()
                .take(5)
                .collect::<Vec<&str>>()
                .join(" ");
            pattern_map.entry(pattern).or_default().push(idx);
        }

        let mut patterns: Vec<PatternAnalysisSimple> = pattern_map
            .into_iter()
            .filter(|(_, occurrences)| occurrences.len() > 1)
            .map(|(pattern, occurrences)| {
                let first = *occurrences.first().unwrap();
                let last = *occurrences.last().unwrap();
                let trend = if occurrences.len() > 5 {
                    "increasing"
                } else if occurrences.len() > 2 {
                    "stable"
                } else {
                    "decreasing"
                };

                PatternAnalysisSimple {
                    pattern,
                    frequency: occurrences.len(),
                    first_occurrence: first,
                    last_occurrence: last,
                    trend: trend.to_string(),
                }
            })
            .collect();

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(10);
        patterns
    }

    fn generate_performance_analytics(&self, entries: &[LogEntry]) -> PerformanceAnalysisSimple {
        let error_count = entries
            .iter()
            .filter(|e| e.level.as_ref().is_some_and(|l| l == "ERROR"))
            .count();
        let warn_count = entries
            .iter()
            .filter(|e| e.level.as_ref().is_some_and(|l| l == "WARN"))
            .count();
        let total_count = entries.len();

        let mut metrics = HashMap::new();
        metrics.insert(
            "error_rate".to_string(),
            (error_count as f64 / total_count.max(1) as f64) * 100.0,
        );
        metrics.insert(
            "warning_rate".to_string(),
            (warn_count as f64 / total_count.max(1) as f64) * 100.0,
        );
        metrics.insert("total_logs".to_string(), total_count as f64);

        let bottlenecks = if error_count > total_count / 10 {
            vec!["High error rate detected".to_string()]
        } else {
            vec![]
        };

        PerformanceAnalysisSimple {
            total_processing_time: 0.0,
            bottlenecks,
            recommendations: vec![
                "Review error patterns for optimization opportunities".to_string()
            ],
            metrics,
        }
    }

    fn generate_anomaly_analytics(&self, entries: &[LogEntry]) -> Vec<AnomalyAnalysisSimple> {
        let mut anomalies = Vec::new();
        let avg_length =
            entries.iter().map(|e| e.message.len()).sum::<usize>() / entries.len().max(1);

        for (idx, entry) in entries.iter().enumerate() {
            if entry.message.len() > avg_length * 3 {
                anomalies.push(AnomalyAnalysisSimple {
                    description: "Unusually long log message detected".to_string(),
                    confidence: 0.7,
                    line_numbers: vec![idx],
                    anomaly_type: "length".to_string(),
                });
            }
        }

        anomalies.truncate(5);
        anomalies
    }

    /// Create an empty response for edge cases
    fn create_empty_response(&self) -> AnalysisResponse {
        AnalysisResponse {
            sequence_of_events: "No log entries provided for analysis.".to_string(),
            root_cause: RootCauseAnalysis {
                category: ErrorCategory::UnknownRelated,
                description: "No errors to analyze".to_string(),
                file_location: None,
                line_number: None,
                function_name: None,
                confidence: 0.0,
            },
            recommendations: vec!["Provide log entries for analysis".to_string()],
            confidence: 0.0,
            related_errors: vec![],
            unrelated_errors: vec![],
            errors_found: None,
            patterns: None,
            performance: None,
            anomalies: None,
        }
    }

    pub async fn analyze_logs_with_context(
        &mut self,
        entries: Vec<LogEntry>,
        user_context: Option<String>,
    ) -> Result<AnalysisResponse> {
        // Use enhanced analysis for all processing - it will determine the best strategy
        self.analyze_logs_enhanced(entries, user_context, None)
            .await
    }
}
