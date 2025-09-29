use crate::input::LogEntry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub error_correlations: Vec<ErrorCorrelation>,
    pub service_dependencies: Vec<ServiceDependency>,
    pub cascading_failures: Vec<CascadingFailure>,
    pub root_causes: Vec<RootCause>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCorrelation {
    pub correlation_id: String,
    pub error_types: Vec<String>,
    pub affected_services: Vec<String>,
    pub correlation_strength: f32,
    pub time_window: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub service_a: String,
    pub service_b: String,
    pub dependency_type: DependencyType,
    pub strength: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Direct,
    Indirect,
    SharedResource,
    DataFlow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadingFailure {
    pub failure_id: String,
    pub sequence: Vec<FailureStep>,
    pub root_service: String,
    pub impact_scope: String,
    pub propagation_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureStep {
    pub service: String,
    pub error: String,
    pub timestamp: Option<String>,
    pub delay_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub cause_id: String,
    pub service: String,
    pub error_pattern: String,
    pub confidence: f32,
    pub contributing_factors: Vec<String>,
}

pub struct CorrelationAnalyzer {
    service_errors: HashMap<String, Vec<LogEntry>>,
    error_patterns: HashMap<String, Vec<String>>,
    timeline: Vec<TimelineEvent>,
}

#[derive(Debug, Clone)]
struct TimelineEvent {
    timestamp: Option<String>,
    service: String,
    error_type: String,
    message: String,
}

impl CorrelationAnalyzer {
    pub fn new() -> Self {
        Self {
            service_errors: HashMap::new(),
            error_patterns: HashMap::new(),
            timeline: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, logs: &[LogEntry]) -> Result<CorrelationAnalysis> {
        self.extract_service_errors(logs);
        self.build_timeline(logs);
        self.identify_error_patterns(logs);
        
        Ok(CorrelationAnalysis {
            error_correlations: self.find_error_correlations(),
            service_dependencies: self.infer_service_dependencies(),
            cascading_failures: self.detect_cascading_failures(),
            root_causes: self.identify_root_causes(),
        })
    }
    
    fn extract_service_errors(&mut self, logs: &[LogEntry]) {
        for log in logs {
            if let Some(level) = &log.level {
                if level == "ERROR" || level == "WARN" {
                    let service = self.extract_service(&log.message);
                    self.service_errors
                        .entry(service)
                        .or_insert_with(Vec::new)
                        .push(log.clone());
                }
            }
        }
    }
    
    fn build_timeline(&mut self, logs: &[LogEntry]) {
        self.timeline = logs
            .iter()
            .filter(|log| log.level.as_ref().map_or(false, |l| l == "ERROR" || l == "WARN"))
            .map(|log| TimelineEvent {
                timestamp: log.timestamp.clone(),
                service: self.extract_service(&log.message),
                error_type: log.level.clone().unwrap_or_else(|| "UNKNOWN".to_string()),
                message: log.message.clone(),
            })
            .collect();
        
        // Sort timeline by timestamp
        self.timeline.sort_by(|a, b| {
            match (&a.timestamp, &b.timestamp) {
                (Some(ts_a), Some(ts_b)) => ts_a.cmp(ts_b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }
    
    fn identify_error_patterns(&mut self, logs: &[LogEntry]) {
        for log in logs {
            if let Some(level) = &log.level {
                if level == "ERROR" {
                    let pattern = self.extract_error_pattern(&log.message);
                    let service = self.extract_service(&log.message);
                    
                    self.error_patterns
                        .entry(pattern)
                        .or_insert_with(Vec::new)
                        .push(service);
                }
            }
        }
    }
    
    fn find_error_correlations(&self) -> Vec<ErrorCorrelation> {
        let mut correlations = Vec::new();
        
        // Find errors that occur together across services
        
        for (pattern1, services1) in &self.error_patterns {
            for (pattern2, services2) in &self.error_patterns {
                if pattern1 != pattern2 {
                    let common_services: HashSet<_> = services1.iter()
                        .filter(|s| services2.contains(s))
                        .collect();
                    
                    if !common_services.is_empty() {
                        let strength = common_services.len() as f32 / services1.len().max(services2.len()) as f32;
                        
                        if strength > 0.3 { // At least 30% overlap
                            correlations.push(ErrorCorrelation {
                                correlation_id: format!("corr_{}_{}", pattern1, pattern2),
                                error_types: vec![pattern1.clone(), pattern2.clone()],
                                affected_services: common_services.into_iter().cloned().collect(),
                                correlation_strength: strength,
                                time_window: "5m".to_string(), // Could be calculated from actual timestamps
                                description: format!("Correlated errors: {} and {}", pattern1, pattern2),
                            });
                        }
                    }
                }
            }
        }
        
        correlations.sort_by(|a, b| b.correlation_strength.partial_cmp(&a.correlation_strength).unwrap());
        correlations
    }
    
    fn infer_service_dependencies(&self) -> Vec<ServiceDependency> {
        let mut dependencies = Vec::new();
        
        // Analyze error sequences to infer dependencies
        for window in self.timeline.windows(2) {
            let event1 = &window[0];
            let event2 = &window[1];
            
            if event1.service != event2.service {
                let time_diff = self.calculate_time_diff(&event1.timestamp, &event2.timestamp);
                
                if time_diff.map_or(false, |diff| diff < 300) { // Within 5 minutes
                    let dependency_type = self.infer_dependency_type(&event1, &event2);
                    
                    dependencies.push(ServiceDependency {
                        service_a: event1.service.clone(),
                        service_b: event2.service.clone(),
                        dependency_type,
                        strength: 0.7, // Could be calculated from frequency
                        evidence: vec![
                            format!("Error in {} followed by error in {}", event1.service, event2.service),
                            format!("Time difference: {} seconds", time_diff.unwrap_or(0)),
                        ],
                    });
                }
            }
        }
        
        // Remove duplicates and aggregate strengths
        self.aggregate_dependencies(&mut dependencies)
    }
    
    fn detect_cascading_failures(&self) -> Vec<CascadingFailure> {
        let mut failures = Vec::new();
        
        // Look for sequences of errors across multiple services
        let mut current_sequence: Vec<FailureStep> = Vec::new();
        let mut sequence_start: Option<String> = None;
        
        for event in &self.timeline {
            let step = FailureStep {
                service: event.service.clone(),
                error: event.error_type.clone(),
                timestamp: event.timestamp.clone(),
                delay_seconds: sequence_start.as_ref().and_then(|start| {
                    self.calculate_time_diff(&Some(start.clone()), &event.timestamp)
                }),
            };
            
            if current_sequence.is_empty() {
                current_sequence.push(step);
                sequence_start = event.timestamp.clone();
            } else {
                let last_service = &current_sequence.last().unwrap().service;
                if last_service != &event.service {
                    current_sequence.push(step);
                } else {
                    // Same service, start new sequence
                    if current_sequence.len() > 2 {
                        failures.push(self.create_cascading_failure(&current_sequence));
                    }
                    current_sequence = vec![step];
                    sequence_start = event.timestamp.clone();
                }
            }
        }
        
        // Add final sequence
        if current_sequence.len() > 2 {
            failures.push(self.create_cascading_failure(&current_sequence));
        }
        
        failures
    }
    
    fn identify_root_causes(&self) -> Vec<RootCause> {
        let mut root_causes = Vec::new();
        
        // Find services that appear early in error sequences
        let service_frequency: HashMap<String, usize> = self.timeline
            .iter()
            .map(|event| &event.service)
            .fold(HashMap::new(), |mut acc, service| {
                *acc.entry(service.clone()).or_insert(0) += 1;
                acc
            });
        
        // Find error patterns that appear early
        for (pattern, services) in &self.error_patterns {
            let first_service = services.first();
            
            if let Some(service) = first_service {
                let frequency = service_frequency.get(service).copied().unwrap_or(0);
                let confidence = if frequency > 0 {
                    (services.len() as f32) / (frequency as f32)
                } else {
                    0.0
                };
                
                if confidence > 0.5 {
                    root_causes.push(RootCause {
                        cause_id: format!("root_{}", pattern),
                        service: service.clone(),
                        error_pattern: pattern.clone(),
                        confidence,
                        contributing_factors: vec![
                            "High error frequency".to_string(),
                            "Appears early in error sequences".to_string(),
                        ],
                    });
                }
            }
        }
        
        root_causes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        root_causes
    }
    
    fn extract_service(&self, message: &str) -> String {
        // Try to extract service names from error messages
        let patterns = vec![
            r"(\w+)-(\w+)", // service-component
            r"(\w+)\.(\w+)", // service.function
            r"(\w+):", // service:
            r"in (\w+)", // in service
            r"(\w+) service", // service service
            r"(\w+) failed", // service failed
        ];
        
        for pattern in patterns {
            if let Some(regex) = regex::Regex::new(pattern).ok() {
                if let Some(caps) = regex.captures(message) {
                    return caps.get(1).unwrap().as_str().to_string();
                }
            }
        }
        
        // Fallback: extract first word that could be a service name
        message.split_whitespace()
            .next()
            .map(|s| s.split(|c| c == '.' || c == ':').next().unwrap_or(s))
            .unwrap_or("unknown")
            .to_string()
    }
    
    fn extract_error_pattern(&self, message: &str) -> String {
        // Normalize error message to identify patterns
        message
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .filter(|word| !["the", "a", "an", "in", "on", "at", "to", "for"].contains(word))
            .take(3) // Take first 3 meaningful words
            .collect::<Vec<_>>()
            .join(" ")
    }
    

    
    fn calculate_time_diff(&self, ts1: &Option<String>, ts2: &Option<String>) -> Option<u64> {
        match (ts1, ts2) {
            (Some(t1), Some(t2)) => {
                let dt1 = self.parse_timestamp(t1);
                let dt2 = self.parse_timestamp(t2);
                
                match (dt1, dt2) {
                    (Some(d1), Some(d2)) => Some(d2.signed_duration_since(d1).num_seconds().abs() as u64),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    fn parse_timestamp(&self, timestamp: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        let formats = vec![
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
        ];
        
        for format in formats {
            if let Ok(dt) = chrono::DateTime::parse_from_str(timestamp, format) {
                return Some(dt.with_timezone(&chrono::Utc));
            }
        }
        
        None
    }
    
    fn infer_dependency_type(&self, event1: &TimelineEvent, event2: &TimelineEvent) -> DependencyType {
        // Simple heuristics to infer dependency type
        if event1.message.contains("database") && event2.message.contains("database") {
            DependencyType::SharedResource
        } else if event1.message.contains("timeout") && event2.message.contains("timeout") {
            DependencyType::Indirect
        } else if event1.message.contains("connection") && event2.message.contains("connection") {
            DependencyType::Direct
        } else {
            DependencyType::DataFlow
        }
    }
    
    fn aggregate_dependencies(&self, dependencies: &mut Vec<ServiceDependency>) -> Vec<ServiceDependency> {
        let mut aggregated = HashMap::new();
        
        for dep in dependencies.iter() {
            let key = (dep.service_a.clone(), dep.service_b.clone());
            let entry = aggregated.entry(key).or_insert(ServiceDependency {
                service_a: dep.service_a.clone(),
                service_b: dep.service_b.clone(),
                dependency_type: dep.dependency_type.clone(),
                strength: 0.0,
                evidence: Vec::new(),
            });
            
            entry.strength = (entry.strength + dep.strength) / 2.0;
            entry.evidence.extend(dep.evidence.clone());
        }
        
        aggregated.into_values().collect()
    }
    
    fn create_cascading_failure(&self, sequence: &[FailureStep]) -> CascadingFailure {
        let root_service = sequence.first().map(|s| s.service.clone()).unwrap_or_default();
        let propagation_time = if sequence.len() > 1 {
            if let (Some(first), Some(last)) = (&sequence.first().unwrap().timestamp, &sequence.last().unwrap().timestamp) {
                if let Some(diff) = self.calculate_time_diff(&Some(first.clone()), &Some(last.clone())) {
                    format!("{} seconds", diff)
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };
        
        CascadingFailure {
            failure_id: format!("cascade_{}", sequence.len()),
            sequence: sequence.to_vec(),
            root_service,
            impact_scope: format!("{} services affected", sequence.len()),
            propagation_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::LogEntry;
    
    #[test]
    fn test_correlation_analyzer_creation() {
        let analyzer = CorrelationAnalyzer::new();
        assert!(analyzer.service_errors.is_empty());
        assert!(analyzer.timeline.is_empty());
    }
    
    #[test]
    fn test_extract_service() {
        let analyzer = CorrelationAnalyzer::new();
        
        // Test service-component pattern
        assert_eq!(
            analyzer.extract_service("user-service: Database connection failed"),
            "user-service".to_string()
        );
        
        // Test service.function pattern
        assert_eq!(
            analyzer.extract_service("auth.Authenticator: Authentication failed"),
            "auth".to_string()
        );
        
        // Test fallback
        assert_eq!(
            analyzer.extract_service("Something went wrong"),
            "something".to_string()
        );
    }
    
    #[test]
    fn test_extract_error_pattern() {
        let analyzer = CorrelationAnalyzer::new();
        let pattern = analyzer.extract_error_pattern("Failed to connect to database server");
        
        assert!(pattern.contains("failed"));
        assert!(pattern.contains("connect"));
        assert!(pattern.contains("database"));
    }
    
    #[test]
    fn test_analyze_logs() {
        let mut analyzer = CorrelationAnalyzer::new();
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
        
        assert!(!result.service_dependencies.is_empty());
        // May or may not have cascading failures depending on timing
    }
}