use crate::input::LogEntry;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum SlimmingMode {
    /// Light slimming - basic duplicate removal and message truncation
    #[default]
    Light,
    /// Aggressive slimming - pattern-based compression and heavy deduplication
    Aggressive,
    /// Ultra aggressive - maximum compression for huge logs
    Ultra,
}


pub fn slim_logs(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    slim_logs_with_mode(entries, SlimmingMode::default())
}

/// Check if a log entry looks like a stack trace line
fn is_stack_trace_entry(entry: &LogEntry) -> bool {
    let message = entry.message.trim_start();
    message.starts_with("at ") ||
    message.starts_with("... ") ||
    message.starts_with("Caused by:") ||
    (entry.level.is_none() && message.contains("at ") && message.contains('('))
}

/// Limit stack trace entries to prevent overwhelming output
fn limit_stack_trace_entries(entries: Vec<LogEntry>, max_stack_lines: usize) -> Vec<LogEntry> {
    let mut result = Vec::new();
    let mut current_stack_count = 0;
    let mut in_stack_trace = false;

    for entry in entries {
        if is_stack_trace_entry(&entry) {
            if !in_stack_trace {
                // Starting a new stack trace
                in_stack_trace = true;
                current_stack_count = 0;
            }
            
            if current_stack_count < max_stack_lines {
                result.push(entry);
                current_stack_count += 1;
            } else if current_stack_count == max_stack_lines {
                // Add truncation marker once
                result.push(LogEntry {
                    timestamp: entry.timestamp.clone(),
                    level: Some("INFO".to_string()),
                    message: format!("... [{} stack trace lines truncated] ...", max_stack_lines),
                    line_number: entry.line_number,
                });
                current_stack_count += 1; // Prevent multiple truncation messages
            }
        } else {
            // Not a stack trace line
            in_stack_trace = false;
            current_stack_count = 0;
            result.push(entry);
        }
    }

    result
}

pub fn slim_logs_with_mode(entries: Vec<LogEntry>, mode: SlimmingMode) -> Vec<LogEntry> {
    if entries.is_empty() {
        return entries;
    }

    match mode {
        SlimmingMode::Light => {
            let max_stack_lines = 10;
            let limited = limit_stack_trace_entries(entries, max_stack_lines);
            slim_logs_light(limited)
        },
        SlimmingMode::Aggressive => {
            let max_stack_lines = 5;
            let limited = limit_stack_trace_entries(entries, max_stack_lines);
            slim_logs_aggressive(limited)
        },
        SlimmingMode::Ultra => {
            let max_stack_lines = 2;
            // Custom ultra slimming that preserves stack trace context
            slim_logs_ultra_with_stack_traces(entries, max_stack_lines)
        },
    }
}

fn slim_logs_light(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    let mut slimmed: Vec<LogEntry> = Vec::new();
    let mut consecutive_count = 1;
    let mut last_message = entries[0].message.clone();

    // Add the first entry
    let mut first_entry = entries[0].clone();
    first_entry.message = slim_message(&first_entry.message, SlimmingMode::Light);
    slimmed.push(first_entry);

    for entry in entries.into_iter().skip(1) {
        if entry.message == last_message {
            consecutive_count += 1;
        } else {
            // Update the previous entry with count if it was repeated
            if consecutive_count > 1 {
                if let Some(last_entry) = slimmed.last_mut() {
                    last_entry.message = format!(
                        "{} (repeated {} times)",
                        last_entry.message, consecutive_count
                    );
                }
            }

            // Add current entry
            let mut slimmed_entry = entry.clone();
            slimmed_entry.message = slim_message(&slimmed_entry.message, SlimmingMode::Light);
            slimmed.push(slimmed_entry);

            last_message = entry.message.clone();
            consecutive_count = 1;
        }
    }

    // Handle the last sequence
    if consecutive_count > 1 {
        if let Some(last_entry) = slimmed.last_mut() {
            last_entry.message = format!(
                "{} (repeated {} times)",
                last_entry.message, consecutive_count
            );
        }
    }

    slimmed
}

