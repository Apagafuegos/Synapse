use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::types::LogEntry;

// Import WASM functions from loglens-wasm
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["loglens_wasm"])]
    fn parse_log_preview(content: &str, max_lines: usize) -> String;

    #[wasm_bindgen(js_namespace = ["loglens_wasm"])]
    fn filter_logs_by_level(content: &str, level: &str) -> String;

    #[wasm_bindgen(js_namespace = ["loglens_wasm"])]
    fn count_log_levels(content: &str) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPreview {
    pub entries: Vec<LogEntry>,
    pub total_lines: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevelCounts {
    pub error: usize,
    pub warn: usize,
    pub info: usize,
    pub debug: usize,
    pub total: usize,
}

pub struct WasmBridge;

impl WasmBridge {
    pub fn new() -> Self {
        Self
    }

    /// Parse log content and return a preview with limited entries
    pub fn parse_preview(&self, content: &str, max_lines: usize) -> Result<LogPreview, String> {
        let result = parse_log_preview(content, max_lines);
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse WASM result: {}", e))
    }

    /// Filter logs by severity level
    pub fn filter_by_level(&self, content: &str, level: &str) -> Result<Vec<LogEntry>, String> {
        let result = filter_logs_by_level(content, level);
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse WASM result: {}", e))
    }

    /// Count log entries by level
    pub fn count_levels(&self, content: &str) -> Result<LogLevelCounts, String> {
        let result = count_log_levels(content);
        serde_json::from_str(&result).map_err(|e| format!("Failed to parse WASM result: {}", e))
    }

    /// Validate file content and return basic statistics
    pub fn validate_file(&self, content: &str) -> Result<FileValidation, String> {
        // Basic validation using WASM functions
        let counts = self.count_levels(content)?;
        let preview = self.parse_preview(content, 10)?;

        let size_mb = content.len() as f64 / (1024.0 * 1024.0);
        let estimated_processing_time = self.estimate_processing_time(content.lines().count());

        Ok(FileValidation {
            is_valid: counts.total > 0,
            line_count: content.lines().count(),
            size_mb,
            has_structured_logs: preview.entries.iter().any(|e| e.timestamp.is_some()),
            log_levels: counts,
            estimated_processing_time,
            sample_entries: preview.entries,
        })
    }

    /// Estimate processing time based on line count
    fn estimate_processing_time(&self, line_count: usize) -> u32 {
        // Simple estimation: 1000 lines per second
        ((line_count as f64 / 1000.0).ceil() as u32).max(1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub total_lines: usize,
    pub total_chars: usize,
    pub avg_line_length: usize,
    pub estimated_processing_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidation {
    pub is_valid: bool,
    pub line_count: usize,
    pub size_mb: f64,
    pub has_structured_logs: bool,
    pub log_levels: LogLevelCounts,
    pub estimated_processing_time: u32, // seconds
    pub sample_entries: Vec<LogEntry>,
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new()
    }
}
