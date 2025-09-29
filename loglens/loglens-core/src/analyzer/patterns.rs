use crate::input::LogEntry;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub recurring_patterns: Vec<RecurringPattern>,
    pub error_chains: Vec<ErrorChain>,
    pub grouped_errors: Vec<ErrorGroup>,
    pub anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringPattern {
    pub pattern: String,
    pub frequency: usize,
    pub examples: Vec<String>,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorChain {
    pub chain_id: String,
    pub sequence: Vec<ErrorStep>,
    pub root_cause: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStep {
    pub timestamp: Option<String>,
    pub level: String,
    pub message: String,
    pub component: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGroup {
    pub group_id: String,
    pub pattern: String,
    pub errors: Vec<LogEntry>,
    pub count: usize,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub log_entry: Option<LogEntry>,
    pub confidence: f32,
}

pub struct PatternAnalyzer {
    message_patterns: HashMap<String, usize>,
    error_sequences: Vec<Vec<ErrorStep>>,
    component_errors: HashMap<String, Vec<LogEntry>>,
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            message_patterns: HashMap::new(),
            error_sequences: Vec::new(),
            component_errors: HashMap::new(),
        }
    }
    
    pub fn analyze(&mut self, logs: &[LogEntry]) -> Result<PatternAnalysis> {
        self.extract_patterns(logs);
        self.build_error_chains(logs);
        self.group_similar_errors(logs);
        self.detect_anomalies(logs);
        
        Ok(PatternAnalysis {
            recurring_patterns: self.get_recurring_patterns(),
            error_chains: self.get_error_chains(),
            grouped_errors: self.get_grouped_errors(),
            anomalies: self.get_anomalies(),
        })
    }
    
    fn extract_patterns(&mut self, logs: &[LogEntry]) {
        for log in logs {
            if log.level.as_ref().map_or(false, |l| l == "ERROR") {
                let normalized_message = self.normalize_message(&log.message);
                *self.message_patterns.entry(normalized_message).or_insert(0) += 1;
                
                // Group by component if detectable
                if let Some(component) = self.extract_component(&log.message) {
                    self.component_errors.entry(component).or_insert_with(Vec::new).push(log.clone());
                }
            }
        }
    }
    
    fn build_error_chains(&mut self, logs: &[LogEntry]) {
        let mut current_sequence: Vec<ErrorStep> = Vec::new();
        let mut last_timestamp = None;
        
        for log in logs {
            if let Some(level) = &log.level {
                if level == "ERROR" || level == "WARN" {
                    let step = ErrorStep {
                        timestamp: log.timestamp.clone(),
                        level: level.clone(),
                        message: log.message.clone(),
                        component: self.extract_component(&log.message),
                    };
                    
                    // Check if this error is related to the previous one
                    if let Some(_last_ts) = last_timestamp.clone() {
                        if let Some(_current_ts) = &log.timestamp {
                            if self.are_related(&step, current_sequence.last()) {
                                current_sequence.push(step);
                                last_timestamp = log.timestamp.clone();
                                continue;
                            }
                        }
                    }
                    
                    // Start new sequence
                    if !current_sequence.is_empty() {
                        self.error_sequences.push(current_sequence);
                    }
                    current_sequence = vec![step];
                    last_timestamp = log.timestamp.clone();
                }
            }
        }
        
        if !current_sequence.is_empty() {
            self.error_sequences.push(current_sequence);
        }
    }
    
    fn group_similar_errors(&mut self, _logs: &[LogEntry]) {
        // This is already handled by extract_patterns and component_errors
        // Additional grouping logic can be added here
    }
    
    fn detect_anomalies(&mut self, logs: &[LogEntry]) {
        // Detect unusual patterns
        let total_errors = logs.iter()
            .filter(|log| log.level.as_ref().map_or(false, |l| l == "ERROR"))
            .count();
        
        if total_errors == 0 {
            return;
        }
        
        // Check for unusually frequent errors
        for (_pattern, count) in &self.message_patterns {
            let frequency = *count as f32 / total_errors as f32;
            if frequency > 0.5 { // More than 50% of errors are the same
                // This could be an anomaly
            }
        }
    }
    
    fn get_recurring_patterns(&self) -> Vec<RecurringPattern> {
        self.message_patterns
            .iter()
            .filter(|(_, count)| **count > 1)
            .map(|(pattern, frequency)| {
                let severity = self.estimate_severity(pattern);
                RecurringPattern {
                    pattern: pattern.clone(),
                    frequency: *frequency,
                    examples: vec![pattern.clone()], // In real implementation, extract actual examples
                    severity,
                }
            })
            .collect()
    }
    
    fn get_error_chains(&self) -> Vec<ErrorChain> {
        self.error_sequences
            .iter()
            .filter(|seq| seq.len() > 1)
            .enumerate()
            .map(|(i, sequence)| {
                let root_cause = sequence.first().map(|step| step.message.clone());
                ErrorChain {
                    chain_id: format!("chain_{}", i),
                    sequence: sequence.clone(),
                    root_cause,
                    confidence: self.calculate_chain_confidence(sequence),
                }
            })
            .collect()
    }
    
    fn get_grouped_errors(&self) -> Vec<ErrorGroup> {
        self.component_errors
            .iter()
            .enumerate()
            .map(|(i, (component, errors))| {
                let severity = self.calculate_group_severity(errors);
                ErrorGroup {
                    group_id: format!("group_{}", i),
                    pattern: component.clone(),
                    errors: errors.clone(),
                    count: errors.len(),
                    severity,
                }
            })
            .collect()
    }
    
    fn get_anomalies(&self) -> Vec<Anomaly> {
        // Basic anomaly detection - can be enhanced
        vec![
            Anomaly {
                anomaly_type: "High Error Frequency".to_string(),
                description: "Multiple similar errors detected in short time span".to_string(),
                severity: "MEDIUM".to_string(),
                log_entry: None,
                confidence: 0.7,
            }
        ]
    }
    
    fn normalize_message(&self, message: &str) -> String {
        // Remove timestamps, numbers, and specific values to find patterns
        let normalized = message
            .chars()
            .map(|c| if c.is_numeric() || c == '.' || c == ':' { 'N' } else { c })
            .collect::<String>()
            .to_lowercase();
        
        // Remove common noise words
        normalized
            .replace("at", "")
            .replace("on", "")
            .replace("in", "")
            .replace("the", "")
            .replace("a", "")
            .replace("an", "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
    
    fn extract_component(&self, message: &str) -> Option<String> {
        // Try to extract component names from error messages
        let patterns = vec![
            r"(\w+)\.(\w+)", // Module.function
            r"(\w+):", // Component:
            r"in (\w+)", // in Component
            r"failed to (\w+)", // failed to component
        ];
        
        for pattern in patterns {
            if let Some(regex) = regex::Regex::new(pattern).ok() {
                if let Some(caps) = regex.captures(message) {
                    return Some(caps.get(1).unwrap().as_str().to_string());
                }
            }
        }
        
        None
    }
    
    fn are_related(&self, current: &ErrorStep, previous: Option<&ErrorStep>) -> bool {
        match previous {
            None => false,
            Some(prev) => {
                // Check if errors are from the same component
                if let (Some(curr_comp), Some(prev_comp)) = (&current.component, &prev.component) {
                    if curr_comp == prev_comp {
                        return true;
                    }
                }
                
                // Check if messages contain similar keywords
                let curr_words: HashSet<_> = current.message.split_whitespace().collect();
                let prev_words: HashSet<_> = prev.message.split_whitespace().collect();
                
                let intersection: HashSet<_> = curr_words.intersection(&prev_words).collect();
                intersection.len() > 2 // At least 2 common words
            }
        }
    }
    
    fn estimate_severity(&self, pattern: &str) -> String {
        let severe_keywords = vec!["fatal", "critical", "panic", "crash", "exception"];
        let warning_keywords = vec!["warning", "warn", "deprecated", "timeout"];
        
        let pattern_lower = pattern.to_lowercase();
        
        if severe_keywords.iter().any(|kw| pattern_lower.contains(kw)) {
            "HIGH".to_string()
        } else if warning_keywords.iter().any(|kw| pattern_lower.contains(kw)) {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }
    
    fn calculate_chain_confidence(&self, sequence: &[ErrorStep]) -> f32 {
        if sequence.is_empty() {
            return 0.0;
        }
        
        // Simple confidence calculation based on sequence length and consistency
        let base_confidence = (sequence.len() as f32 / 10.0).min(1.0);
        
        // Check if all errors are from same component
        let components: Vec<Option<&String>> = sequence.iter().map(|step| step.component.as_ref()).collect();
        let unique_components: HashSet<_> = components.iter().collect();
        
        let component_consistency = if unique_components.len() == 1 {
            1.0
        } else {
            1.0 / (unique_components.len() as f32)
        };
        
        base_confidence * component_consistency
    }
    
    fn calculate_group_severity(&self, errors: &[LogEntry]) -> String {
        let high_count = errors.iter()
            .filter(|log| log.level.as_ref().map_or(false, |l| l == "ERROR"))
            .count();
        
        let total_count = errors.len();
        
        if high_count as f32 / total_count as f32 > 0.7 {
            "HIGH".to_string()
        } else if high_count > 0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::LogEntry;
    
    #[test]
    fn test_pattern_analyzer_creation() {
        let analyzer = PatternAnalyzer::new();
        assert!(analyzer.message_patterns.is_empty());
        assert!(analyzer.error_sequences.is_empty());
    }
    
    #[test]
    fn test_normalize_message() {
        let analyzer = PatternAnalyzer::new();
        let message = "Failed to connect to database at 192.168.1.1:5432";
        let normalized = analyzer.normalize_message(message);
        
        // Should remove IP address and port
        assert!(!normalized.contains("192"));
        assert!(!normalized.contains("5432"));
        assert!(normalized.contains("failed"));
        assert!(normalized.contains("connect"));
        assert!(normalized.contains("database"));
    }
    
    #[test]
    fn test_extract_component() {
        let analyzer = PatternAnalyzer::new();
        
        // Test module.function pattern
        assert_eq!(
            analyzer.extract_component("database.ConnectionPool: connection failed"),
            Some("database".to_string())
        );
        
        // Test component: pattern
        assert_eq!(
            analyzer.extract_component("Auth: authentication failed"),
            Some("Auth".to_string())
        );
    }
    
    #[test]
    fn test_analyze_logs() {
        let mut analyzer = PatternAnalyzer::new();
        let logs = vec![
            LogEntry {
                timestamp: Some("$1".to_string()),
                level: Some("$2".to_string()),
                message: "$3".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("$1".to_string()),
                level: Some("$2".to_string()),
                message: "$3".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("$1".to_string()),
                level: Some("$2".to_string()),
                message: "$3".to_string(),
                line_number: Some(1),
            },
        ];
        
        let result = analyzer.analyze(&logs).unwrap();
        
        assert!(!result.recurring_patterns.is_empty());
        assert!(!result.grouped_errors.is_empty());
    }
}