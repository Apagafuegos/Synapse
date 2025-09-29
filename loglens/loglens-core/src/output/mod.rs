pub mod html;
pub mod json;
pub mod markdown;
pub mod console;

use crate::ai_provider::AnalysisResponse;
use crate::input::LogEntry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use console::ConsoleOutput;
use html::HtmlOutput;
use json::JsonOutput;
use markdown::MarkdownOutput;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub metadata: ReportMetadata,
    pub analysis: AnalysisResponse,
    pub logs: Vec<ProcessedLogEntry>,
    pub stats: ReportStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub timestamp: String,
    pub provider: String,
    pub total_logs: usize,
    pub filtered_logs: usize,
    pub log_level: String,
    pub input_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedLogEntry {
    pub original_index: usize,
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: String,
    pub was_slimmed: bool,
    pub was_consolidated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub unique_messages: usize,
    pub consolidated_duplicates: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Console,
    Html,
    Json,
    Markdown,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "console" => Some(OutputFormat::Console),
            "html" => Some(OutputFormat::Html),
            "json" => Some(OutputFormat::Json),
            "markdown" | "md" => Some(OutputFormat::Markdown),
            _ => None,
        }
    }
    
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            OutputFormat::Console => vec![],
            OutputFormat::Html => vec!["html"],
            OutputFormat::Json => vec!["json"],
            OutputFormat::Markdown => vec!["md", "markdown"],
        }
    }
}

pub trait OutputGenerator {
    fn generate(&self, report: &AnalysisReport) -> Result<String>;
    fn file_extension(&self) -> &str;
}

pub fn generate_report(
    analysis: AnalysisResponse,
    logs: Vec<LogEntry>,
    provider: &str,
    log_level: &str,
    input_source: &str,
    format: OutputFormat,
) -> Result<String> {
    let metadata = ReportMetadata {
        timestamp: chrono::Utc::now().to_rfc3339(),
        provider: provider.to_string(),
        total_logs: logs.len(),
        filtered_logs: logs.len(),
        log_level: log_level.to_string(),
        input_source: input_source.to_string(),
    };

    let processed_logs: Vec<ProcessedLogEntry> = logs.into_iter()
        .enumerate()
        .map(|(i, entry)| ProcessedLogEntry {
            original_index: i,
            timestamp: entry.timestamp,
            level: entry.level,
            message: entry.message,
            was_slimmed: false, // TODO: Track this during slimming
            was_consolidated: false, // TODO: Track this during consolidation
        })
        .collect();

    let stats = calculate_stats(&processed_logs);

    let report = AnalysisReport {
        metadata,
        analysis,
        logs: processed_logs,
        stats,
    };

    match format {
        OutputFormat::Console => ConsoleOutput.generate(&report),
        OutputFormat::Html => HtmlOutput.generate(&report),
        OutputFormat::Json => JsonOutput.generate(&report),
        OutputFormat::Markdown => MarkdownOutput.generate(&report),
    }
}

fn calculate_stats(logs: &[ProcessedLogEntry]) -> ReportStats {
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut debug_count = 0;
    
    for log in logs {
        match log.level.as_ref().map(|s| s.as_str()) {
            Some("ERROR") => error_count += 1,
            Some("WARN") | Some("WARNING") => warning_count += 1,
            Some("INFO") => info_count += 1,
            Some("DEBUG") => debug_count += 1,
            _ => {}
        }
    }
    
    let unique_messages = logs.iter()
        .map(|log| &log.message)
        .collect::<std::collections::HashSet<_>>()
        .len();
    
    ReportStats {
        error_count,
        warning_count,
        info_count,
        debug_count,
        unique_messages,
        consolidated_duplicates: 0, // TODO: Calculate this during consolidation
    }
}

pub fn save_report(content: &str, output_path: &PathBuf) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, content)?;
    Ok(())
}