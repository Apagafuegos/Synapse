use crate::model::{LogEntry, LogLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub entry: LogEntry,
    pub score: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCluster {
    pub pattern: String,
    pub entries: Vec<LogEntry>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub timeline: Vec<TimelinePoint>,
    pub level_distribution: HashMap<String, usize>,
    pub patterns: Vec<PatternInfo>,
    pub anomalies: Vec<AnomalyResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub timestamp: DateTime<Utc>,
    pub count: usize,
    pub level_breakdown: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub pattern: String,
    pub frequency: usize,
    pub examples: Vec<String>,
}

pub trait AnomalyDetector {
    fn detect_anomalies(&self, threshold: f64) -> Vec<AnomalyResult>;
}

pub trait PatternClusterer {
    fn cluster_patterns(&self, num_clusters: usize) -> Vec<PatternCluster>;
}

#[derive(Debug)]
pub struct LogAnalytics {
    entries: Vec<LogEntry>,
    raw_lines: Vec<RawLineData>,
    timestamp_stats: TimestampStatistics,
    level_stats: LevelStatistics,
    pattern_stats: PatternStatistics,
}

#[derive(Debug, Clone)]
struct RawLineData {
    #[allow(dead_code)]
    line: String,
    #[allow(dead_code)]
    line_number: usize,
    #[allow(dead_code)]
    timestamp_hint: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    level_hint: Option<LogLevel>,
}

#[derive(Debug, Clone)]
struct TimestampStatistics {
    min_time: Option<DateTime<Utc>>,
    max_time: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    avg_interval: f64,
    #[allow(dead_code)]
    intervals: Vec<f64>,
}

#[derive(Debug, Clone)]
struct LevelStatistics {
    level_counts: HashMap<LogLevel, usize>,
    total_entries: usize,
}

#[derive(Debug, Clone)]
struct PatternStatistics {
    pattern_counts: HashMap<String, usize>,
    #[allow(dead_code)]
    common_patterns: Vec<String>,
}

impl LogAnalytics {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            raw_lines: Vec::new(),
            timestamp_stats: TimestampStatistics {
                min_time: None,
                max_time: None,
                avg_interval: 0.0,
                intervals: Vec::new(),
            },
            level_stats: LevelStatistics {
                level_counts: HashMap::new(),
                total_entries: 0,
            },
            pattern_stats: PatternStatistics {
                pattern_counts: HashMap::new(),
                common_patterns: Vec::new(),
            },
        }
    }

    pub fn analyze_entry(&mut self, entry: &LogEntry) {
        self.entries.push(entry.clone());
        self.update_timestamp_stats(&entry.timestamp);
        self.update_level_stats(&entry.level);
        self.update_pattern_stats(&entry.message);
    }

    pub fn analyze_raw_line(
        &mut self,
        line: &str,
        line_number: usize,
        timestamp_hint: Option<DateTime<Utc>>,
        level_hint: Option<LogLevel>,
    ) {
        self.raw_lines.push(RawLineData {
            line: line.to_string(),
            line_number,
            timestamp_hint,
            level_hint,
        });
    }

    fn update_timestamp_stats(&mut self, timestamp: &DateTime<Utc>) {
        match self.timestamp_stats.min_time {
            None => {
                self.timestamp_stats.min_time = Some(*timestamp);
                self.timestamp_stats.max_time = Some(*timestamp);
            }
            Some(min_time) => {
                if timestamp < &min_time {
                    self.timestamp_stats.min_time = Some(*timestamp);
                }
                if timestamp > &self.timestamp_stats.max_time.unwrap() {
                    self.timestamp_stats.max_time = Some(*timestamp);
                }
            }
        }
    }

    fn update_level_stats(&mut self, level: &LogLevel) {
        *self.level_stats.level_counts.entry(level.clone()).or_insert(0) += 1;
        self.level_stats.total_entries += 1;
    }

    fn update_pattern_stats(&mut self, message: &str) {
        // Extract simple patterns (first 50 chars, normalized)
        let pattern = self.extract_pattern(message);
        *self.pattern_stats.pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
    }

    fn extract_pattern(&self, message: &str) -> String {
        // Simple pattern extraction: normalize numbers and timestamps
        let normalized = message
            .chars()
            .map(|c| {
                if c.is_ascii_digit() {
                    '0' // Replace all digits with 0
                } else {
                    c
                }
            })
            .collect::<String>();

        // Take first 50 characters for pattern
        normalized.chars().take(50).collect()
    }

    pub fn detect_anomalies(&self, threshold: f64) -> Vec<AnomalyResult> {
        let mut anomalies = Vec::new();

        // Statistical anomaly detection based on timestamp intervals
        if self.entries.len() > 10 {
            let intervals = self.calculate_intervals();
            if let Some((mean, std_dev)) = self.calculate_mean_std(&intervals) {
                for (i, &interval) in intervals.iter().enumerate() {
                    let z_score = (interval - mean).abs() / std_dev;
                    if z_score > threshold && i < self.entries.len() {
                        anomalies.push(AnomalyResult {
                            entry: self.entries[i + 1].clone(),
                            score: z_score,
                            message: format!("Unusual interval: {:.2}s (z-score: {:.2})", interval, z_score),
                        });
                    }
                }
            }
        }

        // Level-based anomaly detection (sudden spikes in error rates)
        if self.entries.len() > 100 {
            if let Some(anomaly) = self.detect_level_anomaly(threshold) {
                anomalies.push(anomaly);
            }
        }

        anomalies.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        anomalies
    }

    fn calculate_intervals(&self) -> Vec<f64> {
        let mut intervals = Vec::new();
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by_key(|e| e.timestamp);

        for i in 1..sorted_entries.len() {
            let interval = (sorted_entries[i].timestamp - sorted_entries[i-1].timestamp).num_seconds() as f64;
            intervals.push(interval);
        }

        intervals
    }

    fn calculate_mean_std(&self, values: &[f64]) -> Option<(f64, f64)> {
        if values.is_empty() {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        Some((mean, std_dev))
    }

    fn detect_level_anomaly(&self, threshold: f64) -> Option<AnomalyResult> {
        // Check for sudden spikes in error rates
        let window_size = self.entries.len() / 10;
        if window_size < 10 {
            return None;
        }

        let recent_errors = self.entries[self.entries.len() - window_size..]
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .count();

        let earlier_errors = self.entries[..self.entries.len() - window_size]
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .count();

        let recent_rate = recent_errors as f64 / window_size as f64;
        let earlier_rate = earlier_errors as f64 / (self.entries.len() - window_size) as f64;

        if earlier_rate > 0.0 {
            let ratio = recent_rate / earlier_rate;
            if ratio > threshold {
                return Some(AnomalyResult {
                    entry: self.entries.last().unwrap().clone(),
                    score: ratio,
                    message: format!("Error rate spike: {:.2}x increase", ratio),
                });
            }
        }

        None
    }

    pub fn cluster_patterns(&self, num_clusters: usize) -> Vec<PatternCluster> {
        let mut clusters = Vec::new();
        
        // Simple clustering based on pattern similarity
        let mut patterns: Vec<(String, usize)> = self.pattern_stats.pattern_counts
            .iter()
            .map(|(pattern, count)| (pattern.clone(), *count))
            .collect();

        patterns.sort_by(|a, b| b.1.cmp(&a.1));

        // Group similar patterns
        let mut used_patterns = HashSet::new();
        for (pattern, count) in patterns {
            if used_patterns.contains(&pattern) {
                continue;
            }

            let mut cluster_entries = Vec::new();
            let mut similar_patterns = Vec::new();

            // Find entries with similar patterns
            for entry in &self.entries {
                let entry_pattern = self.extract_pattern(&entry.message);
                if self.patterns_similar(&pattern, &entry_pattern) {
                    cluster_entries.push(entry.clone());
                    similar_patterns.push(entry_pattern.clone());
                    used_patterns.insert(entry_pattern);
                }
            }

            if !cluster_entries.is_empty() {
                clusters.push(PatternCluster {
                    pattern: pattern.clone(),
                    entries: cluster_entries,
                    confidence: count as f64 / self.entries.len() as f64,
                });
            }

            if clusters.len() >= num_clusters {
                break;
            }
        }

        clusters
    }

    fn patterns_similar(&self, pattern1: &str, pattern2: &str) -> bool {
        // Simple similarity check: Levenshtein distance would be better
        if pattern1.is_empty() || pattern2.is_empty() {
            return false;
        }

        // Check if one pattern is substring of the other (with some tolerance)
        let min_len = pattern1.len().min(pattern2.len());
        if min_len < 10 {
            return pattern1 == pattern2;
        }

        let common_chars = pattern1.chars()
            .filter(|&c| pattern2.contains(c))
            .count();

        let similarity = common_chars as f64 / pattern1.len().max(pattern2.len()) as f64;
        similarity > 0.7 // 70% similarity threshold
    }

    pub fn get_visualization_data(&self) -> VisualizationData {
        let timeline = self.generate_timeline();
        let level_distribution = self.generate_level_distribution();
        let patterns = self.generate_pattern_info();
        let anomalies = self.detect_anomalies(2.0); // Default threshold

        VisualizationData {
            timeline,
            level_distribution,
            patterns,
            anomalies,
        }
    }

    fn generate_timeline(&self) -> Vec<TimelinePoint> {
        let mut timeline = HashMap::new();
        
        for entry in &self.entries {
            // Group by minute
            let time_key = entry.timestamp.timestamp() / 60;
            let point = timeline.entry(time_key).or_insert(TimelinePoint {
                timestamp: entry.timestamp,
                count: 0,
                level_breakdown: HashMap::new(),
            });
            
            point.count += 1;
            *point.level_breakdown.entry(entry.level.to_string()).or_insert(0) += 1;
        }

        let mut timeline_vec: Vec<_> = timeline.into_values().collect();
        timeline_vec.sort_by_key(|p| p.timestamp);
        timeline_vec
    }

    fn generate_level_distribution(&self) -> HashMap<String, usize> {
        self.level_stats.level_counts
            .iter()
            .map(|(level, count)| (level.to_string(), *count))
            .collect()
    }

    fn generate_pattern_info(&self) -> Vec<PatternInfo> {
        let mut patterns: Vec<_> = self.pattern_stats.pattern_counts
            .iter()
            .map(|(pattern, &frequency)| {
                let examples = self.entries
                    .iter()
                    .filter(|e| self.extract_pattern(&e.message) == *pattern)
                    .take(3)
                    .map(|e| e.message.clone())
                    .collect();

                PatternInfo {
                    pattern: pattern.clone(),
                    frequency,
                    examples,
                }
            })
            .collect();

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(10); // Top 10 patterns
        patterns
    }
}

