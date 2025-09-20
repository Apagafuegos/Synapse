use crate::model::LogEntry;
use crate::analyzer::LogSummary;
use colored::*;
use serde_json;

pub enum OutputFormat {
    Text,
    Json,
}

pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    pub fn new(format: &str) -> Self {
        let output_format = match format {
            "json" => OutputFormat::Json,
            _ => OutputFormat::Text,
        };
        Self { format: output_format }
    }
    
    pub fn write_entry(&self, entry: &LogEntry) {
        match self.format {
            OutputFormat::Text => self.write_text_entry(entry),
            OutputFormat::Json => self.write_json_entry(entry),
        }
    }
    
    pub fn write_summary(&self, summary: &LogSummary) {
        match self.format {
            OutputFormat::Text => self.write_text_summary(summary),
            OutputFormat::Json => self.write_json_summary(summary),
        }
    }
    
    fn write_text_entry(&self, entry: &LogEntry) {
        let level_colored = match entry.level {
            crate::model::LogLevel::Error => entry.level.to_string().red(),
            crate::model::LogLevel::Warn => entry.level.to_string().yellow(),
            crate::model::LogLevel::Info => entry.level.to_string().green(),
            crate::model::LogLevel::Debug => entry.level.to_string().blue(),
            crate::model::LogLevel::Trace => entry.level.to_string().purple(),
            crate::model::LogLevel::Unknown => entry.level.to_string().normal(),
        };
        
        println!("{} [{}] {}", 
            entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().cyan(),
            level_colored,
            entry.message
        );
    }
    
    fn write_json_entry(&self, entry: &LogEntry) {
        if let Ok(json) = serde_json::to_string(entry) {
            println!("{}", json);
        }
    }
    
    fn write_text_summary(&self, summary: &LogSummary) {
        println!("{}", "=== Log Summary ===".bold());
        println!("Total entries: {}", summary.total_entries);
        
        println!("\n{}", "Level Distribution:".bold());
        for (level, count) in &summary.level_counts {
            let percentage = if summary.total_entries > 0 {
                (count * 100) / summary.total_entries
            } else {
                0
            };
            println!("  {}: {} ({}%)", level, count, percentage);
        }
        
        if !summary.top_errors.is_empty() {
            println!("\n{}", "Top Error Messages:".bold());
            for (message, count) in &summary.top_errors {
                println!("  {} ({} times)", message, count);
            }
        }
        
        if let Some((start, end)) = &summary.time_range {
            println!("\n{}", "Time Range:".bold());
            println!("  Start: {}", start.format("%Y-%m-%d %H:%M:%S"));
            println!("  End:   {}", end.format("%Y-%m-%d %H:%M:%S"));
        }
    }
    
    fn write_json_summary(&self, summary: &LogSummary) {
        if let Ok(json) = serde_json::to_string(summary) {
            println!("{}", json);
        }
    }
}