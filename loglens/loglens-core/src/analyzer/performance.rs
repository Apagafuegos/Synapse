use crate::input::LogEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub error_frequency: Vec<TimeSeriesPoint>,
    pub timing_statistics: TimingStats,
    pub bottlenecks: Vec<Bottleneck>,
    pub performance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: String,
    pub count: usize,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    pub average_interval: f64,
    pub min_interval: f64,
    pub max_interval: f64,
    pub median_interval: f64,
    pub total_duration: f64,
    pub error_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub description: String,
    pub severity: String,
    pub impact_score: f32,
    pub suggestions: Vec<String>,
}

pub struct PerformanceAnalyzer {
    time_series: HashMap<String, Vec<TimeSeriesPoint>>,
    intervals: Vec<f64>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            time_series: HashMap::new(),
            intervals: Vec::new(),
        }
    }
    
    pub fn analyze(&mut self, logs: &[LogEntry]) -> Result<PerformanceMetrics> {
        self.extract_time_series(logs);
        self.calculate_intervals(logs);
        
        Ok(PerformanceMetrics {
            error_frequency: self.get_error_frequency(),
            timing_statistics: self.get_timing_stats(),
            bottlenecks: self.detect_bottlenecks(),
            performance_score: self.calculate_performance_score(),
        })
    }
    
    fn extract_time_series(&mut self, logs: &[LogEntry]) {
        // Group logs by time windows (e.g., per minute)
        let mut time_windows: HashMap<String, HashMap<String, usize>> = HashMap::new();
        
        for log in logs {
            if let Some(timestamp) = &log.timestamp {
                let window = self.get_time_window(timestamp);
                let level = log.level.clone().unwrap_or_else(|| "UNKNOWN".to_string());
                
                *time_windows.entry(window.clone()).or_insert_with(HashMap::new)
                    .entry(level).or_insert(0) += 1;
            }
        }
        
        // Convert to time series points
        for (window, counts) in time_windows {
            for (level, count) in counts {
                let point = TimeSeriesPoint {
                    timestamp: window.clone(),
                    count,
                    level: level.clone(),
                };
                
                self.time_series
                    .entry(level.clone())
                    .or_insert_with(Vec::new)
                    .push(point);
            }
        }
        
        // Sort time series by timestamp
        for series in self.time_series.values_mut() {
            series.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
    }
    
    fn calculate_intervals(&mut self, logs: &[LogEntry]) {
        // Calculate intervals between consecutive errors
        let mut error_timestamps: Vec<&str> = logs
            .iter()
            .filter(|log| log.level.as_ref().map_or(false, |l| l == "ERROR"))
            .filter_map(|log| log.timestamp.as_deref())
            .collect();
        
        error_timestamps.sort();
        
        self.intervals = error_timestamps
            .windows(2)
            .filter_map(|window| {
                if let (Some(ts1), Some(ts2)) = (self.parse_timestamp(window[0]), self.parse_timestamp(window[1])) {
                    Some((ts2 - ts1).as_seconds_f64())
                } else {
                    None
                }
            })
            .collect();
    }
    
    fn get_error_frequency(&self) -> Vec<TimeSeriesPoint> {
        self.time_series
            .get("ERROR")
            .cloned()
            .unwrap_or_default()
    }
    
    fn get_timing_stats(&self) -> TimingStats {
        if self.intervals.is_empty() {
            return TimingStats {
                average_interval: 0.0,
                min_interval: 0.0,
                max_interval: 0.0,
                median_interval: 0.0,
                total_duration: 0.0,
                error_rate: 0.0,
            };
        }
        
        let total_duration = self.intervals.iter().sum::<f64>();
        let average_interval = total_duration / self.intervals.len() as f64;
        let min_interval = self.intervals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_interval = self.intervals.iter().fold(0.0_f64, |a, b| a.max(*b));
        
        let mut sorted_intervals = self.intervals.clone();
        sorted_intervals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_interval = if sorted_intervals.is_empty() {
            0.0
        } else if sorted_intervals.len() % 2 == 0 {
            (sorted_intervals[sorted_intervals.len() / 2 - 1] + sorted_intervals[sorted_intervals.len() / 2]) / 2.0
        } else {
            sorted_intervals[sorted_intervals.len() / 2]
        };
        
        // Calculate error rate (errors per minute)
        let error_rate = if total_duration > 0.0 {
            (self.intervals.len() as f32) / (total_duration / 60.0) as f32
        } else {
            0.0
        };
        
        TimingStats {
            average_interval,
            min_interval,
            max_interval,
            median_interval,
            total_duration,
            error_rate,
        }
    }
    
    fn detect_bottlenecks(&self) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Detect high error frequency bottlenecks
        if let Some(error_series) = self.time_series.get("ERROR") {
            for point in error_series {
                if point.count > 10 { // More than 10 errors in a time window
                    bottlenecks.push(Bottleneck {
                        component: "System".to_string(),
                        description: format!("High error frequency detected: {} errors in {}", point.count, point.timestamp),
                        severity: "HIGH".to_string(),
                        impact_score: (point.count as f32) / 10.0,
                        suggestions: vec![
                            "Check system resources".to_string(),
                            "Review recent changes".to_string(),
                            "Consider scaling resources".to_string(),
                        ],
                    });
                }
            }
        }
        
        // Detect timing bottlenecks
        if let Some(stats) = self.timing_stats_for_bottlenecks() {
            if stats.average_interval < 5.0 { // Errors happening very frequently
                bottlenecks.push(Bottleneck {
                    component: "Error Frequency".to_string(),
                    description: "Errors occurring too frequently, indicating system instability".to_string(),
                    severity: "MEDIUM".to_string(),
                    impact_score: 1.0 / stats.average_interval as f32,
                    suggestions: vec![
                        "Implement rate limiting".to_string(),
                        "Add circuit breakers".to_string(),
                        "Review error handling logic".to_string(),
                    ],
                });
            }
        }
        
        bottlenecks.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());
        bottlenecks
    }
    
    fn calculate_performance_score(&self) -> f32 {
        let mut score = 100.0;
        
        // Deduct for bottlenecks
        for bottleneck in &self.detect_bottlenecks() {
            score -= bottleneck.impact_score * 10.0;
        }
        
        // Deduct for high error rate
        if let Some(stats) = self.timing_stats_for_bottlenecks() {
            if stats.error_rate > 1.0 { // More than 1 error per minute
                score -= (stats.error_rate - 1.0) * 20.0;
            }
        }
        
        score.max(0.0).min(100.0)
    }
    
    fn get_time_window(&self, timestamp: &str) -> String {
        // Extract minute-level time windows
        if let Some(pos) = timestamp.find('T') {
            let time_part = &timestamp[pos + 1..];
            if let Some(min_pos) = time_part.find(':') {
                return format!("{}:{}", &timestamp[..pos + 1], &time_part[..min_pos]);
            }
        }
        timestamp.to_string()
    }
    
    fn parse_timestamp(&self, timestamp: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        // Try different timestamp formats
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
    
    fn timing_stats_for_bottlenecks(&self) -> Option<TimingStats> {
        if self.intervals.is_empty() {
            return None;
        }
        
        let total_duration = self.intervals.iter().sum::<f64>();
        let average_interval = total_duration / self.intervals.len() as f64;
        let error_rate = if total_duration > 0.0 {
            (self.intervals.len() as f32) / (total_duration / 60.0) as f32
        } else {
            0.0
        };
        
        Some(TimingStats {
            average_interval,
            min_interval: 0.0,
            max_interval: 0.0,
            median_interval: 0.0,
            total_duration,
            error_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::LogEntry;
    
    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new();
        assert!(analyzer.time_series.is_empty());
        assert!(analyzer.intervals.is_empty());
    }
    
    #[test]
    fn test_get_time_window() {
        let analyzer = PerformanceAnalyzer::new();
        let timestamp = "2024-01-01T10:30:45Z";
        let window = analyzer.get_time_window(timestamp);
        
        assert_eq!(window, "2024-01-01T10:30");
    }
    
    #[test]
    fn test_analyze_logs() {
        let mut analyzer = PerformanceAnalyzer::new();
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
        
        assert!(!result.error_frequency.is_empty());
        assert!(result.timing_statistics.total_duration > 0.0);
        assert!(result.performance_score <= 100.0);
    }
    
    #[test]
    fn test_parse_timestamp() {
        let analyzer = PerformanceAnalyzer::new();
        
        // Test ISO format
        let result = analyzer.parse_timestamp("2024-01-01T10:30:45Z");
        assert!(result.is_some());
        
        // Test space format
        let result = analyzer.parse_timestamp("2024-01-01 10:30:45");
        assert!(result.is_some());
        
        // Test invalid format
        let result = analyzer.parse_timestamp("invalid-timestamp");
        assert!(result.is_none());
    }
}