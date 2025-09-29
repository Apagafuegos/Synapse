use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LogStats {
    pub total_lines: usize,
    pub parsed_entries: usize,
    pub levels: HashMap<String, usize>,
    pub components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FilterOptions {
    pub level: Option<String>,
    pub pattern: Option<String>,
    pub case_sensitive: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ParseOptions {
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeOptions {
    pub level: Option<String>,
    pub provider: Option<String>,
    pub slim_logs: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchPatternsRequest {
    #[schemars(description = "Array of log lines to search")]
    pub logs: Vec<String>,
    #[schemars(description = "Array of patterns to search for")]
    pub patterns: Vec<String>,
    #[schemars(description = "Whether search is case sensitive")]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PatternMatch {
    #[schemars(description = "The log entry that matched")]
    pub log_entry: crate::input::LogEntry,
    #[schemars(description = "List of patterns that matched in this entry")]
    pub matched_patterns: Vec<String>,
    #[schemars(description = "Original line number")]
    pub line_number: usize,
}