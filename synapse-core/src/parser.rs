use crate::input::LogEntry;
use regex::Regex;
use std::sync::LazyLock;

// Compile regexes once at startup for performance
static TIMESTAMP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)")
        .expect("Failed to compile timestamp regex")
});

static ANSI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*m")
        .expect("Failed to compile ANSI regex")
});

static BRACKET_PREFIX_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*[\[\(][^\]\)]*[\]\)]\s*")
        .expect("Failed to compile bracket prefix regex")
});

pub fn parse_log_lines(lines: &[String]) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for (line_number, line) in lines.iter().enumerate() {
        let mut entry = parse_single_log_line(line);
        entry.line_number = Some(line_number + 1);
        entries.push(entry);
    }

    entries
}

pub fn parse_single_log_line(line: &str) -> LogEntry {
    let timestamp = TIMESTAMP_REGEX.find(line).map(|m| m.as_str().to_string());

    // Use the flexible log level parsing
    let level = normalize_log_level(line);

    // Extract message by removing timestamp and level if present
    let mut message = line.to_string();
    if let Some(ts) = &timestamp {
        message = message.replace(ts, "").trim().to_string();
    }
    
    // Remove level in various formats - need to check for ALL variants, not just normalized
    if level.is_some() {
        // Build comprehensive list of level patterns to remove
        let all_level_variants = vec![
            "ERROR", "ERR", "FATAL", "CRIT", "CRITICAL",
            "WARN", "WARNING", "INFO", "INFORMATION",
            "DEBUG", "DBG", "TRACE", "TRC"
        ];

        // Try each variant in brackets, parens, with colon first (more specific)
        let mut found = false;
        for variant in &all_level_variants {
            let patterns = [
                format!("[{}]", variant),
                format!("({})", variant),
                format!("{}:", variant),
            ];

            for pattern in &patterns {
                if message.to_uppercase().contains(&pattern.to_uppercase()) {
                    // Use case-insensitive replacement
                    if let Ok(pattern_re) = regex::Regex::new(&format!(r"(?i){}", regex::escape(pattern))) {
                        message = pattern_re.replace(&message, "").to_string().trim().to_string();
                        found = true;
                        break;
                    }
                }
            }
            if found {
                break;
            }
        }

        // If no formatted level found, try standalone word boundaries
        if !found {
            for variant in &all_level_variants {
                if let Ok(pattern_re) = regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(variant))) {
                    if pattern_re.is_match(&message) {
                        message = pattern_re.replace(&message, "").to_string().trim().to_string();
                        break;
                    }
                }
            }
        }
    }

    // Trim before checking for bracketed prefixes
    message = message.trim().to_string();

    // Remove common log prefixes like [main], (thread), etc. but be more selective
    message = BRACKET_PREFIX_REGEX.replace(&message, "").to_string().trim().to_string();

    LogEntry {
        timestamp,
        level,
        message,
        line_number: None, // Will be set by the caller
    }
}

/// Strip ANSI escape codes from a string
/// Examples: "\x1b[31mERROR\x1b[0;39m" -> "ERROR"
fn strip_ansi_codes(text: &str) -> String {
    // ANSI escape codes follow the pattern: ESC[...m where ESC is \x1b
    // We need to remove: \x1b[<any chars>m
    ANSI_REGEX.replace_all(text, "").to_string()
}

