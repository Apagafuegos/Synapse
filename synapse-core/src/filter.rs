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
    let mut result = Vec::new();
    let mut stack_trace_count = usize::MAX; // Start with MAX to skip orphaned lines at start
    const MAX_STACK_TRACE_LINES: usize = 10;

    for entry in entries {
        if let Some(level_str) = &entry.level {
            // Entry has a log level - check if it passes the filter
            if let Ok(level) = level_str.parse::<LogLevel>() {
                if level >= min_level {
                    result.push(entry);
                    stack_trace_count = 0; // Reset counter for next stack trace
                } else {
                    stack_trace_count = usize::MAX; // Skip stack traces after filtered-out entries
                }
            } else {
                stack_trace_count = usize::MAX; // Skip stack traces after invalid levels
            }
        } else {
            // No log level - this is a stack trace line
            // Only keep if we're within the first 10 lines after an ERROR/WARN
            if stack_trace_count < MAX_STACK_TRACE_LINES {
                result.push(entry);
                stack_trace_count += 1;
            }
        }
    }

    Ok(result)
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
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("DEBUG".to_string()),
                message: "Debug message".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:01:00".to_string()),
                level: Some("WARN".to_string()),
                message: "Warning message".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:02:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Error message".to_string(),
                line_number: Some(3),
            },
        ];

        let filtered = filter_logs_by_level(entries, "WARN").unwrap();
        assert_eq!(filtered.len(), 2); // WARN and ERROR should pass
        assert_eq!(filtered[0].level, Some("WARN".to_string()));
        assert_eq!(filtered[1].level, Some("ERROR".to_string()));
    }

    #[test]
    fn test_stack_trace_limit() {
        // Test that only first 10 stack trace lines are kept after ERROR
        let mut entries = vec![
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "java.lang.NullPointerException: Null error".to_string(),
                line_number: Some(1),
            },
        ];

        // Add 15 stack trace lines
        for i in 0..15 {
            entries.push(LogEntry {
                timestamp: None,
                level: None,
                message: format!("    at com.example.Method{}(File.java:{})", i, i),
                line_number: Some(i + 2),
            });
        }

        entries.push(LogEntry {
            timestamp: Some("2025-01-01 12:01:00".to_string()),
            level: Some("INFO".to_string()),
            message: "Info message".to_string(),
            line_number: Some(18),
        });

        let filtered = filter_logs_by_level(entries, "WARN").unwrap();
        assert_eq!(filtered.len(), 11); // ERROR + 10 stack trace lines (not all 15)
        assert_eq!(filtered[0].level, Some("ERROR".to_string()));
        for i in 1..=10 {
            assert_eq!(filtered[i].level, None); // First 10 stack trace lines
        }
    }

    #[test]
    fn test_stack_trace_after_error() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "com.comerzzia.api.core.service.exception.NotFoundException:".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.comerzzia.api.isla.oms.service.servicehandle.ServiceHandleServiceImpl.selectByHandleUser(ServiceHandleServiceImpl.java:191)".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at org.springframework.cglib.proxy.MethodProxy.invoke(MethodProxy.java:218)".to_string(),
                line_number: Some(3),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:01:00".to_string()),
                level: Some("INFO".to_string()),
                message: "Info message that should be filtered out".to_string(),
                line_number: Some(4),
            },
        ];

        let filtered = filter_logs_by_level(entries, "WARN").unwrap();
        assert_eq!(filtered.len(), 3); // ERROR + 2 stack trace lines
        assert_eq!(filtered[0].level, Some("ERROR".to_string()));
        assert_eq!(filtered[1].level, None); // Stack trace line
        assert_eq!(filtered[2].level, None); // Stack trace line
    }

    #[test]
    fn test_orphaned_stack_traces_filtered() {
        // Stack traces without a preceding ERROR/WARN should be dropped
        let entries = vec![
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("INFO".to_string()),
                message: "Info message".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Method.call(Method.java:123)".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.InnerClass.run(InnerClass.java:456)".to_string(),
                line_number: Some(3),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:01:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Error occurred".to_string(),
                line_number: Some(4),
            },
        ];

        let filtered = filter_logs_by_level(entries, "ERROR").unwrap();
        assert_eq!(filtered.len(), 1); // Only the ERROR, orphaned stack traces dropped
        assert_eq!(filtered[0].level, Some("ERROR".to_string()));
    }

    #[test]
    fn test_multiple_errors_with_stack_traces() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("DEBUG".to_string()),
                message: "Debug message - should be filtered".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:00:01".to_string()),
                level: Some("ERROR".to_string()),
                message: "First error".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.First.method(First.java:10)".to_string(),
                line_number: Some(3),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:00:02".to_string()),
                level: Some("WARN".to_string()),
                message: "Warning message".to_string(),
                line_number: Some(4),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Warn.method(Warn.java:20)".to_string(),
                line_number: Some(5),
            },
        ];

        let filtered = filter_logs_by_level(entries, "ERROR").unwrap();
        assert_eq!(filtered.len(), 2); // ERROR + its stack trace (WARN filtered out)
        assert_eq!(filtered[0].level, Some("ERROR".to_string()));
        assert_eq!(filtered[1].level, None);
    }
}
