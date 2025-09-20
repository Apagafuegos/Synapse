use crate::model::{LogEntry, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSummary {
    pub total_entries: usize,
    pub level_counts: HashMap<LogLevel, usize>,
    pub top_errors: Vec<(String, usize)>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
}

impl LogSummary {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            level_counts: HashMap::new(),
            top_errors: Vec::new(),
            time_range: None,
        }
    }

    pub fn add_entry(&mut self, entry: &LogEntry) {
        self.total_entries += 1;

        *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;

        if entry.level == LogLevel::Error {
            let count = self
                .top_errors
                .iter()
                .find(|(msg, _)| msg == &entry.message)
                .map(|(_, c)| *c)
                .unwrap_or(0)
                + 1;

            if let Some(pos) = self
                .top_errors
                .iter()
                .position(|(msg, _)| msg == &entry.message)
            {
                self.top_errors[pos].1 = count;
            } else {
                self.top_errors.push((entry.message.clone(), count));
            }
        }

        match &mut self.time_range {
            Some((start, end)) => {
                if entry.timestamp < *start {
                    *start = entry.timestamp;
                }
                if entry.timestamp > *end {
                    *end = entry.timestamp;
                }
            }
            None => {
                self.time_range = Some((entry.timestamp, entry.timestamp));
            }
        }
    }

    pub fn finalize(&mut self, top_n: usize) {
        self.top_errors.sort_by(|a, b| b.1.cmp(&a.1));
        self.top_errors.truncate(top_n);
    }
}

pub struct LogAnalyzer {
    summary: LogSummary,
    #[allow(dead_code)]
    seen_messages: HashSet<String>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        Self {
            summary: LogSummary::new(),
            seen_messages: HashSet::new(),
        }
    }

    pub fn analyze(&mut self, entry: &LogEntry) {
        self.summary.add_entry(entry);
    }

    pub fn get_summary(&mut self, top_n: usize) -> LogSummary {
        let mut summary = LogSummary::new();
        std::mem::swap(&mut summary, &mut self.summary);
        summary.finalize(top_n);
        summary
    }
}