fn slim_logs_aggressive(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    // Pattern-based compression for aggressive mode
    let mut pattern_counts: HashMap<String, (usize, LogEntry)> = HashMap::new();
    let mut preserved_entries: Vec<LogEntry> = Vec::new();

    // Extract patterns and count occurrences
    for entry in entries {
        let pattern = extract_pattern(&entry.message);

        if let Some((count, _first_entry)) = pattern_counts.get_mut(&pattern) {
            *count += 1;
        } else {
            let mut slimmed_entry = entry.clone();
            slimmed_entry.message = slim_message(&slimmed_entry.message, SlimmingMode::Aggressive);
            pattern_counts.insert(pattern, (1, slimmed_entry));
        }
    }

    // Convert patterns back to entries with counts
    for (_pattern, (count, mut entry)) in pattern_counts {
        if count > 1 {
            entry.message = format!("{} (pattern repeated {} times)", entry.message, count);
        }
        preserved_entries.push(entry);
    }

    // Sort by timestamp if available
    preserved_entries.sort_by(|a, b| {
        match (&a.timestamp, &b.timestamp) {
            (Some(ts_a), Some(ts_b)) => ts_a.cmp(ts_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    preserved_entries
}

/// Ultra aggressive slimming that preserves stack trace context with error entries
fn slim_logs_ultra_with_stack_traces(entries: Vec<LogEntry>, max_stack_lines: usize) -> Vec<LogEntry> {
    let mut result = Vec::new();
    let mut preserved_errors = Vec::new();
    
    // First pass: identify ERROR/WARN/FATAL entries to preserve
    for entry in &entries {
        if let Some(level) = &entry.level {
            if level == "ERROR" || level == "WARN" || level == "FATAL" || 
               (level == "INFO" && entry.message.contains("truncated")) {
                preserved_errors.push(entry.clone());
            }
        }
    }
    
    // Second pass: add associated stack trace entries (limited)
    let mut current_stack_count = 0;
    let mut in_error_context = false;
    
    for entry in &entries {
        // Check if this is an error entry
        if let Some(level) = &entry.level {
            if level == "ERROR" || level == "WARN" || level == "FATAL" {
                result.push(entry.clone());
                in_error_context = true;
                current_stack_count = 0;
                continue;
            }
        }
        
        // Handle stack trace entries in error context
        if in_error_context && is_stack_trace_entry(entry) {
            if current_stack_count < max_stack_lines {
                result.push(entry.clone());
                current_stack_count += 1;
            } else if current_stack_count == max_stack_lines {
                // Add truncation marker once
                result.push(LogEntry {
                    timestamp: entry.timestamp.clone(),
                    level: Some("INFO".to_string()),
                    message: format!("... [stack trace limited to {} lines] ...", max_stack_lines),
                    line_number: entry.line_number,
                });
                current_stack_count += 1; // Prevent multiple truncation messages
            }
        } else {
            // Not a stack trace or no longer in error context
            in_error_context = false;
            current_stack_count = 0;
        }
    }
    
    // If no errors were found, fall back to regular ultra slimming for critical content
    if result.is_empty() {
        slim_logs_ultra(entries)
    } else {
        result
    }
}

fn slim_logs_ultra(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    // Ultra aggressive - keep only most critical errors and summaries
    let mut error_summaries: HashMap<String, usize> = HashMap::new();
    let mut critical_entries: Vec<LogEntry> = Vec::new();

    // Categorize and count errors
    for entry in entries {
        let category = categorize_error(&entry.message);

        // Always preserve CRITICAL and FATAL errors
        if is_critical_error(&entry.message) {
            let mut critical_entry = entry.clone();
            critical_entry.message = slim_message(&critical_entry.message, SlimmingMode::Ultra);
            critical_entries.push(critical_entry);
        } else {
            // Count other errors by category
            *error_summaries.entry(category).or_insert(0) += 1;
        }
    }

    // Add summary entries for non-critical errors
    for (category, count) in error_summaries {
        if count > 0 {
            critical_entries.push(LogEntry {
                timestamp: None,
                level: Some("SUMMARY".to_string()),
                message: format!("{}: {} occurrences", category, count),
                line_number: None,
            });
        }
    }

    critical_entries
}

fn slim_message(message: &str, mode: SlimmingMode) -> String {
    // Ensure the message is valid UTF-8 and fix any issues
    let sanitized_message = sanitize_string_for_utf8(message);
    let mut slimmed = sanitized_message;

    // Apply mode-specific truncation
    let max_length = match mode {
        SlimmingMode::Light => 500,
        SlimmingMode::Aggressive => 250,
        SlimmingMode::Ultra => 100,
    };

    // Truncate very long messages
    if slimmed.len() > max_length {
        slimmed.truncate(max_length);
        slimmed.push_str("...");
    }

    // Handle stack traces based on mode - detect and limit stack trace lines
    let is_stack_trace_line = slimmed.trim_start().starts_with("at ") ||
                            slimmed.trim_start().starts_with("... ") ||
                            slimmed.trim_start().starts_with("Caused by:") ||
                            (slimmed.contains("at ") && slimmed.contains('(')); // Combined trace
    
    if is_stack_trace_line {
        // For individual stack trace lines, we don't truncate the line itself
        // The limiting happens at the entry level in the calling logic
    }

    // Apply aggressive cleaning for higher modes
    match mode {
        SlimmingMode::Light => {
            slimmed = slimmed.replace("\n\n\n", "\n");
            slimmed = slimmed.replace("\t\t", "\t");
        }
        SlimmingMode::Aggressive | SlimmingMode::Ultra => {
            // Remove common noise
            slimmed = slimmed.replace("\n\n\n", "\n");
            slimmed = slimmed.replace("\t\t", "\t");
            slimmed = slimmed.replace("    ", " ");

            // Remove timestamps from within messages (keep essential info)
            if let Ok(timestamp_regex) = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}") {
                slimmed = safe_regex_replace(&slimmed, &timestamp_regex, "[TS]");
            }

            // Remove UUIDs and long hex strings
            if let Ok(uuid_regex) = Regex::new(r"[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}") {
                slimmed = safe_regex_replace(&slimmed, &uuid_regex, "[UUID]");
            }
            if let Ok(hex_regex) = Regex::new(r"\b[a-fA-F0-9]{16,}\b") {
                slimmed = safe_regex_replace(&slimmed, &hex_regex, "[HEX]");
            }
        }
    }

    slimmed.trim().to_string()
}

fn extract_pattern(message: &str) -> String {
    // Extract pattern by replacing variable parts with placeholders
    // Ensure the message is valid UTF-8 first
    let sanitized_message = sanitize_string_for_utf8(message);
    let mut pattern = sanitized_message;

    // Replace common variable patterns - order matters: specific patterns first
    if let Ok(timestamp_regex) = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}") {
        pattern = safe_regex_replace(&pattern, &timestamp_regex, "[TS]");
    }
    if let Ok(uuid_regex) = Regex::new(r"[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}") {
        pattern = safe_regex_replace(&pattern, &uuid_regex, "[UUID]");
    }
    if let Ok(path_regex) = Regex::new(r"/[^\s]+") {
        pattern = safe_regex_replace(&pattern, &path_regex, "[PATH]");
    }
    if let Ok(ip_regex) = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b") {
        pattern = safe_regex_replace(&pattern, &ip_regex, "[IP]");
    }
    // Apply general number pattern last to avoid interfering with specific patterns
    if let Ok(number_regex) = Regex::new(r"\b\d+\b") {
        pattern = safe_regex_replace(&pattern, &number_regex, "[NUM]");
    }

    pattern.trim().to_string()
}

