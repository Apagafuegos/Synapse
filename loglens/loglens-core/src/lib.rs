// LogLens Library - Core log analysis functionality
//
// This library provides the core functionality for log analysis that can be
// used both by the CLI binary and MCP integrations.

use tracing::{info, error, debug, warn};

pub mod ai_provider;
pub mod analyzer;
pub mod classification;
pub mod context_manager;
pub mod config;
pub mod digest;
pub mod filter;
pub mod input;
pub mod mcp_server;
pub mod output;
pub mod parser;
pub mod slimmer;

pub use ai_provider::{create_provider, create_provider_with_model, AIProvider, AnalysisRequest, AnalysisResponse, AIError, AnalysisFocus, RootCauseAnalysis, OpenRouterProvider, OpenAIProvider, ClaudeProvider, GeminiProvider};
pub use analyzer::{Analyzer, AnalysisConfig, AnalysisProgress};
pub use classification::{ErrorClassifier, ErrorClassification, ErrorCategory, Severity};
pub use context_manager::{ContextManager, RelevanceScorer, AIAnalysisPayload, ContextStats};
pub use config::Config;
pub use digest::{IncidentDigest, CriticalError, TimelineEvent, StackTrace, ContextWindow, DigestConfig};
pub use filter::filter_logs_by_level;
pub use input::{execute_and_capture, read_log_file, LogEntry};
pub use output::{generate_report, save_report, OutputFormat, AnalysisReport};
pub use parser::parse_log_lines;
pub use slimmer::{slim_logs, slim_logs_with_mode, SlimmingMode};

// Convenience functions for direct usage (backward compatibility)

/// Analyze log lines using the default LogLens configuration
pub async fn analyze_lines(
    raw_lines: Vec<String>,
    level: &str,
    provider_name: &str,
    api_key: Option<&str>,
    selected_model: Option<&str>,
) -> Result<AnalysisResponse> {
    info!("Starting analysis of {} log lines with provider: {}", raw_lines.len(), provider_name);
    info!("analyze_lines called with selected_model: {:?}", selected_model);
    if let Some(model) = selected_model {
        info!("Using selected model: {}", model);
    } else {
        warn!("No selected model provided to analyze_lines");
    }
    let loglens = match LogLens::new() {
        Ok(loglens) => {
            debug!("LogLens instance created successfully");
            loglens
        }
        Err(e) => {
            error!("Failed to create LogLens instance: {}", e);
            return Err(e);
        }
    };

    match loglens.analyze_lines_with_model(raw_lines, level, provider_name, api_key, selected_model).await {
        Ok(result) => {
            info!("Analysis completed successfully");
            Ok(result)
        }
        Err(e) => {
            error!("Analysis failed: {}", e);
            Err(e)
        }
    }
}

/// Process an MCP request using the default LogLens configuration
pub async fn process_mcp_request(request_json: &str) -> Result<String> {
    debug!("Processing MCP request");
    let loglens = match LogLens::new() {
        Ok(loglens) => loglens,
        Err(e) => {
            error!("Failed to create LogLens instance for MCP request: {}", e);
            return Err(e);
        }
    };
    
    let request = match serde_json::from_str::<McpRequest>(request_json) {
        Ok(request) => {
            debug!("MCP request parsed successfully");
            request
        }
        Err(e) => {
            error!("Failed to parse MCP request JSON: {}", e);
            return Err(e.into());
        }
    };
    
    match loglens.process_mcp_request(request).await {
        Ok(response) => {
            debug!("MCP request processed successfully");
            match serde_json::to_string(&response) {
                Ok(json) => Ok(json),
                Err(e) => {
                    error!("Failed to serialize MCP response: {}", e);
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            error!("MCP request processing failed: {}", e);
            Err(e)
        }
    }
}


use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use digest::{LogStatistics, TimeRange};

/// MCP input structure for JSON I/O mode
#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    /// Raw log lines to analyze
    pub logs: Vec<String>,
    /// Log level to filter by (ERROR, WARN, INFO, DEBUG)
    pub level: String,
    /// AI provider to use (openrouter, openai, claude, gemini)
    pub provider: String,
    /// API key for the provider (optional if set in config/env)
    pub api_key: Option<String>,
    /// Input source description for the report
    pub input_source: Option<String>,
    /// Output format for the analysis
    pub output_format: Option<String>,
}

/// MCP output structure for JSON I/O mode
#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    /// Analysis result from the AI provider
    pub analysis: AnalysisResponse,
    /// Generated report content
    pub report: Option<String>,
    /// Metadata about the analysis
    pub metadata: McpMetadata,
}

