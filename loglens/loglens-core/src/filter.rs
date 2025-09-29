use crate::input::LogEntry;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::str::FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            "FATAL" => Ok(LogLevel::Fatal),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", s)),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

pub fn filter_logs_by_level(entries: Vec<LogEntry>, min_level: &str) -> Result<Vec<LogEntry>> {
    let min_level = min_level.parse::<LogLevel>()?;

    Ok(entries
        .into_iter()
        .filter(|entry| {
            if let Some(level_str) = &entry.level {
                if let Ok(level) = level_str.parse::<LogLevel>() {
                    level >= min_level
                } else {
                    false // Skip entries with invalid level strings
                }
            } else {
                false // Skip entries without level
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
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

        let filtered = filter_logs_by_level(entries, "WARN").unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, Some("ERROR".to_string()));
    }
}
