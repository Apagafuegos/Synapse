use crate::input::LogEntry;
use regex::Regex;

pub fn parse_log_lines(lines: &[String]) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    // Common log patterns
    let timestamp_regex =
        Regex::new(r"(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)")
            .unwrap();

    for (line_number, line) in lines.iter().enumerate() {
        let mut entry = parse_single_log_line(line, &timestamp_regex);
        entry.line_number = Some(line_number + 1);
        entries.push(entry);
    }

    entries
}

pub fn parse_single_log_line(line: &str, timestamp_regex: &Regex) -> LogEntry {
    let timestamp = timestamp_regex.find(line).map(|m| m.as_str().to_string());

    // Use the flexible log level parsing
    let level = normalize_log_level(line);

    // Extract message by removing timestamp and level if present
    let mut message = line.to_string();
    if let Some(ts) = &timestamp {
        message = message.replace(ts, "").trim().to_string();
    }
    
    // Remove level in various formats more carefully
    if let Some(lvl) = &level {
        // Try to remove common level patterns, being careful about order
        let level_patterns = [
            format!("[{}]", lvl),
            format!("({})", lvl),
            format!("{}:", lvl),
            lvl.to_string(),
        ];
        
        for pattern in &level_patterns {
            if message.to_uppercase().contains(&pattern.to_uppercase()) {
                // Use case-insensitive replacement
                let pattern_re = regex::Regex::new(&format!(r"(?i){}", regex::escape(pattern))).unwrap();
                message = pattern_re.replace(&message, "").to_string().trim().to_string();
                break; // Only remove the first match to avoid over-processing
            }
        }
    }

    // Remove common log prefixes like [main], (thread), etc. but be more selective
    let bracket_re = Regex::new(r"^\s*[\[\(][^\]\)]*[\]\)]\s*").unwrap();
    message = bracket_re.replace(&message, "").to_string();

    message = message.trim().to_string();

    LogEntry {
        timestamp,
        level,
        message,
        line_number: None, // Will be set by the caller
    }
}

pub fn normalize_log_level(line: &str) -> Option<String> {
    // Convert to uppercase for case-insensitive matching
    let line_upper = line.to_uppercase();
    
    // Define comprehensive patterns for different log levels - order matters for specificity
    let level_patterns = [
        // ERROR patterns - most specific first
        (vec![r"\bERROR\b", r"\bERR\b", r"\[ERROR\]", r"\(ERROR\)", r"\[ERR\]", r"\(ERR\)"], "ERROR"),
        
        // FATAL/CRITICAL patterns
        (vec![r"\bFATAL\b", r"\bCRIT\b", r"\bCRITICAL\b", r"\[FATAL\]", r"\(FATAL\)", r"\[CRIT\]", r"\(CRIT\)", r"\[CRITICAL\]", r"\(CRITICAL\)"], "FATAL"),
        
        // WARN/WARNING patterns  
        (vec![r"\bWARN\b", r"\bWARNING\b", r"\[WARN\]", r"\(WARN\)", r"\[WARNING\]", r"\(WARNING\)"], "WARN"),
        
        // INFO/INFORMATION patterns
        (vec![r"\bINFO\b", r"\bINFORMATION\b", r"\[INFO\]", r"\(INFO\)", r"\[INFORMATION\]", r"\(INFORMATION\)"], "INFO"),
        
        // DEBUG patterns
        (vec![r"\bDEBUG\b", r"\bDBG\b", r"\[DEBUG\]", r"\(DEBUG\)", r"\[DBG\]", r"\(DBG\)"], "DEBUG"),
        
        // TRACE patterns
        (vec![r"\bTRACE\b", r"\bTRC\b", r"\[TRACE\]", r"\(TRACE\)", r"\[TRC\]", r"\(TRC\)"], "TRACE"),
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
    
    // Check for colon-separated levels like "DEBUG: message"
    if let Ok(re) = Regex::new(r"\b(ERROR|ERR|FATAL|CRIT|CRITICAL|WARN|WARNING|INFO|INFORMATION|DEBUG|DBG|TRACE|TRC)\s*:") {
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
    
    // Additional check for numeric log levels (common in some systems)
    let numeric_patterns = [
        (r"\b[Ll]evel\s*[:\s]\s*0\b", "TRACE"),
        (r"\b[Ll]evel\s*[:\s]\s*1\b", "DEBUG"), 
        (r"\b[Ll]evel\s*[:\s]\s*2\b", "INFO"),
        (r"\b[Ll]evel\s*[:\s]\s*3\b", "WARN"),
        (r"\b[Ll]evel\s*[:\s]\s*4\b", "ERROR"),
        (r"\b[Ll]evel\s*[:\s]\s*5\b", "FATAL"),
    ];
    
    for (pattern, level) in &numeric_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return Some(level.to_string());
            }
        }
    }
    
    // Check for severity indicators
    let severity_patterns = [
        (r"\b[Ss]everity\s*[:\s]\s*[Hh]igh\b", "ERROR"),
        (r"\b[Ss]everity\s*[:\s]\s*[Mm]edium\b", "WARN"),
        (r"\b[Ss]everity\s*[:\s]\s*[Ll]ow\b", "INFO"),
    ];
    
    for (pattern, level) in &severity_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return Some(level.to_string());
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
}