fn categorize_error(message: &str) -> String {
    let message_lower = message.to_lowercase();

    if message_lower.contains("database") || message_lower.contains("sql") || message_lower.contains("connection") {
        "Database Errors".to_string()
    } else if message_lower.contains("network") || message_lower.contains("timeout") || message_lower.contains("connection refused") {
        "Network Errors".to_string()
    } else if message_lower.contains("memory") || message_lower.contains("out of memory") || message_lower.contains("heap") {
        "Memory Errors".to_string()
    } else if message_lower.contains("permission") || message_lower.contains("access denied") || message_lower.contains("unauthorized") {
        "Permission Errors".to_string()
    } else if message_lower.contains("file not found") || message_lower.contains("no such file") || message_lower.contains("directory") {
        "File System Errors".to_string()
    } else if message_lower.contains("exception") || message_lower.contains("error") {
        "Application Errors".to_string()
    } else {
        "Other Messages".to_string()
    }
}

fn is_critical_error(message: &str) -> bool {
    let message_lower = message.to_lowercase();

    message_lower.contains("fatal") ||
    message_lower.contains("critical") ||
    message_lower.contains("severe") ||
    message_lower.contains("panic") ||
    message_lower.contains("segmentation fault") ||
    message_lower.contains("out of memory") ||
    message_lower.contains("stack overflow") ||
    message_lower.contains("corrupted") ||
    message_lower.contains("data loss") ||
    message_lower.contains("security")
}

