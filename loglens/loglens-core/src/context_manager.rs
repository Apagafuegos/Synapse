use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::classification::{ErrorClassification, ErrorClassifier, ErrorCategory};
use crate::input::LogEntry;

#[derive(Debug, Clone)]
pub struct RelevanceScorer {
    user_context_keywords: Vec<String>,
    error_type_weights: HashMap<String, f32>,
    recency_weight: f32,
    severity_weight: f32,
}

impl RelevanceScorer {
    pub fn new(user_context: Option<&str>) -> Self {
        let user_context_keywords = user_context
            .map(|ctx| {
                ctx.to_lowercase()
                    .split_whitespace()
                    .filter(|word| word.len() > 2) // Filter out small words
                    .map(|word| word.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let mut error_type_weights = HashMap::new();
        error_type_weights.insert("error".to_string(), 1.0);
        error_type_weights.insert("exception".to_string(), 0.9);
        error_type_weights.insert("fail".to_string(), 0.8);
        error_type_weights.insert("warn".to_string(), 0.6);
        error_type_weights.insert("info".to_string(), 0.3);
        error_type_weights.insert("debug".to_string(), 0.1);

        Self {
            user_context_keywords,
            error_type_weights,
            recency_weight: 0.2,
            severity_weight: 0.3,
        }
    }

    pub fn score_relevance(&self, entry: &LogEntry, classification: &ErrorClassification, position: usize, total_entries: usize) -> f32 {
        let mut score = 0.0;

        // Base score from error classification confidence
        score += classification.confidence * 0.4;

        // Score based on error category importance
        score += self.score_error_category(&classification.category) * 0.3;

        // Score based on user context keyword matching
        score += self.score_user_context_match(entry) * 0.2;

        // Recency score (more recent errors are more relevant)
        let recency_score = 1.0 - (position as f32 / total_entries as f32);
        score += recency_score * self.recency_weight;

        // Log level severity score
        score += self.score_log_level(entry.level.as_deref().unwrap_or("unknown")) * self.severity_weight;

        // Boost score if it's part of an error cluster
        if self.is_likely_error_cluster(entry) {
            score += 0.15;
        }

        score.min(1.0)
    }

    fn score_error_category(&self, category: &ErrorCategory) -> f32 {
        match category {
            ErrorCategory::CodeRelated { .. } => 0.9,
            ErrorCategory::InfrastructureRelated { severity, .. } => {
                match severity {
                    crate::classification::Severity::Critical => 1.0,
                    crate::classification::Severity::High => 0.8,
                    crate::classification::Severity::Medium => 0.6,
                    crate::classification::Severity::Low => 0.4,
                }
            },
            ErrorCategory::ConfigurationRelated { .. } => 0.7,
            ErrorCategory::ExternalServiceRelated { .. } => 0.6,
            ErrorCategory::UnknownRelated => 0.2,
        }
    }

    fn score_user_context_match(&self, entry: &LogEntry) -> f32 {
        if self.user_context_keywords.is_empty() {
            return 0.0;
        }

        let message_lower = entry.message.to_lowercase();
        let matches = self.user_context_keywords.iter()
            .filter(|keyword| message_lower.contains(*keyword))
            .count();

        (matches as f32 / self.user_context_keywords.len() as f32).min(1.0)
    }

    fn score_log_level(&self, level: &str) -> f32 {
        self.error_type_weights
            .get(&level.to_lowercase())
            .copied()
            .unwrap_or(0.1)
    }

    fn is_likely_error_cluster(&self, entry: &LogEntry) -> bool {
        // Simple heuristic: if the message contains specific patterns suggesting it's part of a sequence
        let message_lower = entry.message.to_lowercase();
        message_lower.contains("caused by") ||
        message_lower.contains("at ") ||
        message_lower.contains("stack trace") ||
        message_lower.contains("nested exception")
    }
}

#[derive(Debug, Clone)]
pub struct ContextManager {
    max_tokens: usize,
    current_size: usize,
    priority_errors: Vec<(LogEntry, ErrorClassification, f32)>, // (entry, classification, relevance_score)
    related_errors: Vec<(LogEntry, ErrorClassification, f32)>,
    unrelated_errors: Vec<(LogEntry, ErrorClassification, f32)>,
    classifier: ErrorClassifier,
    scorer: RelevanceScorer,
}

impl ContextManager {
    pub fn new(max_tokens: usize, user_context: Option<&str>) -> Self {
        Self {
            max_tokens,
            current_size: 0,
            priority_errors: Vec::new(),
            related_errors: Vec::new(),
            unrelated_errors: Vec::new(),
            classifier: ErrorClassifier::new(),
            scorer: RelevanceScorer::new(user_context),
        }
    }

    pub fn add_log_entry(&mut self, entry: LogEntry, position: usize, total_entries: usize) -> Result<()> {
        // Classify the error
        let classification = self.classifier.classify_error(&entry.message, None);

        // Score relevance
        let relevance_score = self.scorer.score_relevance(&entry, &classification, position, total_entries);

        // Estimate token cost (rough estimation: ~4 chars per token)
        let estimated_tokens = entry.message.len() / 4;

        // Categorize based on relevance score
        if relevance_score >= 0.7 {
            self.priority_errors.push((entry, classification, relevance_score));
        } else if relevance_score >= 0.4 {
            self.related_errors.push((entry, classification, relevance_score));
        } else {
            self.unrelated_errors.push((entry, classification, relevance_score));
        }

        self.current_size += estimated_tokens;

        // If we exceed the token limit, trim the lowest priority entries
        if self.current_size > self.max_tokens {
            self.trim_to_fit();
        }

        Ok(())
    }

    pub fn create_ai_payload(&self) -> AIAnalysisPayload {
        let mut payload = AIAnalysisPayload::new();

        // Always include priority errors
        for (entry, classification, score) in &self.priority_errors {
            payload.add_priority_entry(entry.clone(), classification.clone(), *score);
        }

        // Add related errors if we have space
        let remaining_tokens = self.max_tokens.saturating_sub(payload.estimated_tokens());
        let mut added_tokens = 0;

        for (entry, classification, score) in &self.related_errors {
            let entry_tokens = entry.message.len() / 4;
            if added_tokens + entry_tokens <= remaining_tokens {
                payload.add_related_entry(entry.clone(), classification.clone(), *score);
                added_tokens += entry_tokens;
            } else {
                break;
            }
        }

        // Add summary of unrelated errors if any
        if !self.unrelated_errors.is_empty() {
            payload.add_unrelated_summary(self.create_unrelated_summary());
        }

        payload
    }

    fn trim_to_fit(&mut self) {
        // Sort all categories by relevance score (descending)
        self.priority_errors.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        self.related_errors.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        self.unrelated_errors.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Remove lowest scoring entries until we fit
        let mut current_tokens = 0;

        // Keep priority errors (trim if necessary)
        let mut kept_priority = Vec::new();
        for (entry, classification, score) in &self.priority_errors {
            let entry_tokens = entry.message.len() / 4;
            if current_tokens + entry_tokens <= self.max_tokens {
                kept_priority.push((entry.clone(), classification.clone(), *score));
                current_tokens += entry_tokens;
            }
        }
        self.priority_errors = kept_priority;

        // Add related errors if space permits
        let mut kept_related = Vec::new();
        for (entry, classification, score) in &self.related_errors {
            let entry_tokens = entry.message.len() / 4;
            if current_tokens + entry_tokens <= self.max_tokens {
                kept_related.push((entry.clone(), classification.clone(), *score));
                current_tokens += entry_tokens;
            }
        }
        self.related_errors = kept_related;

        // Keep unrelated for summary only
        self.current_size = current_tokens;
    }

    fn create_unrelated_summary(&self) -> String {
        let total_unrelated = self.unrelated_errors.len();
        let mut category_counts = HashMap::new();

        for (_, classification, _) in &self.unrelated_errors {
            let category_name = match &classification.category {
                ErrorCategory::CodeRelated { .. } => "Code",
                ErrorCategory::InfrastructureRelated { component, .. } => component.as_str(),
                ErrorCategory::ConfigurationRelated { .. } => "Configuration",
                ErrorCategory::ExternalServiceRelated { .. } => "External Service",
                ErrorCategory::UnknownRelated => "Unknown",
            };
            *category_counts.entry(category_name.to_string()).or_insert(0) += 1;
        }

        let mut summary = format!("Additional {} unrelated log entries found:\n", total_unrelated);
        for (category, count) in category_counts {
            summary.push_str(&format!("- {}: {} entries\n", category, count));
        }

        summary
    }

    pub fn get_stats(&self) -> ContextStats {
        ContextStats {
            total_entries: self.priority_errors.len() + self.related_errors.len() + self.unrelated_errors.len(),
            priority_entries: self.priority_errors.len(),
            related_entries: self.related_errors.len(),
            unrelated_entries: self.unrelated_errors.len(),
            estimated_tokens: self.current_size,
            max_tokens: self.max_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisPayload {
    pub priority_entries: Vec<AnalysisEntry>,
    pub related_entries: Vec<AnalysisEntry>,
    pub unrelated_summary: Option<String>,
    pub context_meta: ContextMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisEntry {
    pub log_entry: LogEntry,
    pub classification: ErrorClassification,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMeta {
    pub total_entries_analyzed: usize,
    pub priority_count: usize,
    pub estimated_tokens: usize,
    pub truncated: bool,
}

impl AIAnalysisPayload {
    pub fn new() -> Self {
        Self {
            priority_entries: Vec::new(),
            related_entries: Vec::new(),
            unrelated_summary: None,
            context_meta: ContextMeta {
                total_entries_analyzed: 0,
                priority_count: 0,
                estimated_tokens: 0,
                truncated: false,
            },
        }
    }

    pub fn add_priority_entry(&mut self, entry: LogEntry, classification: ErrorClassification, score: f32) {
        self.priority_entries.push(AnalysisEntry {
            log_entry: entry,
            classification,
            relevance_score: score,
        });
        self.context_meta.priority_count += 1;
    }

    pub fn add_related_entry(&mut self, entry: LogEntry, classification: ErrorClassification, score: f32) {
        self.related_entries.push(AnalysisEntry {
            log_entry: entry,
            classification,
            relevance_score: score,
        });
    }

    pub fn add_unrelated_summary(&mut self, summary: String) {
        self.unrelated_summary = Some(summary);
    }

    pub fn estimated_tokens(&self) -> usize {
        let priority_tokens: usize = self.priority_entries.iter()
            .map(|entry| entry.log_entry.message.len() / 4)
            .sum();

        let related_tokens: usize = self.related_entries.iter()
            .map(|entry| entry.log_entry.message.len() / 4)
            .sum();

        let summary_tokens = self.unrelated_summary
            .as_ref()
            .map(|s| s.len() / 4)
            .unwrap_or(0);

        priority_tokens + related_tokens + summary_tokens
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_entries: usize,
    pub priority_entries: usize,
    pub related_entries: usize,
    pub unrelated_entries: usize,
    pub estimated_tokens: usize,
    pub max_tokens: usize,
}