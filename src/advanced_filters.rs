use crate::model::LogEntry;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvancedFilter {
    /// Field-based filtering: field == "value"
    FieldEqual { field: String, value: String },
    /// Field-based filtering: field != "value"
    FieldNotEqual { field: String, value: String },
    /// Field-based filtering: field contains "value"
    FieldContains { field: String, value: String },
    /// Field-based filtering: field matches regex
    FieldMatches { field: String, pattern: String },
    /// Numeric field comparison: field > value
    FieldGreaterThan { field: String, value: f64 },
    /// Numeric field comparison: field < value
    FieldLessThan { field: String, value: f64 },
    /// Numeric field comparison: field >= value
    FieldGreaterEqual { field: String, value: f64 },
    /// Numeric field comparison: field <= value
    FieldLessEqual { field: String, value: f64 },
    /// Time-based filtering: timestamp > time
    AfterTime { time: DateTime<Utc> },
    /// Time-based filtering: timestamp < time
    BeforeTime { time: DateTime<Utc> },
    /// Time-based filtering: timestamp in range [start, end]
    TimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// Logical AND: filter1 AND filter2
    And { filters: Vec<AdvancedFilter> },
    /// Logical OR: filter1 OR filter2
    Or { filters: Vec<AdvancedFilter> },
    /// Logical NOT: NOT filter
    Not { filter: Box<AdvancedFilter> },
    /// Complex expression evaluation
    Expression { expression: String },
    /// Machine learning anomaly detection
    AnomalyDetection { threshold: f64 },
    /// Pattern-based clustering
    PatternCluster { cluster_id: usize },
}

#[derive(Debug)]
pub struct AdvancedFilterChain {
    filters: Vec<AdvancedFilter>,
    compiled_regexes: HashMap<String, Regex>,
}

impl AdvancedFilterChain {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            compiled_regexes: HashMap::new(),
        }
    }

    pub fn add_filter(mut self, filter: AdvancedFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn from_query(query: &str) -> Result<Self, String> {
        let parser = QueryParser::new();
        parser.parse_query(query)
    }

    pub fn matches(&mut self, entry: &LogEntry) -> bool {
        let filters = std::mem::take(&mut self.filters);
        let result = filters
            .iter()
            .all(|filter| self.evaluate_filter(filter, entry));
        self.filters = filters;
        result
    }

    fn evaluate_filter(&mut self, filter: &AdvancedFilter, entry: &LogEntry) -> bool {
        match filter {
            AdvancedFilter::FieldEqual { field, value } => {
                self.get_field_value(entry, field).as_deref() == Some(value)
            }
            AdvancedFilter::FieldNotEqual { field, value } => {
                self.get_field_value(entry, field).as_deref() != Some(value)
            }
            AdvancedFilter::FieldContains { field, value } => self
                .get_field_value(entry, field)
                .map(|v| v.contains(value))
                .unwrap_or(false),
            AdvancedFilter::FieldMatches { field, pattern } => {
                if let Some(field_value) = self.get_field_value(entry, field) {
                    let regex = self
                        .compiled_regexes
                        .entry(pattern.clone())
                        .or_insert_with(|| {
                            Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap())
                        });
                    regex.is_match(&field_value)
                } else {
                    false
                }
            }
            AdvancedFilter::FieldGreaterThan { field, value } => self
                .get_numeric_field_value(entry, field)
                .map(|v| v > *value)
                .unwrap_or(false),
            AdvancedFilter::FieldLessThan { field, value } => self
                .get_numeric_field_value(entry, field)
                .map(|v| v < *value)
                .unwrap_or(false),
            AdvancedFilter::FieldGreaterEqual { field, value } => self
                .get_numeric_field_value(entry, field)
                .map(|v| v >= *value)
                .unwrap_or(false),
            AdvancedFilter::FieldLessEqual { field, value } => self
                .get_numeric_field_value(entry, field)
                .map(|v| v <= *value)
                .unwrap_or(false),
            AdvancedFilter::AfterTime { time } => entry.timestamp > *time,
            AdvancedFilter::BeforeTime { time } => entry.timestamp < *time,
            AdvancedFilter::TimeRange { start, end } => {
                entry.timestamp >= *start && entry.timestamp <= *end
            }
            AdvancedFilter::And { filters } => {
                filters.iter().all(|f| self.evaluate_filter(f, entry))
            }
            AdvancedFilter::Or { filters } => {
                filters.iter().any(|f| self.evaluate_filter(f, entry))
            }
            AdvancedFilter::Not { filter } => !self.evaluate_filter(filter, entry),
            AdvancedFilter::Expression { expression } => {
                self.evaluate_expression(expression, entry)
            }
            AdvancedFilter::AnomalyDetection { threshold } => {
                // This will be implemented in AI integration phase
                self.calculate_anomaly_score(entry) < *threshold
            }
            AdvancedFilter::PatternCluster { cluster_id } => {
                // This will be implemented in AI integration phase
                self.get_pattern_cluster_id(entry) == Some(*cluster_id)
            }
        }
    }

    fn get_field_value(&self, entry: &LogEntry, field: &str) -> Option<String> {
        match field {
            "timestamp" => Some(entry.timestamp.to_rfc3339()),
            "level" => Some(entry.level.to_string()),
            "message" => Some(entry.message.clone()),
            "raw_line" => Some(entry.raw_line.clone()),
            _ => entry.fields.get(field).cloned(),
        }
    }

    fn get_numeric_field_value(&self, entry: &LogEntry, field: &str) -> Option<f64> {
        self.get_field_value(entry, field)
            .and_then(|s| s.parse::<f64>().ok())
    }

    fn evaluate_expression(&self, expression: &str, entry: &LogEntry) -> bool {
        // Simple expression evaluator for basic arithmetic and comparisons
        // This is a simplified implementation - in production, use a proper expression parser
        let expr = expression.to_lowercase();

        // Handle simple field comparisons
        if expr.contains("==") {
            let parts: Vec<&str> = expr.split("==").collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('"');
                return self.get_field_value(entry, field).as_deref() == Some(value);
            }
        } else if expr.contains("!=") {
            let parts: Vec<&str> = expr.split("!=").collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                let value = parts[1].trim().trim_matches('"');
                return self.get_field_value(entry, field).as_deref() != Some(value);
            }
        } else if expr.contains(">") {
            let parts: Vec<&str> = expr.split(">").collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    return self
                        .get_numeric_field_value(entry, field)
                        .map(|v| v > value)
                        .unwrap_or(false);
                }
            }
        } else if expr.contains("<") {
            let parts: Vec<&str> = expr.split("<").collect();
            if parts.len() == 2 {
                let field = parts[0].trim();
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    return self
                        .get_numeric_field_value(entry, field)
                        .map(|v| v < value)
                        .unwrap_or(false);
                }
            }
        }

        // Default: check if expression is contained in message
        entry.message.to_lowercase().contains(&expr)
    }

    fn calculate_anomaly_score(&self, _entry: &LogEntry) -> f64 {
        // Placeholder for anomaly detection
        // Will be implemented in AI integration phase
        0.5
    }

    fn get_pattern_cluster_id(&self, _entry: &LogEntry) -> Option<usize> {
        // Placeholder for pattern clustering
        // Will be implemented in AI integration phase
        None
    }
}

