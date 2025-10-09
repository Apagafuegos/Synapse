use super::{AnalysisReport, OutputGenerator};
use crate::ai_provider::AnalysisResponse;
use anyhow::Result;
use askama::Template;

#[derive(Template)]
#[template(path = "report_template.html")]
struct ReportTemplate {
    input_source: String,
    provider: String,
    level: String,
    timestamp: String,
    log_count: usize,
    analysis: AnalysisResponse,
    log_entries: Vec<DisplayLogEntry>,
}

#[derive(Clone)]
struct DisplayLogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct HtmlOutput;

impl OutputGenerator for HtmlOutput {
    fn generate(&self, report: &AnalysisReport) -> Result<String> {
        let display_entries: Vec<DisplayLogEntry> = report.logs.iter()
            .map(|entry| DisplayLogEntry {
                timestamp: entry.timestamp.as_deref().unwrap_or("No timestamp").to_string(),
                level: entry.level.as_deref().unwrap_or("UNKNOWN").to_string(),
                message: entry.message.clone(),
            })
            .collect();

        let template = ReportTemplate {
            input_source: report.metadata.input_source.clone(),
            provider: report.metadata.provider.clone(),
            level: report.metadata.log_level.clone(),
            timestamp: report.metadata.timestamp.clone(),
            log_count: report.logs.len(),
            analysis: report.analysis.clone(),
            log_entries: display_entries,
        };

        template.render().map_err(|e| anyhow::anyhow!("Template rendering failed: {}", e))
    }

    fn file_extension(&self) -> &str {
        "html"
    }
}