impl AnomalyDetector for LogAnalytics {
    fn detect_anomalies(&self, threshold: f64) -> Vec<AnomalyResult> {
        self.detect_anomalies(threshold)
    }
}

impl PatternClusterer for LogAnalytics {
    fn cluster_patterns(&self, num_clusters: usize) -> Vec<PatternCluster> {
        self.cluster_patterns(num_clusters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_analytics_basic() {
        let mut analytics = LogAnalytics::new();
        
        let entry1 = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap(),
            LogLevel::Info,
            "Test message 1".to_string(),
            "raw line 1".to_string(),
        );
        
        let entry2 = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 1, 0).unwrap(),
            LogLevel::Error,
            "Test message 2".to_string(),
            "raw line 2".to_string(),
        );

        analytics.analyze_entry(&entry1);
        analytics.analyze_entry(&entry2);

        assert_eq!(analytics.entries.len(), 2);
        assert_eq!(analytics.level_stats.total_entries, 2);
        assert_eq!(*analytics.level_stats.level_counts.get(&LogLevel::Info).unwrap(), 1);
        assert_eq!(*analytics.level_stats.level_counts.get(&LogLevel::Error).unwrap(), 1);
    }

    #[test]
    fn test_pattern_extraction() {
        let analytics = LogAnalytics::new();
        
        let pattern1 = analytics.extract_pattern("User 123 logged in from 192.168.1.1");
        let pattern2 = analytics.extract_pattern("User 456 logged in from 192.168.1.2");
        
        // Patterns should be similar after normalization
        assert!(analytics.patterns_similar(&pattern1, &pattern2));
    }

    #[test]
    fn test_anomaly_detection() {
        let mut analytics = LogAnalytics::new();
        
        // Create entries with normal intervals
        for i in 0..10 {
            let entry = LogEntry::new(
                Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, i).unwrap(),
                LogLevel::Info,
                format!("Message {}", i),
                format!("raw line {}", i),
            );
            analytics.analyze_entry(&entry);
        }

        // Add entry with unusual interval
        let anomaly_entry = LogEntry::new(
            Utc.with_ymd_and_hms(2023, 1, 1, 12, 10, 0).unwrap(), // 10 minute gap
            LogLevel::Info,
            "Anomaly message".to_string(),
            "anomaly line".to_string(),
        );
        analytics.analyze_entry(&anomaly_entry);

        let anomalies = analytics.detect_anomalies(2.0);
        assert!(!anomalies.is_empty());
    }
}