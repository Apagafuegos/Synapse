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

pub fn slim_logs_with_mode(entries: Vec<LogEntry>, mode: SlimmingMode) -> Vec<LogEntry> {
    if entries.is_empty() {
        return entries;
    }

    match mode {
        SlimmingMode::Light => slim_logs_light(entries),
        SlimmingMode::Aggressive => slim_logs_aggressive(entries),
        SlimmingMode::Ultra => slim_logs_ultra(entries),
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

    // Handle stack traces based on mode
    if slimmed.contains("at ") && slimmed.contains('(') {
        let lines: Vec<&str> = slimmed.lines().collect();
        let keep_lines = match mode {
            SlimmingMode::Light => 10,
            SlimmingMode::Aggressive => 5,
            SlimmingMode::Ultra => 2,
        };

        if lines.len() > keep_lines {
            let keep_top = match mode {
                SlimmingMode::Light => 3,
                SlimmingMode::Aggressive => 2,
                SlimmingMode::Ultra => 1,
            };
            let keep_bottom = match mode {
                SlimmingMode::Light => 2,
                SlimmingMode::Aggressive => 1,
                SlimmingMode::Ultra => 1,
            };

            let mut important_lines = lines[0..keep_top].to_vec();
            important_lines.push("... [stack trace truncated] ...");
            important_lines.extend(lines[lines.len() - keep_bottom..].to_vec());
            slimmed = important_lines.join("\n");
        }
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

        // Should preserve the critical error and summarize others
        assert!(!slimmed.is_empty());
        assert!(slimmed.iter().any(|entry| entry.message.contains("FATAL")));
        assert!(slimmed.iter().any(|entry| entry.level == Some("SUMMARY".to_string())));
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
    fn test_critical_error_detection() {
        assert!(is_critical_error("FATAL: System crash detected"));
        assert!(is_critical_error("Out of memory error"));
        assert!(is_critical_error("Security breach detected"));
        assert!(!is_critical_error("User login successful"));
        assert!(!is_critical_error("Processing completed"));
    }
}
