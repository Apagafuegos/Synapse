mod patterns;
mod performance;
mod anomaly;
mod correlation;

use crate::ai_provider::{AIProvider, AnalysisRequest, AnalysisResponse, AnalysisFocus, RootCauseAnalysis};
use crate::context_manager::ContextManager;
use crate::classification::ErrorCategory;
use crate::input::LogEntry;
use crate::slimmer::{slim_logs_with_mode, SlimmingMode};
use anyhow::Result;
use patterns::PatternAnalyzer;
use performance::PerformanceAnalyzer;
use anomaly::AnomalyDetector;
use correlation::CorrelationAnalyzer;

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
    pattern_analyzer: PatternAnalyzer,
    performance_analyzer: PerformanceAnalyzer,
    anomaly_detector: AnomalyDetector,
    correlation_analyzer: CorrelationAnalyzer,
    config: AnalysisConfig,
}

impl Analyzer {
    pub fn new(provider: Box<dyn AIProvider>) -> Self {
        Self {
            provider,
            pattern_analyzer: PatternAnalyzer::new(),
            performance_analyzer: PerformanceAnalyzer::new(),
            anomaly_detector: AnomalyDetector::new(),
            correlation_analyzer: CorrelationAnalyzer::new(),
            config: AnalysisConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnalysisConfig) -> Self {
        self.config = config;
        self
    }

    /// Estimate the token count for a collection of log entries
    fn estimate_tokens(&self, entries: &[LogEntry]) -> usize {
        entries.iter()
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

        if entry_count <= self.config.chunking_threshold && estimated_tokens <= self.config.max_tokens_per_chunk {
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
            if current_tokens + entry_tokens > target_tokens_per_chunk && !current_chunk.is_empty() {
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
    pub async fn analyze_logs_enhanced(&mut self, entries: Vec<LogEntry>, user_context: Option<String>, progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>) -> Result<AnalysisResponse> {
        if entries.is_empty() {
            return Ok(self.create_empty_response());
        }

        if self.config.progress_feedback {
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

        match strategy {
            ProcessingStrategy::Single => {
                self.analyze_single_chunk(entries, user_context, progress_callback).await
            }
            ProcessingStrategy::AggressiveSlimming => {
                self.analyze_with_aggressive_slimming(entries, user_context, progress_callback).await
            }
            ProcessingStrategy::Chunked => {
                self.analyze_chunked(entries, user_context, progress_callback).await
            }
        }
    }

    /// Analyze logs using single request (original behavior)
    async fn analyze_single_chunk(&mut self, entries: Vec<LogEntry>, user_context: Option<String>, progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>) -> Result<AnalysisResponse> {
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

        let mut context_manager = ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

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

        Ok(response)
    }

    /// Analyze logs with aggressive slimming
    async fn analyze_with_aggressive_slimming(&mut self, entries: Vec<LogEntry>, user_context: Option<String>, progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>) -> Result<AnalysisResponse> {
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

        let mut context_manager = ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

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

        Ok(response)
    }

    /// Analyze logs using chunking strategy for very large logs
    async fn analyze_chunked(&mut self, entries: Vec<LogEntry>, user_context: Option<String>, progress_callback: Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>) -> Result<AnalysisResponse> {
        // Apply slimming first
        let slimmed_entries = slim_logs_with_mode(entries.clone(), self.config.slimming_mode);

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: 0,
                    total_chunks: 1,
                    chunks_completed: 0,
                    estimated_tokens_processed: 0,
                    phase: format!("Slimmed {} entries to {} entries, creating chunks", entries.len(), slimmed_entries.len()),
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
                    phase: format!("Created {} chunks, starting parallel processing", chunks.len()),
                });
            }
        }

        // Process chunks in parallel
        let chunk_results = self.process_chunks_parallel(chunks, user_context.clone(), &progress_callback).await?;

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

        if self.config.progress_feedback {
            if let Some(ref callback) = progress_callback {
                callback(AnalysisProgress {
                    current_chunk: synthesized_response.related_errors.len(),
                    total_chunks: synthesized_response.related_errors.len(),
                    chunks_completed: synthesized_response.related_errors.len(),
                    estimated_tokens_processed: self.estimate_tokens(&entries),
                    phase: "Chunked analysis complete".to_string(),
                });
            }
        }

        Ok(synthesized_response)
    }

    /// Process multiple chunks in parallel
    async fn process_chunks_parallel(&self, chunks: Vec<LogChunk>, user_context: Option<String>, progress_callback: &Option<Box<dyn Fn(AnalysisProgress) + Send + Sync>>) -> Result<Vec<ChunkAnalysisResult>> {
        let max_parallel = self.config.max_parallel_chunks.min(chunks.len());
        let mut results = Vec::with_capacity(chunks.len());

        // Process chunks in batches to respect parallelism limits
        for chunk_batch in chunks.chunks(max_parallel) {
            for chunk in chunk_batch {
                let chunk_clone = chunk.clone();
                let user_context_clone = user_context.clone();

                // For now, process sequentially to avoid provider sharing issues
                let chunk_result = self.process_single_chunk(chunk_clone, user_context_clone).await?;
                results.push(chunk_result);

                if self.config.progress_feedback {
                    if let Some(ref callback) = progress_callback {
                        callback(AnalysisProgress {
                            current_chunk: chunk.chunk_id + 1,
                            total_chunks: chunk.total_chunks,
                            chunks_completed: results.len(),
                            estimated_tokens_processed: results.iter().map(|r| r.chunk_size).sum(),
                            phase: format!("Processed chunk {} of {}", chunk.chunk_id + 1, chunk.total_chunks),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Process a single chunk
    async fn process_single_chunk(&self, chunk: LogChunk, user_context: Option<String>) -> Result<ChunkAnalysisResult> {
        let mut context_manager = ContextManager::new(self.config.max_tokens_per_chunk, user_context.as_deref());

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
    fn synthesize_chunk_results(&self, chunk_results: Vec<ChunkAnalysisResult>, original_entry_count: usize) -> Result<AnalysisResponse> {
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
                all_sequences.push(format!("Chunk {} ({}): {}", i + 1, chunk_result.chunk_id + 1, response.sequence_of_events));
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
            format!("Analysis of {} log entries across {} chunks found no significant patterns.", original_entry_count, chunk_results.len())
        } else {
            format!("Large log analysis across {} chunks (original {} entries):\n\n{}",
                   chunk_results.len(),
                   original_entry_count,
                   all_sequences.join("\n\n"))
        };

        // Find the most confident root cause
        let best_root_cause = root_causes.into_iter()
            .zip(confidence_scores.iter())
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(rc, _)| rc)
            .unwrap_or_else(|| RootCauseAnalysis {
                category: ErrorCategory::UnknownRelated,
                description: "Multiple chunks analyzed but no clear root cause identified".to_string(),
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
        })
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
        }
    }

    pub async fn analyze_logs_with_context(&mut self, entries: Vec<LogEntry>, user_context: Option<String>) -> Result<AnalysisResponse> {
        // Use enhanced analysis for all processing - it will determine the best strategy
        self.analyze_logs_enhanced(entries, user_context, None).await
    }
}