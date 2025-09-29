use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;

pub mod streaming;

thread_local! {
    static REGEX_CACHE: OnceCell<HashMap<String, Regex>> = OnceCell::new();
}

// Import the `console.log` function from the `console` module
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro for easier console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Use `wee_alloc` as the global allocator for smaller WASM binary size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log!("LogLens WASM module initialized with optimizations");
}

#[derive(Serialize, Deserialize)]
pub struct LogParseResult {
    pub total_lines: usize,
    pub error_lines: usize,
    pub warning_lines: usize,
    pub info_lines: usize,
    pub debug_lines: usize,
    pub lines_by_level: Vec<LogLinePreview>,
}

#[derive(Serialize, Deserialize)]
pub struct LogLinePreview {
    pub line_number: usize,
    pub level: Option<String>,
    pub timestamp: Option<String>,
    pub message: String,
    pub is_truncated: bool,
}

/// Parse log content on the client side for quick preview
#[wasm_bindgen]
pub fn parse_log_preview(content: &str, max_lines: usize) -> Result<JsValue, JsValue> {
    console_log!(
        "Starting log preview parsing for {} characters",
        content.len()
    );

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut error_lines = 0;
    let mut warning_lines = 0;
    let mut info_lines = 0;
    let mut debug_lines = 0;
    let mut lines_by_level = Vec::new();

    let lines_to_process = std::cmp::min(max_lines, total_lines);

    for (i, line) in lines.iter().take(lines_to_process).enumerate() {
        let parsed = parse_log_line(line);

        // Count by level
        if let Some(ref level) = parsed.level {
            match level.to_uppercase().as_str() {
                "ERROR" | "ERR" => error_lines += 1,
                "WARN" | "WARNING" => warning_lines += 1,
                "INFO" => info_lines += 1,
                "DEBUG" | "DBG" => debug_lines += 1,
                _ => {}
            }
        }

        lines_by_level.push(LogLinePreview {
            line_number: i + 1,
            level: parsed.level,
            timestamp: parsed.timestamp,
            message: if line.len() > 200 {
                format!("{}...", &line[..200])
            } else {
                line.to_string()
            },
            is_truncated: line.len() > 200,
        });
    }

    let result = LogParseResult {
        total_lines,
        error_lines,
        warning_lines,
        info_lines,
        debug_lines,
        lines_by_level,
    };

    console_log!(
        "Parsing complete: {} total lines, {} errors, {} warnings",
        total_lines,
        error_lines,
        warning_lines
    );

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Filter logs by level on the client side
#[wasm_bindgen]
pub fn filter_logs_by_level(content: &str, min_level: &str) -> Result<String, JsValue> {
    console_log!("Filtering logs by level: {}", min_level);

    let level_priority = match min_level.to_uppercase().as_str() {
        "DEBUG" | "DBG" => 0,
        "INFO" => 1,
        "WARN" | "WARNING" => 2,
        "ERROR" | "ERR" => 3,
        _ => return Err(JsValue::from_str("Invalid log level")),
    };

    let filtered_lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            let parsed = parse_log_line(line);
            if let Some(level) = parsed.level {
                let line_priority = match level.to_uppercase().as_str() {
                    "DEBUG" | "DBG" => 0,
                    "INFO" => 1,
                    "WARN" | "WARNING" => 2,
                    "ERROR" | "ERR" => 3,
                    _ => 0,
                };
                line_priority >= level_priority
            } else {
                // Include lines without level detection
                level_priority <= 1
            }
        })
        .collect();

    console_log!(
        "Filtered {} lines to {} lines",
        content.lines().count(),
        filtered_lines.len()
    );

    Ok(filtered_lines.join("\n"))
}

/// Count lines by log level
#[wasm_bindgen]
pub fn count_log_levels(content: &str) -> Result<JsValue, JsValue> {
    let mut counts = std::collections::HashMap::new();
    counts.insert("error".to_string(), 0u32);
    counts.insert("warning".to_string(), 0u32);
    counts.insert("info".to_string(), 0u32);
    counts.insert("debug".to_string(), 0u32);
    counts.insert("unknown".to_string(), 0u32);

    for line in content.lines() {
        let parsed = parse_log_line(line);
        let key = if let Some(level) = parsed.level {
            match level.to_uppercase().as_str() {
                "ERROR" | "ERR" => "error",
                "WARN" | "WARNING" => "warning",
                "INFO" => "info",
                "DEBUG" | "DBG" => "debug",
                _ => "unknown",
            }
        } else {
            "unknown"
        };

        *counts.get_mut(key).unwrap() += 1;
    }

    serde_wasm_bindgen::to_value(&counts).map_err(|e| JsValue::from_str(&e.to_string()))
}

