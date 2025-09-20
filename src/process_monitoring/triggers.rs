//! Trigger Pattern Matching System
//! 
//! Provides advanced pattern matching for log analysis with AI integration,
//! supporting regex patterns, severity-based triggers, and custom rules.

use std::collections::{HashMap, VecDeque};
use regex::Regex;
use crate::model::LogEntry;
use crate::model::LogLevel;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerConfig {
    pub enabled: bool,
    pub auto_analyze: bool,
    pub analysis_cooldown_seconds: u64,
    pub max_triggers_per_minute: u32,
    pub include_context_lines: usize,
    pub severity_threshold: Option<LogLevel>,
}

/// Trigger type definitions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Pattern-based trigger (regex)
    Pattern { pattern: String, case_sensitive: bool },
    /// Log level trigger
    Level { min_level: LogLevel },
    /// Time-based trigger (frequency)
    Frequency { count: usize, within_seconds: u64 },
    /// Composite trigger (AND/OR logic)
    Composite { triggers: Vec<TriggerDefinition>, operator: LogicalOperator },
    /// Custom trigger with external function
    Custom { name: String, params: HashMap<String, String> },
}

/// Logical operators for composite triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Trigger definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger_type: TriggerType,
    pub enabled: bool,
    pub priority: TriggerPriority,
    pub action: TriggerAction,
    pub cooldown_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u64,
}

/// Trigger priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriggerPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actions to take when trigger fires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerAction {
    /// Perform AI analysis
    Analyze { provider: Option<String>, model: Option<String> },
    /// Send notification
    Notify { channel: String, message: String },
    /// Execute command
    Execute { command: String, args: Vec<String> },
    /// Log the trigger event
    Log { level: LogLevel, message: String },
    /// Multiple actions
    Composite { actions: Vec<TriggerAction> },
}

/// Trigger match result with context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerMatch {
    pub trigger_id: String,
    pub trigger_name: String,
    pub matched_at: DateTime<Utc>,
    pub matched_entries: Vec<LogEntry>,
    pub context_entries: Vec<LogEntry>,
    pub priority: TriggerPriority,
    pub action: TriggerAction,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Trigger statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerStats {
    pub total_triggers: usize,
    pub enabled_triggers: usize,
    pub matches_last_hour: u64,
    pub matches_last_24h: u64,
    pub most_active_trigger: Option<String>,
    pub average_response_time_ms: f64,
}

/// Advanced trigger pattern matcher
pub struct TriggerMatcher {
    config: TriggerConfig,
    triggers: HashMap<String, TriggerDefinition>,
    compiled_patterns: HashMap<String, Regex>,
    match_history: VecDeque<TriggerMatch>,
    recent_matches: HashMap<String, Vec<DateTime<Utc>>>,
    frequency_counters: HashMap<String, VecDeque<DateTime<Utc>>>,
}

impl TriggerMatcher {
    /// Create a new trigger matcher
    pub fn new(config: TriggerConfig) -> Self {
        Self {
            config,
            triggers: HashMap::new(),
            compiled_patterns: HashMap::new(),
            match_history: VecDeque::new(),
            recent_matches: HashMap::new(),
            frequency_counters: HashMap::new(),
        }
    }
    
    /// Add a trigger definition
    pub fn add_trigger(&mut self, trigger: TriggerDefinition) -> Result<()> {
        // Pre-compile regex patterns for efficiency
        if let TriggerType::Pattern { pattern, case_sensitive } = &trigger.trigger_type {
            let regex = if *case_sensitive {
                Regex::new(pattern)
            } else {
                Regex::new(&format!("(?i){}", pattern))
            }.context(format!("Invalid regex pattern: {}", pattern))?;
            
            self.compiled_patterns.insert(trigger.id.clone(), regex);
        }
        
        self.triggers.insert(trigger.id.clone(), trigger);
        Ok(())
    }
    
    /// Add multiple triggers
    pub fn add_triggers(&mut self, triggers: Vec<TriggerDefinition>) -> Result<()> {
        for trigger in triggers {
            self.add_trigger(trigger)?;
        }
        Ok(())
    }
    
    /// Remove a trigger
    pub fn remove_trigger(&mut self, trigger_id: &str) -> Result<()> {
        self.triggers.remove(trigger_id);
        self.compiled_patterns.remove(trigger_id);
        self.recent_matches.remove(trigger_id);
        self.frequency_counters.remove(trigger_id);
        Ok(())
    }
    