#[derive(Debug)]
pub struct QueryParser {
    // Query parsing state
}

impl QueryParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_query(&self, query: &str) -> Result<AdvancedFilterChain, String> {
        let mut chain = AdvancedFilterChain::new();

        // Simple query parser - supports basic field operations
        let tokens: Vec<&str> = query.split_whitespace().collect();
        let mut i = 0;

        while i < tokens.len() {
            match tokens[i] {
                "AND" => i += 1, // Skip AND operators for now
                "OR" => {
                    // Handle OR logic
                    if i + 1 < tokens.len() {
                        let next_filter = self.parse_token(tokens[i + 1])?;
                        chain = chain.add_filter(AdvancedFilter::Or {
                            filters: vec![
                                AdvancedFilter::Expression {
                                    expression: tokens[i - 1].to_string(),
                                },
                                next_filter,
                            ],
                        });
                        i += 2;
                    } else {
                        return Err("Invalid OR expression".to_string());
                    }
                }
                token => {
                    let filter = self.parse_token(token)?;
                    chain = chain.add_filter(filter);
                    i += 1;
                }
            }
        }

        Ok(chain)
    }

    fn parse_token(&self, token: &str) -> Result<AdvancedFilter, String> {
        if token.contains("==") {
            let parts: Vec<&str> = token.split("==").collect();
            if parts.len() == 2 {
                return Ok(AdvancedFilter::FieldEqual {
                    field: parts[0].trim().to_string(),
                    value: parts[1].trim().trim_matches('"').to_string(),
                });
            }
        } else if token.contains("!=") {
            let parts: Vec<&str> = token.split("!=").collect();
            if parts.len() == 2 {
                return Ok(AdvancedFilter::FieldNotEqual {
                    field: parts[0].trim().to_string(),
                    value: parts[1].trim().trim_matches('"').to_string(),
                });
            }
        } else if token.contains(">") {
            let parts: Vec<&str> = token.split(">").collect();
            if parts.len() == 2 {
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    return Ok(AdvancedFilter::FieldGreaterThan {
                        field: parts[0].trim().to_string(),
                        value,
                    });
                }
            }
        } else if token.contains("<") {
            let parts: Vec<&str> = token.split("<").collect();
            if parts.len() == 2 {
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    return Ok(AdvancedFilter::FieldLessThan {
                        field: parts[0].trim().to_string(),
                        value,
                    });
                }
            }
        } else if token.contains("~") {
            let parts: Vec<&str> = token.split("~").collect();
            if parts.len() == 2 {
                return Ok(AdvancedFilter::FieldMatches {
                    field: parts[0].trim().to_string(),
                    pattern: parts[1].trim().trim_matches('"').to_string(),
                });
            }
        }

        // Default to message contains
        Ok(AdvancedFilter::FieldContains {
            field: "message".to_string(),
            value: token.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_field_equal_filter() {
        let mut entry = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            crate::model::LogLevel::Info,
            "Test message".to_string(),
            "raw line".to_string(),
        );
        entry
            .fields
            .insert("custom_field".to_string(), "custom_value".to_string());

        let filter = AdvancedFilter::FieldEqual {
            field: "custom_field".to_string(),
            value: "custom_value".to_string(),
        };

        let mut chain = AdvancedFilterChain::new().add_filter(filter);
        assert!(chain.matches(&entry));
    }

    #[test]
    fn test_query_parsing() {
        let parser = QueryParser::new();
        let mut chain = parser
            .parse_query("level==\"INFO\" AND message~\"Test\"")
            .unwrap();

        let entry = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            crate::model::LogLevel::Info,
            "Test message".to_string(),
            "raw line".to_string(),
        );

        assert!(chain.matches(&entry));
    }
}