// Simplified log parsing for client-side use
struct ParsedLogLine {
    timestamp: Option<String>,
    level: Option<String>,
    message: String,
}

// Regex cache structure for search patterns
struct RegexCache {
    cache: HashMap<String, Regex>,
    access_order: VecDeque<String>,
    max_size: usize,
}

impl RegexCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }

    pub fn get_or_compile(&mut self, pattern: &str) -> Regex {
        if let Some(regex) = self.cache.get(pattern) {
            // Update access order
            self.access_order.retain(|p| p != pattern);
            self.access_order.push_back(pattern.to_string());
            regex.clone()
        } else {
            // Evict if cache is full
            if self.cache.len() >= self.max_size {
                if let Some(oldest) = self.access_order.pop_front() {
                    self.cache.remove(&oldest);
                }
            }
            
            let regex = Regex::new(pattern).unwrap();
            self.cache.insert(pattern.to_string(), regex.clone());
            self.access_order.push_back(pattern.to_string());
            regex
        }
    }
}

// Global regex patterns for better performance (compiled once)
thread_local! {
    static TIMESTAMP_REGEX: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?").unwrap();
    static ERROR_REGEX: Regex = Regex::new(r"(?i)\b(error|err|fatal|critical)\b").unwrap();
    static WARN_REGEX: Regex = Regex::new(r"(?i)\b(warn|warning)\b").unwrap();
    static INFO_REGEX: Regex = Regex::new(r"(?i)\b(info|information)\b").unwrap();
    static DEBUG_REGEX: Regex = Regex::new(r"(?i)\b(debug|dbg|trace)\b").unwrap();
    static SEARCH_CACHE: std::cell::RefCell<RegexCache> = std::cell::RefCell::new(RegexCache::new(50));
}

fn parse_log_line(line: &str) -> ParsedLogLine {
    // Extract timestamp using compiled regex
    let timestamp = TIMESTAMP_REGEX.with(|re| re.find(line).map(|m| m.as_str().to_string()));

    // Extract level using compiled regexes for better performance
    let level = if ERROR_REGEX.with(|re| re.is_match(line)) {
        Some("ERROR".to_string())
    } else if WARN_REGEX.with(|re| re.is_match(line)) {
        Some("WARN".to_string())
    } else if INFO_REGEX.with(|re| re.is_match(line)) {
        Some("INFO".to_string())
    } else if DEBUG_REGEX.with(|re| re.is_match(line)) {
        Some("DEBUG".to_string())
    } else {
        None
    };

    ParsedLogLine {
        timestamp,
        level,
        message: line.to_string(),
    }
}

