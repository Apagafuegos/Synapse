// Incident Digest - MCP-optimized structures for large log analysis
//
// This module provides structured data types for creating incident digests
// that are optimized for MCP context limits while preserving essential
// debugging information from large log files.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main incident digest structure optimized for MCP integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDigest {
    /// Unique identifier for this incident analysis
    pub incident_id: String,

    /// When this analysis was performed
    pub timestamp: DateTime<Utc>,

    /// Overall severity assessment (CRITICAL, HIGH, MEDIUM, LOW)
    pub severity: String,

    /// Core incident data (MCP-optimized sizes)
    pub critical_errors: Vec<CriticalError>,
    pub error_timeline: Vec<TimelineEvent>,
    pub stack_traces: Vec<StackTrace>,
    pub context_snippets: Vec<ContextWindow>,

    /// Analysis results from AI processing
    pub root_cause_analysis: String,
    pub recommended_actions: Vec<String>,
    pub investigation_areas: Vec<String>,

    /// Metadata about the log processing
    pub log_stats: LogStatistics,
    pub processing_time_ms: u64,
}

/// Critical error information with frequency and impact data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalError {
    /// Type/category of error (e.g., "DatabaseTimeout", "NullPointer")
    pub error_type: String,

    /// Representative error message
    pub message: String,

    /// How many times this error occurred
    pub frequency: u32,

    /// When this error first appeared
    pub first_occurrence: Option<String>,

    /// When this error last appeared
    pub last_occurrence: Option<String>,

    /// Components/services affected by this error
    pub affected_components: Vec<String>,

    /// Confidence score for this error classification (0.0-1.0)
    pub confidence: f32,
}

/// Timeline event representing key moments in incident progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// When this event occurred
    pub timestamp: Option<String>,

    /// Type of event (ERROR, WARN, INFO, STATE_CHANGE, etc.)
    pub event_type: String,

    /// Brief description of what happened
    pub description: String,

    /// Component or service that generated this event
    pub component: Option<String>,

    /// Severity of this specific event
    pub severity: String,

    /// Whether this appears to be a cause or effect
    pub causality: Option<String>, // "cause", "effect", "symptom"
}

/// Complete stack trace information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrace {
    /// Unique identifier for this stack trace
    pub trace_id: String,

    /// The complete stack trace text
    pub full_trace: String,

    /// Root exception or error that caused this trace
    pub root_exception: String,

    /// Key methods/functions in the call stack
    pub key_methods: Vec<String>,

    /// When this stack trace occurred
    pub timestamp: Option<String>,

    /// How many times this exact stack trace appeared
    pub frequency: u32,
}

/// Context window showing code/log lines around critical errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// What error this context is related to
    pub related_error: String,

    /// Lines before the error (up to 5 lines)
    pub before_lines: Vec<String>,

    /// The actual error line
    pub error_line: String,

    /// Lines after the error (up to 5 lines)
    pub after_lines: Vec<String>,

    /// Line number in original log (if available)
    pub line_number: Option<usize>,

    /// Timestamp of the error line
    pub timestamp: Option<String>,
}

/// Statistics about the log processing and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatistics {
    /// Total number of log lines processed
    pub total_lines: usize,

    /// Number of lines after filtering by level
    pub filtered_lines: usize,

    /// Number of lines sent to AI analysis
    pub analyzed_lines: usize,

    /// Breakdown by log level
    pub level_breakdown: HashMap<String, usize>,

    /// Time range covered by the logs
    pub time_range: TimeRange,

    /// Unique components/services identified
    pub unique_components: Vec<String>,

    /// Total unique error patterns found
    pub unique_error_patterns: usize,
}

/// Time range information for the analyzed logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Earliest timestamp found in logs
    pub start_time: Option<String>,

    /// Latest timestamp found in logs
    pub end_time: Option<String>,

    /// Duration covered (in seconds, if calculable)
    pub duration_seconds: Option<f64>,
}

/// Configuration for incident digest creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestConfig {
    /// Maximum number of critical errors to include (default: 5)
    pub max_critical_errors: usize,

    /// Maximum number of timeline events (default: 10)
    pub max_timeline_events: usize,

    /// Maximum number of stack traces (default: 3)
    pub max_stack_traces: usize,

    /// Maximum number of context windows (default: 5)
    pub max_context_windows: usize,

    /// Lines to include before/after errors in context windows (default: 3)
    pub context_lines: usize,

    /// Minimum frequency for errors to be considered "critical" (default: 1)
    pub min_error_frequency: u32,

    /// Whether to include low-severity events in timeline (default: false)
    pub include_low_severity: bool,
}

impl Default for DigestConfig {
    fn default() -> Self {
        Self {
            max_critical_errors: 5,
            max_timeline_events: 10,
            max_stack_traces: 3,
            max_context_windows: 5,
            context_lines: 3,
            min_error_frequency: 1,
            include_low_severity: false,
        }
    }
}

impl IncidentDigest {
    /// Create a new incident digest with the given ID
    pub fn new(incident_id: String) -> Self {
        Self {
            incident_id,
            timestamp: Utc::now(),
            severity: "UNKNOWN".to_string(),
            critical_errors: Vec::new(),
            error_timeline: Vec::new(),
            stack_traces: Vec::new(),
            context_snippets: Vec::new(),
            root_cause_analysis: String::new(),
            recommended_actions: Vec::new(),
            investigation_areas: Vec::new(),
            log_stats: LogStatistics::default(),
            processing_time_ms: 0,
        }
    }

    /// Get a summary suitable for MCP context inclusion
    pub fn get_mcp_summary(&self) -> String {
        format!(
            "Incident {} ({}): {} critical errors, {} timeline events. Severity: {}. Root cause: {}",
            self.incident_id,
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.critical_errors.len(),
            self.error_timeline.len(),
            self.severity,
            if self.root_cause_analysis.len() > 100 {
                format!("{}...", &self.root_cause_analysis[..100])
            } else {
                self.root_cause_analysis.clone()
            }
        )
    }

    /// Estimate the size of this digest when serialized (for MCP context planning)
    pub fn estimated_size(&self) -> usize {
        // Rough estimation for context planning
        let json_str = serde_json::to_string(self).unwrap_or_default();
        json_str.len()
    }
}

impl Default for LogStatistics {
    fn default() -> Self {
        Self {
            total_lines: 0,
            filtered_lines: 0,
            analyzed_lines: 0,
            level_breakdown: HashMap::new(),
            time_range: TimeRange {
                start_time: None,
                end_time: None,
                duration_seconds: None,
            },
            unique_components: Vec::new(),
            unique_error_patterns: 0,
        }
    }
}