    /// Get all triggers
    pub fn get_triggers(&self) -> Vec<&TriggerDefinition> {
        self.triggers.values().collect()
    }
    
    /// Get enabled triggers
    pub fn get_enabled_triggers(&self) -> Vec<TriggerDefinition> {
        self.triggers.values()
            .filter(|t| t.enabled)
            .cloned()
            .collect()
    }
    
    /// Process a log entry and return matching triggers
    pub fn process_entry(&mut self, entry: &LogEntry, recent_entries: &[LogEntry]) -> Result<Vec<TriggerMatch>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }
        
        let mut matches = Vec::new();
        let now = Utc::now();
        
        // Get enabled triggers and evaluate them
        let enabled_triggers = self.get_enabled_triggers();
        let mut context_entries_vec = Vec::new();
        
        for trigger in enabled_triggers {
            // Check cooldown
            if let Some(last_triggered) = trigger.last_triggered {
                let cooldown = Duration::seconds(trigger.cooldown_seconds as i64);
                if now - last_triggered < cooldown {
                    continue;
                }
            }
            
            // Check rate limiting
            if self.is_rate_limited(&trigger.id, now)? {
                continue;
            }
            
            // Evaluate trigger
            if self.evaluate_trigger(&trigger, entry, recent_entries)? {
                let context_entries = self.get_context_entries(entry, recent_entries);
                context_entries_vec.push(context_entries.clone());
                
                let trigger_match = TriggerMatch {
                    trigger_id: trigger.id.clone(),
                    trigger_name: trigger.name.clone(),
                    matched_at: now,
                    matched_entries: vec![entry.clone()],
                    context_entries,
                    priority: trigger.priority.clone(),
                    action: trigger.action.clone(),
                    confidence: self.calculate_confidence(&trigger, entry),
                    metadata: HashMap::new(),
                };
                
                matches.push(trigger_match);
                
                // Update trigger state
                self.update_trigger_state(&trigger.id, now);
            }
        }
        
        // Store matches in history
        for match_result in &matches {
            self.match_history.push_back(match_result.clone());
            if self.match_history.len() > 1000 { // Keep last 1000 matches
                self.match_history.pop_front();
            }
        }
        
        Ok(matches)
    }
    
    /// Process multiple log entries
    pub fn process_entries(&mut self, entries: &[LogEntry], recent_entries: &[LogEntry]) -> Result<Vec<TriggerMatch>> {
        let mut all_matches = Vec::new();
        
        for entry in entries {
            let matches = self.process_entry(entry, recent_entries)?;
            all_matches.extend(matches);
        }
        
        Ok(all_matches)
    }
    
    /// Get trigger statistics
    pub fn get_stats(&self) -> TriggerStats {
        let now = Utc::now();
        let hour_ago = now - Duration::hours(1);
        let day_ago = now - Duration::days(1);
        
        let matches_last_hour = self.match_history.iter()
            .filter(|m| m.matched_at > hour_ago)
            .count() as u64;
        
        let matches_last_24h = self.match_history.iter()
            .filter(|m| m.matched_at > day_ago)
            .count() as u64;
        
        let trigger_counts = self.match_history.iter()
            .fold(HashMap::new(), |mut counts, m| {
                *counts.entry(&m.trigger_id).or_insert(0) += 1;
                counts
            });
        
        let most_active_trigger = trigger_counts.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(id, _)| id.clone());
        
        TriggerStats {
            total_triggers: self.triggers.len(),
            enabled_triggers: self.get_enabled_triggers().len(),
            matches_last_hour,
            matches_last_24h,
            most_active_trigger,
            average_response_time_ms: 0.0, // Would need timing data
        }
    }
    
    /// Get recent trigger matches
    pub fn get_recent_matches(&self, count: usize) -> Vec<TriggerMatch> {
        self.match_history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Clear trigger history
    pub fn clear_history(&mut self) {
        self.match_history.clear();
        self.recent_matches.clear();
        self.frequency_counters.clear();
    }
    
    /// Create default trigger definitions
    pub fn create_default_triggers() -> Vec<TriggerDefinition> {
        vec![
            TriggerDefinition {
                id: "error_trigger".to_string(),
                name: "Error Detection".to_string(),
                description: "Triggers on ERROR level log entries".to_string(),
                trigger_type: TriggerType::Level { min_level: LogLevel::Error },
                enabled: true,
                priority: TriggerPriority::High,
                action: TriggerAction::Analyze { provider: None, model: None },
                cooldown_seconds: 60,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            TriggerDefinition {
                id: "critical_trigger".to_string(),
                name: "Critical Error Detection".to_string(),
                description: "Triggers on CRITICAL or FATAL log entries".to_string(),
                trigger_type: TriggerType::Pattern { 
                    pattern: r"(?i)(critical|fatal|panic)".to_string(), 
                    case_sensitive: false 
                },
                enabled: true,
                priority: TriggerPriority::Critical,
                action: TriggerAction::Analyze { provider: None, model: None },
                cooldown_seconds: 30,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            TriggerDefinition {
                id: "exception_trigger".to_string(),
                name: "Exception Detection".to_string(),
                description: "Triggers on exception patterns in logs".to_string(),
                trigger_type: TriggerType::Pattern { 
                    pattern: r"(?i)(exception|stacktrace|traceback)".to_string(), 
                    case_sensitive: false 
                },
                enabled: true,
                priority: TriggerPriority::High,
                action: TriggerAction::Analyze { provider: None, model: None },
                cooldown_seconds: 120,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            TriggerDefinition {
                id: "connection_error_trigger".to_string(),
                name: "Connection Error Detection".to_string(),
                description: "Triggers on connection-related errors".to_string(),
                trigger_type: TriggerType::Pattern { 
                    pattern: r"(?i)(connection.*failed|timeout|refused|unreachable)".to_string(), 
                    case_sensitive: false 
                },
                enabled: true,
                priority: TriggerPriority::Medium,
                action: TriggerAction::Analyze { provider: None, model: None },
                cooldown_seconds: 300,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
            TriggerDefinition {
                id: "high_frequency_trigger".to_string(),
                name: "High Frequency Error Detection".to_string(),
                description: "Triggers when many errors occur in short time".to_string(),
                trigger_type: TriggerType::Frequency { 
                    count: 10, 
                    within_seconds: 60 
                },
                enabled: true,
                priority: TriggerPriority::High,
                action: TriggerAction::Analyze { provider: None, model: None },
                cooldown_seconds: 600,
                created_at: Utc::now(),
                last_triggered: None,
                trigger_count: 0,
            },
        ]
    }
    
    /// Evaluate if a trigger matches the given log entry
    fn evaluate_trigger(&mut self, trigger: &TriggerDefinition, entry: &LogEntry, recent_entries: &[LogEntry]) -> Result<bool> {
        match &trigger.trigger_type {
            TriggerType::Pattern { pattern, case_sensitive } => {
                let regex = self.compiled_patterns.get(&trigger.id)
                    .ok_or_else(|| anyhow::anyhow!("Pattern not compiled for trigger: {}", trigger.id))?;
                
                let search_text = if *case_sensitive {
                    format!("{} {}", entry.message, entry.raw_line)
                } else {
                    format!("{} {}", entry.message.to_lowercase(), entry.raw_line.to_lowercase())
                };
                
                Ok(regex.is_match(&search_text))
            },
            TriggerType::Level { min_level } => {
                Ok(entry.level >= *min_level)
            },
            TriggerType::Frequency { count, within_seconds } => {
                self.check_frequency_trigger(trigger.id.clone(), *count, *within_seconds, recent_entries)
            },
            TriggerType::Composite { triggers, operator } => {
                self.evaluate_composite_trigger(triggers, operator, entry, recent_entries)
            },
            TriggerType::Custom { .. } => {
                // Custom triggers would need external function evaluation
                // For now, return false
                Ok(false)
            },
        }
    }
    
    /// Check frequency-based trigger
    fn check_frequency_trigger(&mut self, trigger_id: String, count: usize, within_seconds: u64, recent_entries: &[LogEntry]) -> Result<bool> {
        let now = Utc::now();
        let time_threshold = now - Duration::seconds(within_seconds as i64);
        
        // Count matching entries in the time window
        let matching_count = recent_entries.iter()
            .filter(|entry| entry.timestamp > time_threshold)
            .count();
        
        // Update frequency counter
        let counter = self.frequency_counters.entry(trigger_id.clone()).or_insert_with(|| {
            VecDeque::with_capacity(count * 2)
        });
        
        // Add current timestamp
        counter.push_back(now);
        
        // Remove old timestamps
        while counter.front().map(|&t| t <= time_threshold).unwrap_or(false) {
            counter.pop_front();
        }
        
        Ok(counter.len() >= count)
    }
    
    /// Evaluate composite trigger with logical operators
    fn evaluate_composite_trigger(&mut self, triggers: &[TriggerDefinition], operator: &LogicalOperator, entry: &LogEntry, recent_entries: &[LogEntry]) -> Result<bool> {
        let results: Vec<bool> = triggers.iter()
            .map(|t| self.evaluate_trigger(t, entry, recent_entries))
            .collect::<Result<Vec<bool>>>()?;
        
        match operator {
            LogicalOperator::And => Ok(results.iter().all(|&r| r)),
            LogicalOperator::Or => Ok(results.iter().any(|&r| r)),
            LogicalOperator::Not => Ok(!results.iter().all(|&r| r)),
        }
    }
    
    /// Get context entries around a matched entry
    fn get_context_entries(&self, entry: &LogEntry, recent_entries: &[LogEntry]) -> Vec<LogEntry> {
        let context_size = self.config.include_context_lines;
        
        if recent_entries.is_empty() {
            return Vec::new();
        }
        
        // Find the position of the matched entry
        let entry_pos = recent_entries.iter()
            .position(|e| e.timestamp == entry.timestamp && e.message == entry.message);
        
        if let Some(pos) = entry_pos {
            let start = pos.saturating_sub(context_size / 2);
            let end = (pos + context_size / 2 + 1).min(recent_entries.len());
            
            recent_entries[start..end].to_vec()
        } else {
            Vec::new()
        }
    }
    
    /// Calculate confidence score for a trigger match
    fn calculate_confidence(&self, trigger: &TriggerDefinition, entry: &LogEntry) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence
        
        // Boost confidence based on log level
        confidence += match entry.level {
            LogLevel::Error => 0.2,
            LogLevel::Warn => 0.1,
            LogLevel::Info => 0.0,
            LogLevel::Debug => -0.1,
            LogLevel::Trace => -0.2,
            LogLevel::Unknown => -0.15,
        };
        
        // Boost confidence based on trigger priority
        confidence += match trigger.priority {
            TriggerPriority::Critical => 0.3,
            TriggerPriority::High => 0.2,
            TriggerPriority::Medium => 0.1,
            TriggerPriority::Low => 0.0,
        };
        
        // Boost confidence for pattern matches
        if let TriggerType::Pattern { .. } = trigger.trigger_type {
            confidence += 0.1;
        }
        
        confidence.max(0.0).min(1.0) as f64
    }
    
    /// Check if trigger is rate limited
    fn is_rate_limited(&self, trigger_id: &str, now: DateTime<Utc>) -> Result<bool> {
        let recent_matches = self.recent_matches.get(trigger_id)
            .map(|matches| matches.as_slice())
            .unwrap_or(&[]);
        
        // Count matches in the last minute
        let minute_ago = now - Duration::minutes(1);
        let recent_count = recent_matches.iter()
            .filter(|&&timestamp| timestamp > minute_ago)
            .count() as u32;
        
        Ok(recent_count >= self.config.max_triggers_per_minute)
    }
    
    /// Update trigger state after a match
    fn update_trigger_state(&mut self, trigger_id: &str, now: DateTime<Utc>) {
        // Update recent matches
        let matches = self.recent_matches.entry(trigger_id.to_string()).or_insert_with(Vec::new);
        matches.push(now);
        
        // Keep only recent matches (last hour)
        let hour_ago = now - Duration::hours(1);
        matches.retain(|&timestamp| timestamp > hour_ago);
        
        // Update trigger definition
        if let Some(trigger) = self.triggers.get_mut(trigger_id) {
            trigger.last_triggered = Some(now);
            trigger.trigger_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_matcher_creation() {
        let config = TriggerConfig {
            enabled: true,
            auto_analyze: true,
            analysis_cooldown_seconds: 60,
            max_triggers_per_minute: 10,
            include_context_lines: 5,
            severity_threshold: None,
        };
        
        let matcher = TriggerMatcher::new(config);
        assert_eq!(matcher.get_triggers().len(), 0);
        assert_eq!(matcher.get_stats().total_triggers, 0);
    }

    #[test]
    fn test_add_trigger() -> Result<()> {
        let config = TriggerConfig {
            enabled: true,
            auto_analyze: true,
            analysis_cooldown_seconds: 60,
            max_triggers_per_minute: 10,
            include_context_lines: 5,
            severity_threshold: None,
        };
        
        let mut matcher = TriggerMatcher::new(config);
        
        let trigger = TriggerDefinition {
            id: "test_trigger".to_string(),
            name: "Test Trigger".to_string(),
            description: "Test trigger for unit testing".to_string(),
            trigger_type: TriggerType::Level { min_level: LogLevel::Error },
            enabled: true,
            priority: TriggerPriority::Medium,
            action: TriggerAction::Log { level: LogLevel::Info, message: "Trigger fired".to_string() },
            cooldown_seconds: 0,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        };
        
        matcher.add_trigger(trigger)?;
        
        assert_eq!(matcher.get_triggers().len(), 1);
        assert_eq!(matcher.get_enabled_triggers().len(), 1);
        
        Ok(())
    }

    #[test]
    fn test_level_trigger_matching() -> Result<()> {
        let config = TriggerConfig {
            enabled: true,
            auto_analyze: true,
            analysis_cooldown_seconds: 0,
            max_triggers_per_minute: 10,
            include_context_lines: 0,
            severity_threshold: None,
        };
        
        let mut matcher = TriggerMatcher::new(config);
        
        let trigger = TriggerDefinition {
            id: "error_trigger".to_string(),
            name: "Error Trigger".to_string(),
            description: "Triggers on error level".to_string(),
            trigger_type: TriggerType::Level { min_level: LogLevel::Error },
            enabled: true,
            priority: TriggerPriority::High,
            action: TriggerAction::Analyze { provider: None, model: None },
            cooldown_seconds: 0,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        };
        
        matcher.add_trigger(trigger)?;
        
        let error_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: "Something went wrong".to_string(),
            fields: HashMap::new(),
            raw_line: "ERROR: Something went wrong".to_string(),
        };
        
        let info_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Everything is fine".to_string(),
            fields: HashMap::new(),
            raw_line: "INFO: Everything is fine".to_string(),
        };
        
        let error_matches = matcher.process_entry(&error_entry, &[])?;
        let info_matches = matcher.process_entry(&info_entry, &[])?;
        
        assert_eq!(error_matches.len(), 1);
        assert_eq!(info_matches.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_pattern_trigger_matching() -> Result<()> {
        let config = TriggerConfig {
            enabled: true,
            auto_analyze: true,
            analysis_cooldown_seconds: 0,
            max_triggers_per_minute: 10,
            include_context_lines: 0,
            severity_threshold: None,
        };
        
        let mut matcher = TriggerMatcher::new(config);
        
        let trigger = TriggerDefinition {
            id: "critical_trigger".to_string(),
            name: "Critical Trigger".to_string(),
            description: "Triggers on critical patterns".to_string(),
            trigger_type: TriggerType::Pattern { 
                pattern: r"(?i)critical".to_string(), 
                case_sensitive: false 
            },
            enabled: true,
            priority: TriggerPriority::Critical,
            action: TriggerAction::Analyze { provider: None, model: None },
            cooldown_seconds: 0,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        };
        
        matcher.add_trigger(trigger)?;
        
        let critical_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: "CRITICAL: System failure".to_string(),
            fields: HashMap::new(),
            raw_line: "CRITICAL: System failure".to_string(),
        };
        
        let normal_entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Normal operation".to_string(),
            fields: HashMap::new(),
            raw_line: "Normal operation".to_string(),
        };
        
        let critical_matches = matcher.process_entry(&critical_entry, &[])?;
        let normal_matches = matcher.process_entry(&normal_entry, &[])?;
        
        assert_eq!(critical_matches.len(), 1);
        assert_eq!(normal_matches.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_default_triggers_creation() {
        let triggers = TriggerMatcher::create_default_triggers();
        
        assert!(triggers.len() >= 5); // Should create at least 5 default triggers
        
        // Check that error trigger exists
        let error_trigger = triggers.iter().find(|t| t.id == "error_trigger");
        assert!(error_trigger.is_some());
        
        // Check that critical trigger exists
        let critical_trigger = triggers.iter().find(|t| t.id == "critical_trigger");
        assert!(critical_trigger.is_some());
    }
}