/// Enhanced streaming parser for large log files
#[wasm_bindgen]
pub fn parse_log_streaming(
    content: &str,
    chunk_size: usize,
    max_lines: usize,
) -> Result<JsValue, JsValue> {
    console_log!(
        "Starting streaming log parsing: {} chars, chunk size: {}, max lines: {}",
        content.len(),
        chunk_size,
        max_lines
    );

    let start_time = js_sys::Date::now();
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let total_lines = lines.len();

    // Process in chunks for better performance
    let mut error_lines = 0;
    let mut warning_lines = 0;
    let mut info_lines = 0;
    let mut debug_lines = 0;
    let mut lines_by_level = Vec::with_capacity(std::cmp::min(1000, total_lines));

    // Pre-allocate HashMap for level counts
    let mut level_counts: HashMap<String, usize> = HashMap::with_capacity(5);
    level_counts.insert("ERROR".to_string(), 0);
    level_counts.insert("WARN".to_string(), 0);
    level_counts.insert("INFO".to_string(), 0);
    level_counts.insert("DEBUG".to_string(), 0);
    level_counts.insert("UNKNOWN".to_string(), 0);

    for (chunk_start, chunk) in lines.chunks(chunk_size).enumerate() {
        for (i, line) in chunk.iter().enumerate() {
            let line_number = chunk_start * chunk_size + i + 1;
            let parsed = parse_log_line(line);

            // Count by level with pre-allocated HashMap
            match parsed.level.as_deref() {
                Some("ERROR") => error_lines += 1,
                Some("WARN") => warning_lines += 1,
                Some("INFO") => info_lines += 1,
                Some("DEBUG") => debug_lines += 1,
                _ => {} // Unknown levels
            }

            // Only store preview lines up to a reasonable limit
            if lines_by_level.len() < 1000 {
                lines_by_level.push(LogLinePreview {
                    line_number,
                    level: parsed.level.clone(),
                    timestamp: parsed.timestamp,
                    message: if line.len() > 500 {
                        format!("{}...", &line[..500])
                    } else {
                        line.to_string()
                    },
                    is_truncated: line.len() > 500,
                });
            }

            // Yield control periodically for better UI responsiveness
            if line_number % 1000 == 0 {
                // This would ideally use web_sys::window().unwrap().request_idle_callback()
                // but for now we just continue processing
            }
        }
    }

    let result = LogParseResult {
        total_lines,
        error_lines,
        warning_lines,
        info_lines,
        debug_lines,
        lines_by_level,
    };

    let end_time = js_sys::Date::now();
    console_log!(
        "Streaming parsing complete: {} lines in {:.2}ms",
        total_lines,
        end_time - start_time
    );

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Search logs with optimized regex compilation
#[wasm_bindgen]
pub fn search_logs(content: &str, query: &str, case_sensitive: bool) -> Result<JsValue, JsValue> {
    if query.trim().is_empty() {
        return Ok(serde_wasm_bindgen::to_value(&Vec::<String>::new()).unwrap());
    }

    console_log!(
        "Searching logs for query: '{}', case_sensitive: {}",
        query,
        case_sensitive
    );
    let start_time = js_sys::Date::now();

    // Compile regex once for the entire search
    let search_pattern = if case_sensitive {
        regex::escape(query)
    } else {
        format!("(?i){}", regex::escape(query))
    };

    let regex = match Regex::new(&search_pattern) {
        Ok(re) => re,
        Err(e) => return Err(JsValue::from_str(&format!("Invalid search pattern: {}", e))),
    };

    let mut matches = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if regex.is_match(line) {
            matches.push(format!("{}:{}", line_num + 1, line));
        }

        // Limit results to prevent UI overload
        if matches.len() >= 1000 {
            console_log!("Search limited to first 1000 matches");
            break;
        }
    }

    let end_time = js_sys::Date::now();
    console_log!(
        "Search complete: {} matches in {:.2}ms",
        matches.len(),
        end_time - start_time
    );

    serde_wasm_bindgen::to_value(&matches).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get performance statistics for the WASM module
#[wasm_bindgen]
pub fn get_performance_stats() -> Result<JsValue, JsValue> {
    // Simple memory stats that don't require web_sys features
    let stats = serde_json::json!({
        "memory_mb": 0.0,
        "wasm_version": env!("CARGO_PKG_VERSION"),
        "optimizations": [
            "Pre-compiled regex patterns",
            "Chunked processing for large files",
            "Limited preview line storage",
            "Optimized level detection",
            "Memory-efficient data structures"
        ]
    });

    serde_wasm_bindgen::to_value(&stats).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Memory-efficient log line counting
#[wasm_bindgen]
pub fn count_log_levels_optimized(content: &str) -> Result<JsValue, JsValue> {
    console_log!("Starting optimized log level counting");
    let start_time = js_sys::Date::now();

    // Use array instead of HashMap for better performance on small sets
    let mut error_count = 0u32;
    let mut warning_count = 0u32;
    let mut info_count = 0u32;
    let mut debug_count = 0u32;
    let mut unknown_count = 0u32;

    for line in content.lines() {
        // Use compiled thread-local regexes for better performance
        if ERROR_REGEX.with(|re| re.is_match(line)) {
            error_count += 1;
        } else if WARN_REGEX.with(|re| re.is_match(line)) {
            warning_count += 1;
        } else if INFO_REGEX.with(|re| re.is_match(line)) {
            info_count += 1;
        } else if DEBUG_REGEX.with(|re| re.is_match(line)) {
            debug_count += 1;
        } else {
            unknown_count += 1;
        }
    }

    let mut counts = HashMap::with_capacity(5);
    counts.insert("error".to_string(), error_count);
    counts.insert("warning".to_string(), warning_count);
    counts.insert("info".to_string(), info_count);
    counts.insert("debug".to_string(), debug_count);
    counts.insert("unknown".to_string(), unknown_count);

    let end_time = js_sys::Date::now();
    console_log!(
        "Optimized counting complete in {:.2}ms",
        end_time - start_time
    );

    serde_wasm_bindgen::to_value(&counts).map_err(|e| JsValue::from_str(&e.to_string()))
}