pub fn normalize_log_level(line: &str) -> Option<String> {
    // Strip ANSI escape codes first (e.g., [31mERROR[0;39m -> ERROR)
    let line_no_ansi = strip_ansi_codes(line);

    // Convert to uppercase for case-insensitive matching
    let line_upper = line_no_ansi.to_uppercase();

    // Check for numeric log levels first (most specific - e.g., "Level: 1 Debug info" should match level 1, not "info")
    let numeric_patterns = [
        (r"\b[Ll]evel\s*[:=]\s*0\b", "TRACE"),
        (r"\b[Ll]evel\s*[:=]\s*1\b", "DEBUG"),
        (r"\b[Ll]evel\s*[:=]\s*2\b", "INFO"),
        (r"\b[Ll]evel\s*[:=]\s*3\b", "WARN"),
        (r"\b[Ll]evel\s*[:=]\s*4\b", "ERROR"),
        (r"\b[Ll]evel\s*[:=]\s*5\b", "FATAL"),
    ];

    for (pattern, level) in &numeric_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return Some(level.to_string());
            }
        }
    }

    // Check for severity indicators (also more specific)
    let severity_patterns = [
        (r"\b[Ss]everity\s*[:=]\s*[Hh]igh\b", "ERROR"),
        (r"\b[Ss]everity\s*[:=]\s*[Mm]edium\b", "WARN"),
        (r"\b[Ss]everity\s*[:=]\s*[Ll]ow\b", "INFO"),
    ];

    for (pattern, level) in &severity_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return Some(level.to_string());
            }
        }
    }

    // Check for colon-separated levels ONLY at the start of the line like "DEBUG: message"
    // This prevents "Debug information: trace" from matching
    if let Ok(re) = Regex::new(r"^\s*(ERROR|ERR|FATAL|CRIT|CRITICAL|WARN|WARNING|INFO|INFORMATION|DEBUG|DBG|TRACE|TRC)\s*:\s") {
        if let Some(caps) = re.captures(&line_upper) {
            let level = &caps[1];
            match level {
                "ERROR" | "ERR" => return Some("ERROR".to_string()),
                "FATAL" | "CRIT" | "CRITICAL" => return Some("FATAL".to_string()),
                "WARN" | "WARNING" => return Some("WARN".to_string()),
                "INFO" | "INFORMATION" => return Some("INFO".to_string()),
                "DEBUG" | "DBG" => return Some("DEBUG".to_string()),
                "TRACE" | "TRC" => return Some("TRACE".to_string()),
                _ => {}
            }
        }
    }

    // Define patterns for different log levels - ONLY match formatted levels
    // Order matters for specificity: more specific patterns first
    // FATAL/CRITICAL must come before ERROR to match "FATAL error occurred" correctly
    let level_patterns = [
        // FATAL/CRITICAL patterns - formatted only
        (vec![r"\[FATAL\]", r"\(FATAL\)", r"\[CRIT\]", r"\(CRIT\)", r"\[CRITICAL\]", r"\(CRITICAL\)"], "FATAL"),

        // ERROR patterns - formatted only
        (vec![r"\[ERROR\]", r"\(ERROR\)", r"\[ERR\]", r"\(ERR\)"], "ERROR"),

        // WARN/WARNING patterns - formatted only
        (vec![r"\[WARN\]", r"\(WARN\)", r"\[WARNING\]", r"\(WARNING\)"], "WARN"),

        // INFO/INFORMATION patterns - formatted only
        (vec![r"\[INFO\]", r"\(INFO\)", r"\[INFORMATION\]", r"\(INFORMATION\)"], "INFO"),

        // DEBUG patterns - formatted only
        (vec![r"\[DEBUG\]", r"\(DEBUG\)", r"\[DBG\]", r"\(DBG\)"], "DEBUG"),

        // TRACE patterns - formatted only
        (vec![r"\[TRACE\]", r"\(TRACE\)", r"\[TRC\]", r"\(TRC\)"], "TRACE"),
    ];

    // Try each pattern group
    for (patterns, normalized_level) in &level_patterns {
        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(&line_upper) {
                    return Some(normalized_level.to_string());
                }
            }
        }
    }

    // Check for standalone ALL-CAPS log levels (not in brackets/parens, not with colons)
    // These must be ENTIRE WORDS in ALL CAPS followed by space or bracket
    // Examples that match: "ERROR something", "2024-01-01 ERROR [main]", "INFO test"
    // Examples that DON'T match: "Debug information", "error occurred", "The error message"
    //
    // Strategy: Find the level keyword in the line, then check if it's actually all-caps in original
    let standalone_patterns = [
        (r"\b(FATAL|CRIT|CRITICAL)\s+", "FATAL"),
        (r"\b(ERROR|ERR)\s+", "ERROR"),
        (r"\b(WARN|WARNING)\s+", "WARN"),
        (r"\bINFO\s+", "INFO"),
        (r"\b(DEBUG|DBG)\s+", "DEBUG"),
        (r"\b(TRACE|TRC)\s+", "TRACE"),
    ];

    for (pattern, normalized_level) in &standalone_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(mat) = re.find(&line_upper) {
                // Found a match in uppercase version - now verify it's actually uppercase in original (ANSI-stripped)
                let match_start = mat.start();
                let match_end = mat.end();

                // Extract the same region from the ANSI-stripped line
                if match_start < line_no_ansi.len() {
                    let original_match = &line_no_ansi[match_start..match_end.min(line_no_ansi.len())];
                    let level_word = original_match.trim().split_whitespace().next().unwrap_or("");

                    // Only accept if the level word is ALL CAPS in the original
                    if level_word == level_word.to_uppercase() && !level_word.is_empty() {
                        return Some(normalized_level.to_string());
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line_with_timestamp_and_level() {
        let line = "2024-01-20T10:30:45.123Z ERROR [main] Database connection failed";
        let entries = parse_log_lines(&[line.to_string()]);

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].timestamp,
            Some("2024-01-20T10:30:45.123Z".to_string())
        );
        assert_eq!(entries[0].level, Some("ERROR".to_string()));
        assert_eq!(entries[0].message, "Database connection failed");
    }

    #[test]
    fn test_parse_log_line_unstructured() {
        let line = "Something went wrong here";
        let entries = parse_log_lines(&[line.to_string()]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].timestamp, None);
        assert_eq!(entries[0].level, None);
        assert_eq!(entries[0].message, "Something went wrong here");
    }

    #[test]
    fn test_log_level_variations() {
        let test_cases = vec![
            ("[ERROR] Something failed", Some("ERROR".to_string())),
            ("(WARNING) Check this", Some("WARN".to_string())),
            ("[INFO] Process started", Some("INFO".to_string())),
            ("DEBUG: Debugging info", Some("DEBUG".to_string())),
            ("[TRACE] Detailed trace", Some("TRACE".to_string())),
            ("FATAL error occurred", Some("FATAL".to_string())),
            ("[WARN] Warning message", Some("WARN".to_string())),
            ("(ERR) Short error", Some("ERROR".to_string())),
            ("DBG: Debug message", Some("DEBUG".to_string())),
            ("INFORMATION: Detail", Some("INFO".to_string())),
            ("CRITICAL failure", Some("FATAL".to_string())),
            ("Level: 3 - Something", Some("WARN".to_string())),
            ("Level: 4 - Error occurred", Some("ERROR".to_string())),
            ("Severity: High issue", Some("ERROR".to_string())),
            ("Severity: Medium concern", Some("WARN".to_string())),
            ("No log level here", None),
        ];

        for (input, expected) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test] 
    fn test_bracketed_log_levels() {
        let test_cases = vec![
            ("2024-01-20 [ERROR] Connection failed", "ERROR"),
            ("2024-01-20 [WARNING] Resource low", "WARN"), 
            ("2024-01-20 [INFO] Service started", "INFO"),
            ("2024-01-20 [DEBUG] Variable x=5", "DEBUG"),
            ("2024-01-20 [TRACE] Method entry", "TRACE"),
        ];

        for (input, expected_level) in test_cases {
            let entries = parse_log_lines(&[input.to_string()]);
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].level.as_deref(), Some(expected_level), 
                      "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_case_insensitive_log_levels() {
        let test_cases = vec![
            ("error: Something failed", "ERROR"),
            ("Warning: Check this", "WARN"),
            ("info: Process started", "INFO"),
            ("Debug: Variable state", "DEBUG"),
            ("trace: Method call", "TRACE"),
        ];

        for (input, expected_level) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result.as_deref(), Some(expected_level), 
                      "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_numeric_log_levels() {
        let test_cases = vec![
            ("Level: 0 Trace message", "TRACE"),
            ("Level: 1 Debug info", "DEBUG"),
            ("Level: 2 Information", "INFO"),
            ("Level: 3 Warning here", "WARN"),
            ("Level: 4 Error occurred", "ERROR"),
            ("Level: 5 Fatal issue", "FATAL"),
        ];

        for (input, expected_level) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result.as_deref(), Some(expected_level), 
                      "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_severity_based_levels() {
        let test_cases = vec![
            ("Severity: High - Critical issue", "ERROR"),
            ("Severity: Medium - Warning condition", "WARN"), 
            ("Severity: Low - Information only", "INFO"),
        ];

        for (input, expected_level) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result.as_deref(), Some(expected_level), 
                      "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_complex_log_parsing() {
        let complex_log = "2024-01-20T10:30:45.123Z [WARNING] [main] Database pool exhausted, retrying in 5s";
        let entries = parse_log_lines(&[complex_log.to_string()]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].timestamp, Some("2024-01-20T10:30:45.123Z".to_string()));
        assert_eq!(entries[0].level, Some("WARN".to_string()));
        assert_eq!(entries[0].message, "Database pool exhausted, retrying in 5s");
    }

    #[test]
    fn test_false_level_detection_in_message() {
        // BUG TEST: These lines contain level keywords in message content
        // but should NOT be detected as having those log levels
        let test_cases = vec![
            ("User provided invalid information about the account", None),
            ("The system encountered an error in processing", None),
            ("Debug information: trace level data", None),
            ("Please trace the issue back to its source", None),
            ("The error message was displayed to the user", None),
            ("This message contains information for debugging", None),
        ];

        for (input, expected) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result, expected,
                      "BUG: Line '{}' incorrectly detected as {:?}, should be {:?}",
                      input, result, expected);
        }
    }

    #[test]
    fn test_ansi_escape_code_stripping() {
        // Test ANSI escape code removal from Spring Boot logs
        let test_cases = vec![
            // Actual Spring Boot format with ANSI codes
            ("\x1b[2m2025-06-23 11:47:10.714\x1b[0;39m \x1b[31mERROR\x1b[0;39m \x1b[35m36\x1b[0;39m \x1b[2m---\x1b[0;39m \x1b[2m[nio-8080-exec-4]\x1b[0;39m", "ERROR"),
            ("\x1b[31mERROR\x1b[0;39m Application startup failed", "ERROR"),
            ("\x1b[33mWARN\x1b[0;39m Deprecated API usage", "WARN"),
            ("\x1b[32mINFO\x1b[0;39m Server started successfully", "INFO"),
            ("\x1b[36mDEBUG\x1b[0;39m Processing request", "DEBUG"),
            ("\x1b[35mTRACE\x1b[0;39m Method entry", "TRACE"),
            // Bracketed with ANSI codes
            ("\x1b[2m\x1b[31m[ERROR]\x1b[0;39m\x1b[0;39m Connection failed", "ERROR"),
        ];

        for (input, expected_level) in test_cases {
            let result = normalize_log_level(input);
            assert_eq!(result.as_deref(), Some(expected_level),
                      "Failed to detect level in ANSI-coded line: '{}'", input);
        }
    }

    #[test]
    fn test_strip_ansi_codes_function() {
        let test_cases = vec![
            ("\x1b[31mERROR\x1b[0;39m", "ERROR"),
            ("\x1b[2m2025-06-23\x1b[0;39m", "2025-06-23"),
            ("No ANSI codes here", "No ANSI codes here"),
            ("\x1b[31m\x1b[1mMultiple\x1b[0m codes", "Multiple codes"),
        ];

        for (input, expected) in test_cases {
            let result = strip_ansi_codes(input);
            assert_eq!(result, expected,
                      "Failed to strip ANSI codes from '{}'", input);
        }
    }
}