/// Sanitize a string to ensure it's valid UTF-8, replacing invalid sequences
fn sanitize_string_for_utf8(input: &str) -> String {
    // If the string is already valid UTF-8, return as-is
    if input.is_ascii() || input.chars().all(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r')) {
        return input.to_string();
    }

    // Convert to bytes and back, replacing invalid UTF-8 sequences
    let bytes = input.as_bytes();
    String::from_utf8_lossy(bytes).to_string()
}

/// Safely perform regex replacement on potentially problematic strings
fn safe_regex_replace(input: &str, regex: &Regex, replacement: &str) -> String {
    // Sanitize input first
    let sanitized = sanitize_string_for_utf8(input);

    // Perform replacement with error handling
    match std::panic::catch_unwind(|| {
        regex.replace_all(&sanitized, replacement).to_string()
    }) {
        Ok(result) => result,
        Err(_) => {
            // If regex fails, return the sanitized input
            eprintln!("Warning: Regex operation failed on string, returning sanitized input");
            sanitized
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slim_logs_no_duplicates() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-01T12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Database connection failed".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:01".to_string()),
                level: Some("INFO".to_string()),
                message: "User logged in".to_string(),
                line_number: Some(2),
            },
        ];

        let slimmed = slim_logs(entries);
        assert_eq!(slimmed.len(), 2);
        assert_eq!(slimmed[0].message, "Database connection failed");
        assert_eq!(slimmed[1].message, "User logged in");
    }

    #[test]
    fn test_slim_logs_with_duplicates() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-01T12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Connection timeout".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:01".to_string()),
                level: Some("ERROR".to_string()),
                message: "Connection timeout".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:02".to_string()),
                level: Some("ERROR".to_string()),
                message: "Connection timeout".to_string(),
                line_number: Some(3),
            },
        ];

        let slimmed = slim_logs(entries);
        assert_eq!(slimmed.len(), 1);
        assert!(slimmed[0].message.contains("(repeated 3 times)"));
    }

    #[test]
    fn test_slim_long_message() {
        let long_message = "a".repeat(600);
        let entries = vec![LogEntry {
            timestamp: Some("2024-01-01T12:00:00".to_string()),
            level: Some("ERROR".to_string()),
            message: long_message,
            line_number: Some(1),
        }];

        let slimmed = slim_logs(entries);
        assert_eq!(slimmed.len(), 1);
        assert!(slimmed[0].message.len() <= 503); // 500 + "..."
        assert!(slimmed[0].message.ends_with("..."));
    }

    #[test]
    fn test_aggressive_slimming() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-01T12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Database connection failed to 192.168.1.100:5432".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:01".to_string()),
                level: Some("ERROR".to_string()),
                message: "Database connection failed to 192.168.1.200:5432".to_string(),
                line_number: Some(2),
            },
        ];

        let slimmed = slim_logs_with_mode(entries, SlimmingMode::Aggressive);
        assert_eq!(slimmed.len(), 1);
        assert!(slimmed[0].message.contains("(pattern repeated 2 times)"));
    }

    #[test]
    fn test_ultra_slimming() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2024-01-01T12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "FATAL: System crashed".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:01".to_string()),
                level: Some("INFO".to_string()),
                message: "User logged in".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: Some("2024-01-01T12:00:02".to_string()),
                level: Some("DEBUG".to_string()),
                message: "Processing request".to_string(),
                line_number: Some(3),
            },
        ];

        let slimmed = slim_logs_with_mode(entries, SlimmingMode::Ultra);

        // Should preserve the ERROR entry (it contains "FATAL" text)
        assert!(!slimmed.is_empty());
        assert!(slimmed.iter().any(|entry| entry.level.as_deref() == Some("ERROR")));
        
        // Should have some entries (either the ERROR itself or summaries if it falls back)
        assert!(slimmed.len() > 0);
    }

    #[test]
    fn test_pattern_extraction() {
        let message = "Error 404: File /var/log/app-12345.log not found at 2024-01-01T12:00:00";
        let pattern = extract_pattern(message);

        println!("Original: {}", message);
        println!("Pattern:  {}", pattern);

        assert!(pattern.contains("[NUM]"));
        assert!(pattern.contains("[PATH]"));
        assert!(pattern.contains("[TS]"));
    }

    #[test]
    fn test_stack_trace_limiting() {
        let entries = vec![
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Java exception occurred".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Method.method(Method.java:123)".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Another.method(Another.java:456)".to_string(),
                line_number: Some(3),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Third.method(Third.java:789)".to_string(),
                line_number: Some(4),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:01:00".to_string()),
                level: Some("INFO".to_string()),
                message: "Regular log message".to_string(),
                line_number: Some(5),
            },
        ];

        // Test with Ultra mode (2 lines max, should limit to 2 stack trace lines)
        let slimmed_ultra = slim_logs_with_mode(entries.clone(), SlimmingMode::Ultra);
        
        // Should preserve ERROR entry and limit stack trace to 2 lines + truncation message
        let stack_trace_entries_ultra: Vec<_> = slimmed_ultra.iter()
            .filter(|e| is_stack_trace_entry(e))
            .collect();
        assert_eq!(stack_trace_entries_ultra.len(), 3); // 2 stack trace + 1 truncation message
        
        // Should have the truncation message
        let has_truncation: bool = slimmed_ultra.iter()
            .any(|e| e.level.as_deref() == Some("INFO") && e.message.contains("limited"));
        assert!(has_truncation); // Should have truncation message
        
        // Should preserve the ERROR entry
        assert!(slimmed_ultra.iter().any(|e| e.level.as_deref() == Some("ERROR")));
    }

    #[test]
    fn test_stack_trace_entry_detection() {
        let entries = vec![
            LogEntry {
                timestamp: None,
                level: None,
                message: "    at com.example.Method.method(Method.java:123)".to_string(),
                line_number: Some(1),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    ... 23 more".to_string(),
                line_number: Some(2),
            },
            LogEntry {
                timestamp: None,
                level: None,
                message: "    Caused by: java.sql.SQLException".to_string(),
                line_number: Some(3),
            },
            LogEntry {
                timestamp: Some("2025-01-01 12:00:00".to_string()),
                level: Some("ERROR".to_string()),
                message: "Regular error message".to_string(),
                line_number: Some(4),
            },
        ];

        assert!(is_stack_trace_entry(&entries[0])); // "at " line
        assert!(is_stack_trace_entry(&entries[1])); // "... " line  
        assert!(is_stack_trace_entry(&entries[2])); // "Caused by:" line
        assert!(!is_stack_trace_entry(&entries[3])); // Regular log entry
    }
}
