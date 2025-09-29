use super::{AnalysisReport, OutputGenerator};
use anyhow::Result;
use serde_json;

pub struct JsonOutput;

impl OutputGenerator for JsonOutput {
    fn generate(&self, report: &AnalysisReport) -> Result<String> {
        Ok(serde_json::to_string_pretty(report)?)
    }
    
    fn file_extension(&self) -> &str {
        "json"
    }
}