/// Metadata for MCP response
#[derive(Debug, Serialize, Deserialize)]
pub struct McpMetadata {
    /// Number of original log lines
    pub total_lines: usize,
    /// Number of filtered log entries
    pub filtered_entries: usize,
    /// Number of slimmed log entries
    pub slimmed_entries: usize,
    /// Provider used for analysis
    pub provider: String,
    /// Log level used for filtering
    pub level: String,
    /// Input source description
    pub input_source: String,
}

/// Core log analysis functionality
pub struct LogLens {
    config: Config,
}

impl LogLens {
    /// Create a new LogLens instance
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self { config })
    }

    /// Create a LogLens instance with custom config
    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    /// Analyze logs from a file
    pub async fn analyze_file(
        &self,
        file_path: &str,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
    ) -> Result<AnalysisResponse> {
        let raw_lines = read_log_file(file_path).await?;
        self.analyze_lines(raw_lines, level, provider_name, api_key).await
    }

    /// Analyze logs from a command execution
    pub async fn analyze_command(
        &self,
        command: &str,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
    ) -> Result<AnalysisResponse> {
        let raw_lines = execute_and_capture(command).await?;
        self.analyze_lines(raw_lines, level, provider_name, api_key).await
    }

    /// Analyze raw log lines
    pub async fn analyze_lines(
        &self,
        raw_lines: Vec<String>,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
    ) -> Result<AnalysisResponse> {
        self.analyze_lines_with_model(raw_lines, level, provider_name, api_key, None).await
    }

    /// Analyze raw log lines with optional model selection
    pub async fn analyze_lines_with_model(
        &self,
        raw_lines: Vec<String>,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
        selected_model: Option<&str>,
    ) -> Result<AnalysisResponse> {
        // Parse logs
        let parsed_entries = parse_log_lines(&raw_lines);

        // Filter by level
        let filtered_entries = filter_logs_by_level(parsed_entries, level)?;

        if filtered_entries.is_empty() {
            return Ok(AnalysisResponse {
                sequence_of_events: "No log entries found matching the specified level.".to_string(),
                root_cause: RootCauseAnalysis {
                    category: ErrorCategory::UnknownRelated,
                    description: "No errors to analyze".to_string(),
                    file_location: None,
                    line_number: None,
                    function_name: None,
                    confidence: 0.0,
                },
                recommendations: vec!["Provide valid log entries for analysis".to_string()],
                confidence: 0.0,
                related_errors: vec![],
                unrelated_errors: vec![],
                errors_found: None,
                patterns: None,
                performance: None,
                anomalies: None,
            });
        }

        // Slim logs
        let slimmed_entries = slim_logs(filtered_entries);

        // Get API key with precedence: parameter > config > env
        let api_key = match api_key {
            Some(key) => key.to_string(),
            None => self.config.get_api_key(provider_name)
                .ok_or_else(|| anyhow::anyhow!(
                    "API key required for provider {}. Set {}_API_KEY environment variable",
                    provider_name,
                    provider_name.to_uppercase()
                ))?,
        };

        // Analyze with AI using enhanced analysis with optional model selection
        info!("Creating provider with model: {:?}", selected_model);
        let provider = create_provider_with_model(
            provider_name,
            &api_key,
            selected_model.map(|s| s.to_string())
        )?;

        // Configure analyzer for large logs
        let analysis_config = AnalysisConfig {
            max_tokens_per_chunk: 8000,
            chunking_threshold: 500, // Enable chunking for 500+ entries
            slimming_mode: if slimmed_entries.len() > 1000 {
                SlimmingMode::Aggressive
            } else if slimmed_entries.len() > 500 {
                SlimmingMode::Light
            } else {
                SlimmingMode::Light
            },
            max_parallel_chunks: 4,
            progress_feedback: false,
        };

        let mut analyzer = Analyzer::new(provider).with_config(analysis_config);
        analyzer.analyze_logs(slimmed_entries).await
    }

    /// Generate a full analysis report
    pub async fn generate_full_report(
        &self,
        raw_lines: Vec<String>,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
        input_source: &str,
        output_format: OutputFormat,
    ) -> Result<String> {
        // Parse logs
        let parsed_entries = parse_log_lines(&raw_lines);

        // Filter by level
        let filtered_entries = filter_logs_by_level(parsed_entries, level)?;

        if filtered_entries.is_empty() {
            return Ok("No log entries found matching the specified level.".to_string());
        }

        // Slim logs
        let slimmed_entries = slim_logs(filtered_entries);

        // Get API key with precedence: parameter > config > env
        let api_key = match api_key {
            Some(key) => key.to_string(),
            None => self.config.get_api_key(provider_name)
                .ok_or_else(|| anyhow::anyhow!(
                    "API key required for provider {}. Set {}_API_KEY environment variable",
                    provider_name,
                    provider_name.to_uppercase()
                ))?,
        };

        // Analyze with AI using enhanced analysis
        let provider = create_provider(provider_name, &api_key)?;

        // Configure analyzer for large logs
        let analysis_config = AnalysisConfig {
            max_tokens_per_chunk: 8000,
            chunking_threshold: 500, // Enable chunking for 500+ entries
            slimming_mode: if slimmed_entries.len() > 1000 {
                SlimmingMode::Aggressive
            } else if slimmed_entries.len() > 500 {
                SlimmingMode::Light
            } else {
                SlimmingMode::Light
            },
            max_parallel_chunks: 4,
            progress_feedback: false,
        };

        let mut analyzer = Analyzer::new(provider).with_config(analysis_config);
        let analysis = analyzer.analyze_logs(slimmed_entries.clone()).await?;

        // Generate report
        generate_report(
            analysis,
            slimmed_entries,
            provider_name,
            level,
            input_source,
            output_format,
        )
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Process an MCP request and return the response
    pub async fn process_mcp_request(&self, request: McpRequest) -> Result<McpResponse> {
        // Parse logs
        let parsed_entries = parse_log_lines(&request.logs);

        // Filter by level
        let filtered_entries = filter_logs_by_level(parsed_entries, &request.level)?;

        if filtered_entries.is_empty() {
            let analysis = AnalysisResponse {
                sequence_of_events: "No log entries found matching the specified level.".to_string(),
                root_cause: RootCauseAnalysis {
                    category: ErrorCategory::UnknownRelated,
                    description: "No errors to analyze".to_string(),
                    file_location: None,
                    line_number: None,
                    function_name: None,
                    confidence: 0.0,
                },
                recommendations: vec!["Provide valid log entries for analysis".to_string()],
                confidence: 0.0,
                related_errors: vec![],
                unrelated_errors: vec![],
                errors_found: None,
                patterns: None,
                performance: None,
                anomalies: None,
            };

            return Ok(McpResponse {
                analysis,
                report: None,
                metadata: McpMetadata {
                    total_lines: request.logs.len(),
                    filtered_entries: 0,
                    slimmed_entries: 0,
                    provider: request.provider,
                    level: request.level,
                    input_source: request.input_source.unwrap_or_else(|| "stdin".to_string()),
                },
            });
        }

        // Slim logs
        let slimmed_entries = slim_logs(filtered_entries.clone());

        // Get API key with precedence: parameter > config > env
        let api_key = match request.api_key {
            Some(key) => key,
            None => self.config.get_api_key(&request.provider)
                .ok_or_else(|| anyhow::anyhow!(
                    "API key required for provider {}. Set {}_API_KEY environment variable",
                    request.provider,
                    request.provider.to_uppercase()
                ))?,
        };

        // Analyze with AI using enhanced analysis
        let provider = create_provider(&request.provider, &api_key)?;

        // Configure analyzer for large logs with progress feedback for MCP
        let analysis_config = AnalysisConfig {
            max_tokens_per_chunk: 8000,
            chunking_threshold: 500, // Enable chunking for 500+ entries
            slimming_mode: if slimmed_entries.len() > 1000 {
                SlimmingMode::Aggressive
            } else if slimmed_entries.len() > 500 {
                SlimmingMode::Light
            } else {
                SlimmingMode::Light
            },
            max_parallel_chunks: 4,
            progress_feedback: true, // Enable progress feedback for MCP
        };

        let mut analyzer = Analyzer::new(provider).with_config(analysis_config);
        let analysis = analyzer.analyze_logs(slimmed_entries.clone()).await?;

        // Generate report if format is specified
        let report = if let Some(format_str) = &request.output_format {
            if let Some(output_format) = OutputFormat::from_str(format_str) {
                Some(generate_report(
                    analysis.clone(),
                    slimmed_entries.clone(),
                    &request.provider,
                    &request.level,
                    request.input_source.as_deref().unwrap_or("stdin"),
                    output_format,
                )?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(McpResponse {
            analysis,
            report,
            metadata: McpMetadata {
                total_lines: request.logs.len(),
                filtered_entries: filtered_entries.len(),
                slimmed_entries: slimmed_entries.len(),
                provider: request.provider,
                level: request.level,
                input_source: request.input_source.unwrap_or_else(|| "stdin".to_string()),
            },
        })
    }

    /// Create an incident digest from raw log lines - optimized for MCP integration
    pub async fn create_incident_digest(
        &self,
        raw_lines: Vec<String>,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
        config: Option<DigestConfig>,
    ) -> Result<IncidentDigest> {
        let start_time = std::time::Instant::now();
        let config = config.unwrap_or_default();

        // Parse logs
        let parsed_entries = parse_log_lines(&raw_lines);

        // Filter by level
        let filtered_entries = filter_logs_by_level(parsed_entries, level)?;

        if filtered_entries.is_empty() {
            let mut digest = IncidentDigest::new(uuid::Uuid::new_v4().to_string());
            digest.severity = "LOW".to_string();
            digest.root_cause_analysis = "No log entries found matching the specified level.".to_string();
            digest.log_stats = self.calculate_statistics(&[], &raw_lines);
            digest.processing_time_ms = start_time.elapsed().as_millis() as u64;
            return Ok(digest);
        }

        // Get API key with precedence: parameter > config > env
        let api_key = match api_key {
            Some(key) => key.to_string(),
            None => self.config.get_api_key(provider_name)
                .ok_or_else(|| anyhow::anyhow!(
                    "API key required for provider {}. Set {}_API_KEY environment variable",
                    provider_name,
                    provider_name.to_uppercase()
                ))?,
        };

        // Slim logs for AI analysis
        let slimmed_entries = slim_logs(filtered_entries.clone());

        // Analyze with AI using enhanced analysis
        let provider = create_provider(provider_name, &api_key)?;

        // Configure analyzer for large logs with appropriate settings for incident digest
        let analysis_config = AnalysisConfig {
            max_tokens_per_chunk: 8000,
            chunking_threshold: 500, // Enable chunking for 500+ entries
            slimming_mode: if slimmed_entries.len() > 1000 {
                SlimmingMode::Ultra // More aggressive for incident digest
            } else if slimmed_entries.len() > 500 {
                SlimmingMode::Aggressive
            } else {
                SlimmingMode::Light
            },
            max_parallel_chunks: 4,
            progress_feedback: false, // No progress feedback for incident digest
        };

        let mut analyzer = Analyzer::new(provider).with_config(analysis_config);
        let ai_analysis = analyzer.analyze_logs(slimmed_entries).await?;

        // Create incident digest
        let mut digest = IncidentDigest::new(uuid::Uuid::new_v4().to_string());

        // Extract structured data
        digest.critical_errors = self.extract_critical_errors(&filtered_entries, &config);
        digest.error_timeline = self.create_error_timeline(&filtered_entries, &config);
        digest.stack_traces = self.extract_stack_traces(&raw_lines, &config);
        digest.context_snippets = self.extract_context_windows(&raw_lines, &filtered_entries, &config);

        // Set analysis results
        digest.root_cause_analysis = ai_analysis.sequence_of_events.clone();
        digest.recommended_actions = self.extract_recommendations(&ai_analysis);
        digest.investigation_areas = self.suggest_investigation_areas(&filtered_entries);

        // Calculate metadata
        digest.severity = self.calculate_severity(&filtered_entries);
        digest.log_stats = self.calculate_statistics(&filtered_entries, &raw_lines);
        digest.processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(digest)
    }

    /// Create incident digest from a file
    pub async fn create_incident_digest_from_file(
        &self,
        file_path: &str,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
        config: Option<DigestConfig>,
    ) -> Result<IncidentDigest> {
        let raw_lines = read_log_file(file_path).await?;
        self.create_incident_digest(raw_lines, level, provider_name, api_key, config).await
    }

    /// Create incident digest from command execution
    pub async fn create_incident_digest_from_command(
        &self,
        command: &str,
        level: &str,
        provider_name: &str,
        api_key: Option<&str>,
        config: Option<DigestConfig>,
    ) -> Result<IncidentDigest> {
        let raw_lines = execute_and_capture(command).await?;
        self.create_incident_digest(raw_lines, level, provider_name, api_key, config).await
    }

    // === Incident Digest Extraction Methods ===

    /// Extract critical errors from filtered log entries
    fn extract_critical_errors(&self, entries: &[LogEntry], config: &DigestConfig) -> Vec<CriticalError> {
        let mut error_map: HashMap<String, CriticalError> = HashMap::new();

        for entry in entries {
            // Only process ERROR level entries
            if entry.level.as_deref() != Some("ERROR") {
                continue;
            }

            // Extract error type from message
            let error_type = self.classify_error_type(&entry.message);
            let component = self.extract_component_from_message(&entry.message);

            // Update or create error entry
            let critical_error = error_map.entry(error_type.clone()).or_insert_with(|| CriticalError {
                error_type: error_type.clone(),
                message: entry.message.clone(),
                frequency: 0,
                first_occurrence: entry.timestamp.clone(),
                last_occurrence: entry.timestamp.clone(),
                affected_components: Vec::new(),
                confidence: 0.8, // Default confidence
            });

            critical_error.frequency += 1;
            if let Some(timestamp) = &entry.timestamp {
                critical_error.last_occurrence = Some(timestamp.clone());
                if critical_error.first_occurrence.is_none() {
                    critical_error.first_occurrence = Some(timestamp.clone());
                }
            }

            if let Some(comp) = component {
                if !critical_error.affected_components.contains(&comp) {
                    critical_error.affected_components.push(comp);
                }
            }
        }

        // Filter by minimum frequency and sort by frequency
        let mut errors: Vec<CriticalError> = error_map
            .into_values()
            .filter(|e| e.frequency >= config.min_error_frequency)
            .collect();

        errors.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        errors.truncate(config.max_critical_errors);
        errors
    }

    /// Create error timeline from log entries
    fn create_error_timeline(&self, entries: &[LogEntry], config: &DigestConfig) -> Vec<TimelineEvent> {
        let mut timeline: Vec<TimelineEvent> = Vec::new();
        let mut seen_events: HashSet<String> = HashSet::new();

        for entry in entries {
            // Skip low severity events if not configured to include them
            if !config.include_low_severity && matches!(entry.level.as_deref(), Some("INFO") | Some("DEBUG")) {
                continue;
            }

            let event_key = format!("{}_{}",
                entry.timestamp.as_deref().unwrap_or("unknown"),
                &entry.message[..entry.message.len().min(50)]
            );

            if seen_events.contains(&event_key) {
                continue;
            }
            seen_events.insert(event_key);

            let event = TimelineEvent {
                timestamp: entry.timestamp.clone(),
                event_type: entry.level.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
                description: self.create_event_description(&entry.message),
                component: self.extract_component_from_message(&entry.message),
                severity: self.assess_event_severity(&entry.level, &entry.message),
                causality: self.assess_causality(&entry.message),
            };

            timeline.push(event);
        }

        // Sort by timestamp and limit
        timeline.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        timeline.truncate(config.max_timeline_events);
        timeline
    }

    /// Extract stack traces from raw log lines
    fn extract_stack_traces(&self, raw_lines: &[String], config: &DigestConfig) -> Vec<StackTrace> {
        let mut stack_traces: Vec<StackTrace> = Vec::new();
        let mut current_trace: Option<Vec<String>> = None;
        let mut trace_start_timestamp: Option<String> = None;

        for (i, line) in raw_lines.iter().enumerate() {
            // Detect start of stack trace (common patterns)
            if self.is_stack_trace_start(line) {
                if let Some(trace_lines) = current_trace.take() {
                    // Process previous trace
                    if let Some(trace) = self.build_stack_trace(trace_lines, trace_start_timestamp.clone(), i) {
                        stack_traces.push(trace);
                    }
                }
                // Start new trace
                current_trace = Some(vec![line.clone()]);
                trace_start_timestamp = self.extract_timestamp_from_line(line);
            } else if current_trace.is_some() && self.is_stack_trace_continuation(line) {
                // Continue current trace
                if let Some(ref mut trace_lines) = current_trace {
                    trace_lines.push(line.clone());
                }
            } else if current_trace.is_some() {
                // End of current trace
                if let Some(trace_lines) = current_trace.take() {
                    if let Some(trace) = self.build_stack_trace(trace_lines, trace_start_timestamp.clone(), i) {
                        stack_traces.push(trace);
                    }
                }
                trace_start_timestamp = None;
            }
        }

        // Process final trace if exists
        if let Some(trace_lines) = current_trace {
            if let Some(trace) = self.build_stack_trace(trace_lines, trace_start_timestamp, raw_lines.len()) {
                stack_traces.push(trace);
            }
        }

        // Deduplicate and limit
        stack_traces = self.deduplicate_stack_traces(stack_traces);
        stack_traces.truncate(config.max_stack_traces);
        stack_traces
    }

    /// Extract context windows around critical errors
    fn extract_context_windows(&self, raw_lines: &[String], entries: &[LogEntry], config: &DigestConfig) -> Vec<ContextWindow> {
        let mut context_windows: Vec<ContextWindow> = Vec::new();
        let mut processed_lines: HashSet<usize> = HashSet::new();

        for entry in entries {
            if entry.level.as_deref() != Some("ERROR") {
                continue;
            }

            // Find the line number for this entry in raw_lines
            if let Some(line_num) = self.find_line_number_for_entry(raw_lines, entry) {
                if processed_lines.contains(&line_num) {
                    continue;
                }
                processed_lines.insert(line_num);

                let context = self.create_context_window(raw_lines, line_num, entry, config);
                context_windows.push(context);

                if context_windows.len() >= config.max_context_windows {
                    break;
                }
            }
        }

        context_windows
    }

    /// Calculate overall incident severity
    fn calculate_severity(&self, entries: &[LogEntry]) -> String {
        let mut error_count = 0;
        let mut warn_count = 0;
        let mut has_fatal = false;

        for entry in entries {
            match entry.level.as_deref() {
                Some("ERROR") => error_count += 1,
                Some("FATAL") => { has_fatal = true; error_count += 1; },
                Some("WARN") => warn_count += 1,
                _ => {}
            }
        }

        if has_fatal || error_count > 50 {
            "CRITICAL".to_string()
        } else if error_count > 10 {
            "HIGH".to_string()
        } else if error_count > 0 || warn_count > 20 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }

    /// Extract recommendations from AI analysis
    fn extract_recommendations(&self, analysis: &AnalysisResponse) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Use existing recommendations from the analysis
        recommendations.extend(analysis.recommendations.clone());

        // Parse additional recommendations from the AI analysis text
        let text = format!("{} {}",
            analysis.sequence_of_events,
            analysis.root_cause.description
        );

        // Look for common recommendation patterns
        let recommendation_patterns = [
            "should check",
            "recommend",
            "suggest",
            "try to",
            "need to",
            "consider",
            "might want to",
            "solution would be",
        ];

        for line in text.lines() {
            for pattern in &recommendation_patterns {
                if line.to_lowercase().contains(pattern) {
                    recommendations.push(line.trim().to_string());
                    break;
                }
            }
        }

        // Add some generic recommendations based on common issues
        if text.to_lowercase().contains("database") {
            recommendations.push("Check database connection pool settings".to_string());
        }
        if text.to_lowercase().contains("timeout") {
            recommendations.push("Review timeout configurations".to_string());
        }
        if text.to_lowercase().contains("memory") {
            recommendations.push("Analyze memory usage and garbage collection".to_string());
        }

        recommendations.truncate(5); // Limit to 5 recommendations
        recommendations
    }

    /// Suggest areas for deeper investigation
    fn suggest_investigation_areas(&self, entries: &[LogEntry]) -> Vec<String> {
        let mut areas = Vec::new();
        let mut components: HashSet<String> = HashSet::new();

        for entry in entries {
            if let Some(component) = self.extract_component_from_message(&entry.message) {
                components.insert(component);
            }
        }

        for component in components {
            areas.push(format!("Review {} component logs and configurations", component));
        }

        areas.push("Check system resources (CPU, memory, disk)".to_string());
        areas.push("Review recent deployments or configuration changes".to_string());
        areas.push("Analyze network connectivity and dependencies".to_string());

        areas.truncate(5);
        areas
    }

    /// Calculate log statistics
    fn calculate_statistics(&self, entries: &[LogEntry], raw_lines: &[String]) -> LogStatistics {
        let mut level_breakdown: HashMap<String, usize> = HashMap::new();
        let mut components: HashSet<String> = HashSet::new();
        let mut timestamps: Vec<String> = Vec::new();

        for entry in entries {
            // Count by level
            let level = entry.level.as_deref().unwrap_or("UNKNOWN");
            *level_breakdown.entry(level.to_string()).or_insert(0) += 1;

            // Collect components
            if let Some(component) = self.extract_component_from_message(&entry.message) {
                components.insert(component);
            }

            // Collect timestamps
            if let Some(timestamp) = &entry.timestamp {
                timestamps.push(timestamp.clone());
            }
        }

        // Calculate time range
        timestamps.sort();
        let time_range = TimeRange {
            start_time: timestamps.first().cloned(),
            end_time: timestamps.last().cloned(),
            duration_seconds: None, // TODO: Calculate actual duration
        };

        LogStatistics {
            total_lines: raw_lines.len(),
            filtered_lines: entries.len(),
            analyzed_lines: entries.len(), // TODO: Use actual slimmed count
            level_breakdown,
            time_range,
            unique_components: components.into_iter().collect(),
            unique_error_patterns: 0, // TODO: Calculate from pattern analysis
        }
    }

    // === Helper Methods ===

    /// Classify error type from message
    fn classify_error_type(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();

        if message_lower.contains("timeout") {
            "Timeout".to_string()
        } else if message_lower.contains("connection") && message_lower.contains("failed") {
            "ConnectionFailure".to_string()
        } else if message_lower.contains("null") && message_lower.contains("pointer") {
            "NullPointerException".to_string()
        } else if message_lower.contains("database") {
            "DatabaseError".to_string()
        } else if message_lower.contains("authentication") || message_lower.contains("unauthorized") {
            "AuthenticationError".to_string()
        } else if message_lower.contains("memory") || message_lower.contains("outofmemory") {
            "MemoryError".to_string()
        } else {
            // Extract first few words as error type
            message.split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(50)
                .collect()
        }
    }

    /// Extract component name from log message
    fn extract_component_from_message(&self, message: &str) -> Option<String> {
        // Look for common patterns: [component], service.method, component:
        let patterns = [
            r"\[([^\]]+)\]",
            r"(\w+)\.\w+",
            r"(\w+):",
            r"(\w+Service)",
            r"(\w+Controller)",
        ];

        for pattern in &patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(message) {
                    if let Some(component) = captures.get(1) {
                        let comp_str = component.as_str().to_string();
                        if comp_str.len() > 2 && comp_str.len() < 30 {
                            return Some(comp_str);
                        }
                    }
                }
            }
        }

        None
    }

    /// Create event description from message
    fn create_event_description(&self, message: &str) -> String {
        // Truncate and clean up message for timeline
        message.chars().take(100).collect::<String>()
            .trim()
            .to_string()
    }

    /// Assess event severity
    fn assess_event_severity(&self, level: &Option<String>, message: &str) -> String {
        match level.as_deref() {
            Some("FATAL") => "CRITICAL".to_string(),
            Some("ERROR") => {
                if message.to_lowercase().contains("critical") ||
                   message.to_lowercase().contains("fatal") {
                    "CRITICAL".to_string()
                } else {
                    "HIGH".to_string()
                }
            },
            Some("WARN") => "MEDIUM".to_string(),
            Some("INFO") => "LOW".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }

    /// Assess causality (cause/effect/symptom)
    fn assess_causality(&self, message: &str) -> Option<String> {
        let message_lower = message.to_lowercase();

        if message_lower.contains("caused by") || message_lower.contains("due to") {
            Some("effect".to_string())
        } else if message_lower.contains("timeout") || message_lower.contains("failed") {
            Some("symptom".to_string())
        } else {
            None
        }
    }

    /// Check if line starts a stack trace
    fn is_stack_trace_start(&self, line: &str) -> bool {
        let patterns = [
            "Exception in thread",
            "Caused by:",
            "java.lang.",
            "    at ",
            "Traceback",
            "RuntimeError:",
            "Error:",
        ];

        patterns.iter().any(|pattern| line.contains(pattern))
    }

    /// Check if line continues a stack trace
    fn is_stack_trace_continuation(&self, line: &str) -> bool {
        line.trim_start().starts_with("at ") ||
        line.trim_start().starts_with("... ") ||
        line.trim_start().starts_with("Caused by:") ||
        line.trim().starts_with("File \"")
    }

    /// Build stack trace from lines
    fn build_stack_trace(&self, lines: Vec<String>, timestamp: Option<String>, line_num: usize) -> Option<StackTrace> {
        if lines.is_empty() {
            return None;
        }

        let full_trace = lines.join("\n");
        let root_exception = lines.first()?.clone();
        let key_methods = self.extract_key_methods_from_trace(&lines);

        Some(StackTrace {
            trace_id: format!("trace_{}", line_num),
            full_trace,
            root_exception,
            key_methods,
            timestamp,
            frequency: 1,
        })
    }

    /// Extract key methods from stack trace
    fn extract_key_methods_from_trace(&self, lines: &[String]) -> Vec<String> {
        let mut methods = Vec::new();

        for line in lines {
            if let Ok(regex) = regex::Regex::new(r"at\s+([^(]+)\(") {
                if let Some(captures) = regex.captures(line) {
                    if let Some(method) = captures.get(1) {
                        methods.push(method.as_str().to_string());
                    }
                }
            }
        }

        methods.truncate(5);
        methods
    }

    /// Deduplicate stack traces
    fn deduplicate_stack_traces(&self, traces: Vec<StackTrace>) -> Vec<StackTrace> {
        let mut deduplicated: Vec<StackTrace> = Vec::new();
        let mut seen_traces: HashSet<String> = HashSet::new();

        for trace in traces {
            // Create a signature for deduplication
            let signature = format!("{}_{}",
                trace.root_exception,
                trace.key_methods.join(",")
            );

            if !seen_traces.contains(&signature) {
                seen_traces.insert(signature);
                deduplicated.push(trace);
            }
        }

        deduplicated
    }

    /// Find line number for log entry in raw lines
    fn find_line_number_for_entry(&self, raw_lines: &[String], entry: &LogEntry) -> Option<usize> {
        for (i, line) in raw_lines.iter().enumerate() {
            if line.contains(&entry.message) {
                return Some(i);
            }
        }
        None
    }

    /// Create context window around error
    fn create_context_window(&self, raw_lines: &[String], line_num: usize, entry: &LogEntry, config: &DigestConfig) -> ContextWindow {
        let start = line_num.saturating_sub(config.context_lines);
        let end = (line_num + config.context_lines + 1).min(raw_lines.len());

        let before_lines = raw_lines[start..line_num].to_vec();
        let after_lines = if line_num + 1 < end {
            raw_lines[line_num + 1..end].to_vec()
        } else {
            Vec::new()
        };

        ContextWindow {
            related_error: self.classify_error_type(&entry.message),
            before_lines,
            error_line: raw_lines[line_num].clone(),
            after_lines,
            line_number: Some(line_num + 1), // 1-based line numbers
            timestamp: entry.timestamp.clone(),
        }
    }

    /// Extract timestamp from log line
    fn extract_timestamp_from_line(&self, line: &str) -> Option<String> {
        // Simple timestamp extraction - could be more sophisticated
        if let Ok(regex) = regex::Regex::new(r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})") {
            if let Some(captures) = regex.captures(line) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }
}

impl Default for LogLens {
    fn default() -> Self {
        Self::new().expect("Failed to create LogLens with default config")
    }
}