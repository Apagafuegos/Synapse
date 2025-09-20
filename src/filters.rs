use crate::model::{LogEntry, LogLevel};
use chrono::{DateTime, Utc};
use regex::Regex;

pub trait LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool;
}

pub struct LevelFilter {
    level: LogLevel,
}

impl LevelFilter {
    pub fn new(level: &str) -> Result<Self, String> {
        let parsed_level = level.parse::<LogLevel>()
            .map_err(|_| format!("Invalid log level: {}", level))?;
        Ok(Self { level: parsed_level })
    }
}

impl LogFilter for LevelFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        entry.level == self.level
    }
}

pub struct TimeRangeFilter {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeRangeFilter {
    pub fn new(start_str: &str, end_str: &str) -> Result<Self, String> {
        let start = parse_datetime(start_str)
            .ok_or_else(|| format!("Invalid start time format: {}", start_str))?;
        let end = parse_datetime(end_str)
            .ok_or_else(|| format!("Invalid end time format: {}", end_str))?;
        
        if start > end {
            return Err("Start time must be before end time".to_string());
        }
        
        Ok(Self { start, end })
    }
}

impl LogFilter for TimeRangeFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        entry.timestamp >= self.start && entry.timestamp <= self.end
    }
}

pub struct PatternFilter {
    regex: Regex,
}

impl PatternFilter {
    pub fn new(pattern: &str) -> Result<Self, String> {
        let regex = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern: {}", e))?;
        Ok(Self { regex })
    }
}

impl LogFilter for PatternFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        self.regex.is_match(&entry.message) || 
        self.regex.is_match(&entry.raw_line)
    }
}

pub struct FilterChain {
    filters: Vec<Box<dyn LogFilter>>,
}

impl FilterChain {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }
    
    pub fn add_filter(mut self, filter: Box<dyn LogFilter>) -> Self {
        self.filters.push(filter);
        self
    }
    
    pub fn matches(&self, entry: &LogEntry) -> bool {
        self.filters.iter().all(|filter| filter.matches(entry))
    }
}

fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .or_else(|_| DateTime::parse_from_rfc2822(s))
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}