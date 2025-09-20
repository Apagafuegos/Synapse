use crate::model::{LogEntry, LogLevel};
use chrono::{DateTime, Datelike, Utc};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ParseResult {
    Success(LogEntry),
    Skip,
    Error(String),
    /// For AI analysis: raw line with metadata for later processing
    RawWithMetadata {
        line: String,
        line_number: usize,
        timestamp_hint: Option<DateTime<Utc>>,
        level_hint: Option<LogLevel>,
    },
}

#[derive(Debug, Clone)]
pub struct ParseContext {
    pub line_number: usize,
    pub file_path: Option<String>,
    pub previous_entry: Option<LogEntry>,
    pub ai_analysis_enabled: bool,
}

impl Default for ParseContext {
    fn default() -> Self {
        Self {
            line_number: 0,
            file_path: None,
            previous_entry: None,
            ai_analysis_enabled: false,
        }
    }
}

pub trait LogParser {
    fn parse_line(&self, line: &str, context: &ParseContext) -> ParseResult;
    fn can_parse(&self, sample_lines: &[&str]) -> f32;
    fn name(&self) -> &'static str;
}

pub struct ParserRegistry {
    parsers: Vec<Box<dyn LogParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: Vec::new(),
        };
        registry.register_parser(Box::new(JsonParser::new()));
        registry.register_parser(Box::new(TextParser::new()));
        registry.register_parser(Box::new(CommonLogFormatParser::new()));
        registry
    }

    pub fn register_parser(&mut self, parser: Box<dyn LogParser>) {
        self.parsers.push(parser);
    }

    pub fn detect_parser(&self, sample_lines: &[&str]) -> Option<&dyn LogParser> {
        let mut best_parser = None;
        let mut best_score = 0.0;

        for parser in &self.parsers {
            let score = parser.can_parse(sample_lines);
            if score > best_score && score > 0.5 {
                best_score = score;
                best_parser = Some(parser.as_ref());
            }
        }

        best_parser
    }

    pub fn get_parser(&self, name: &str) -> Option<&dyn LogParser> {
        self.parsers
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }
}

pub struct JsonParser {
    strict_mode: bool,
}

impl JsonParser {
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
}

impl LogParser for JsonParser {
    fn parse_line(&self, line: &str, context: &ParseContext) -> ParseResult {
        let json_value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                if self.strict_mode {
                    return ParseResult::Error(format!("JSON parse error: {}", e));
                }
                return ParseResult::Skip;
            }
        };

        let timestamp = extract_timestamp(&json_value).unwrap_or_else(Utc::now);
        let level = extract_level(&json_value);
        let message = extract_message(&json_value);
        let fields = extract_fields(&json_value);

        let mut entry = LogEntry::new(timestamp, level, message, line.to_string());
        entry.fields = fields;

        // Add AI analysis metadata
        if context.ai_analysis_enabled {
            entry
                .fields
                .insert("_line_number".to_string(), context.line_number.to_string());
            if let Some(ref path) = context.file_path {
                entry.fields.insert("_file_path".to_string(), path.clone());
            }
        }

        ParseResult::Success(entry)
    }

    fn can_parse(&self, sample_lines: &[&str]) -> f32 {
        let mut json_count = 0;
        let total_lines = sample_lines.len();

        for line in sample_lines {
            if line.trim().is_empty() {
                continue;
            }
            if serde_json::from_str::<Value>(line).is_ok() {
                json_count += 1;
            }
        }

        if total_lines == 0 {
            0.0
        } else {
            json_count as f32 / total_lines as f32
        }
    }

    fn name(&self) -> &'static str {
        "json"
    }
}

pub struct TextParser {
    timestamp_regex: Regex,
    level_regex: Regex,
    apache_regex: Regex,
    syslog_regex: Regex,
    patterns: Vec<LogPattern>,
}

#[derive(Debug, Clone)]
pub struct LogPattern {
    name: String,
    regex: Regex,
    timestamp_group: usize,
    level_group: usize,
    message_group: usize,
}

impl TextParser {
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // Standard format: 2024-01-01T10:00:00Z LEVEL message
        patterns.push(LogPattern {
            name: "standard".to_string(),
            regex: Regex::new(r"^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)\s+(\w+)\s+(.+)$").unwrap(),
            timestamp_group: 1,
            level_group: 2,
            message_group: 3,
        });

        // Apache/Nginx style: [01/Jan/2024:10:00:00 +0000] LEVEL message
        patterns.push(LogPattern {
            name: "apache".to_string(),
            regex: Regex::new(
                r"^\[(\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2}\s+[+-]\d{4})\]\s+(\w+)\s+(.+)$",
            )
            .unwrap(),
            timestamp_group: 1,
            level_group: 2,
            message_group: 3,
        });

        // Syslog style: Jan 01 10:00:00 hostname service[pid]: LEVEL message
        patterns.push(LogPattern {
            name: "syslog".to_string(),
            regex: Regex::new(
                r"^(\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+\S+\s+\S+:\s+(\w+):\s+(.+)$",
            )
            .unwrap(),
            timestamp_group: 1,
            level_group: 2,
            message_group: 3,
        });

        Self {
            timestamp_regex: Regex::new(
                r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?",
            )
            .unwrap(),
            level_regex: Regex::new(r"(?i)\b(DEBUG|INFO|WARN|ERROR|TRACE)\b").unwrap(),
            apache_regex: Regex::new(
                r"^(\d{2})/(\w{3})/(\d{4}):(\d{2}):(\d{2}):(\d{2})\s+([+-]\d{4})$",
            )
            .unwrap(),
            syslog_regex: Regex::new(r"^(\w{3})\s+(\d{1,2})\s+(\d{2}):(\d{2}):(\d{2})$").unwrap(),
            patterns,
        }
    }

    pub fn add_pattern(&mut self, pattern: LogPattern) {
        self.patterns.push(pattern);
    }
}

impl LogParser for TextParser {
    fn parse_line(&self, line: &str, context: &ParseContext) -> ParseResult {
        let line = line.trim();
        if line.is_empty() {
            return ParseResult::Skip;
        }

        // Try each pattern first
        for pattern in &self.patterns {
            if let Some(caps) = pattern.regex.captures(line) {
                let timestamp = self
                    .parse_timestamp(&caps[pattern.timestamp_group])
                    .unwrap_or_else(Utc::now);
                let level = caps[pattern.level_group]
                    .parse::<LogLevel>()
                    .unwrap_or(LogLevel::Unknown);
                let message = caps[pattern.message_group].trim().to_string();

                let mut entry = LogEntry::new(timestamp, level.clone(), message, line.to_string());

                // Add AI analysis metadata
                if context.ai_analysis_enabled {
                    entry
                        .fields
                        .insert("_pattern".to_string(), pattern.name.clone());
                    entry
                        .fields
                        .insert("_line_number".to_string(), context.line_number.to_string());
                    if let Some(ref path) = context.file_path {
                        entry.fields.insert("_file_path".to_string(), path.clone());
                    }
                }

                return ParseResult::Success(entry);
            }
        }

        // Fallback to basic extraction
        let timestamp = self
            .extract_timestamp_from_text(line)
            .unwrap_or_else(Utc::now);
        let level = self.extract_level_from_text(line);
        let message = line.to_string();

        let entry = LogEntry::new(timestamp, level.clone(), message, line.to_string());

        // For AI analysis: store raw lines that don't match patterns
        if context.ai_analysis_enabled {
            return ParseResult::RawWithMetadata {
                line: line.to_string(),
                line_number: context.line_number,
                timestamp_hint: Some(timestamp),
                level_hint: Some(level),
            };
        }

        ParseResult::Success(entry)
    }

    fn can_parse(&self, sample_lines: &[&str]) -> f32 {
        let mut matched_lines = 0;
        let total_lines = sample_lines.len();

        for line in sample_lines {
            if line.trim().is_empty() {
                continue;
            }

            // Quick check: look for timestamp and level first (most common case)
            if self.timestamp_regex.is_match(line) && self.level_regex.is_match(line) {
                matched_lines += 1;
                continue;
            }

            // Only check patterns if quick check fails
            for pattern in &self.patterns {
                if pattern.regex.is_match(line) {
                    matched_lines += 1;
                    break;
                }
            }
        }

        if total_lines == 0 {
            0.0
        } else {
            matched_lines as f32 / total_lines as f32
        }
    }

    fn name(&self) -> &'static str {
        "text"
    }
}

impl TextParser {
    fn extract_timestamp_from_text(&self, line: &str) -> Option<DateTime<Utc>> {
        self.timestamp_regex
            .find(line)
            .and_then(|mat| self.parse_timestamp(mat.as_str()))
    }

    fn parse_timestamp(&self, timestamp_str: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| DateTime::parse_from_rfc2822(timestamp_str))
            .or_else(|_| {
                // Try Apache format: 01/Jan/2024:10:00:00 +0000
                if let Some(caps) = self.apache_regex.captures(timestamp_str) {
                    let day = &caps[1];
                    let month = &caps[2];
                    let year = &caps[3];
                    let hour = &caps[4];
                    let minute = &caps[5];
                    let second = &caps[6];
                    let offset = &caps[7];

                    let datetime_str = format!(
                        "{} {} {} {}:{}:{} {}",
                        day, month, year, hour, minute, second, offset
                    );
                    DateTime::parse_from_str(&datetime_str, "%d %b %Y %H:%M:%S %z").map_err(|_| ())
                } else {
                    Err(())
                }
            })
            .or_else(|_| {
                // Try syslog format: Jan 01 10:00:00
                if let Some(caps) = self.syslog_regex.captures(timestamp_str) {
                    let month = &caps[1];
                    let day = &caps[2];
                    let hour = &caps[3];
                    let minute = &caps[4];
                    let second = &caps[5];

                    let current_year = Utc::now().year();
                    let datetime_str = format!(
                        "{} {} {} {}:{}:{}",
                        day, month, current_year, hour, minute, second
                    );
                    DateTime::parse_from_str(&datetime_str, "%d %b %Y %H:%M:%S").map_err(|_| ())
                } else {
                    Err(())
                }
            })
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    }

    fn extract_level_from_text(&self, line: &str) -> LogLevel {
        self.level_regex
            .find(line)
            .and_then(|mat| mat.as_str().parse::<LogLevel>().ok())
            .unwrap_or(LogLevel::Unknown)
    }
}

pub struct CommonLogFormatParser {
    regex: Regex,
}

impl CommonLogFormatParser {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(
                r#"^(\S+) \S+ \S+ \[([\w:/]+\s[+\-]\d{4})\] "(\S+) (\S+) (\S+)" (\d{3}) (\d+|-)"#,
            )
            .unwrap(),
        }
    }
}

impl LogParser for CommonLogFormatParser {
    fn parse_line(&self, line: &str, context: &ParseContext) -> ParseResult {
        if let Some(caps) = self.regex.captures(line) {
            let timestamp = self.parse_timestamp(&caps[2]).unwrap_or_else(Utc::now);
            let status_code = &caps[6];
            let level = match status_code {
                "200" | "201" | "202" | "204" => LogLevel::Info,
                "300" | "301" | "302" | "304" => LogLevel::Debug,
                "400" | "401" | "403" | "404" => LogLevel::Warn,
                "500" | "501" | "502" | "503" => LogLevel::Error,
                _ => LogLevel::Unknown,
            };

            let message = format!(
                "{} {} {} - Status {}",
                &caps[3], &caps[4], &caps[5], status_code
            );

            let mut entry = LogEntry::new(timestamp, level.clone(), message, line.to_string());
            entry.fields.insert("ip".to_string(), caps[1].to_string());
            entry
                .fields
                .insert("method".to_string(), caps[3].to_string());
            entry.fields.insert("path".to_string(), caps[4].to_string());
            entry
                .fields
                .insert("protocol".to_string(), caps[5].to_string());
            entry
                .fields
                .insert("status_code".to_string(), caps[6].to_string());
            entry.fields.insert("size".to_string(), caps[7].to_string());

            // Add AI analysis metadata
            if context.ai_analysis_enabled {
                entry
                    .fields
                    .insert("_format".to_string(), "common_log".to_string());
                entry
                    .fields
                    .insert("_line_number".to_string(), context.line_number.to_string());
                if let Some(ref path) = context.file_path {
                    entry.fields.insert("_file_path".to_string(), path.clone());
                }
            }

            ParseResult::Success(entry)
        } else {
            ParseResult::Skip
        }
    }

    fn can_parse(&self, sample_lines: &[&str]) -> f32 {
        let mut matched_lines = 0;
        let total_lines = sample_lines.len();

        for line in sample_lines {
            if line.trim().is_empty() {
                continue;
            }
            if self.regex.is_match(line) {
                matched_lines += 1;
            }
        }

        if total_lines == 0 {
            0.0
        } else {
            matched_lines as f32 / total_lines as f32
        }
    }

    fn name(&self) -> &'static str {
        "common_log"
    }
}

impl CommonLogFormatParser {
    fn parse_timestamp(&self, timestamp_str: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_str(timestamp_str, "%d/%b/%Y:%H:%M:%S %z")
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    }
}

fn extract_timestamp(json: &Value) -> Option<DateTime<Utc>> {
    json.get("timestamp")
        .or_else(|| json.get("time"))
        .or_else(|| json.get("@timestamp"))
        .or_else(|| json.get("datetime"))
        .and_then(|v| v.as_str())
        .and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .or_else(|_| DateTime::parse_from_rfc2822(s))
                .or_else(|_| DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f"))
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        })
}

fn extract_level(json: &Value) -> LogLevel {
    json.get("level")
        .or_else(|| json.get("severity"))
        .or_else(|| json.get("priority"))
        .or_else(|| json.get("loglevel"))
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                s.parse::<LogLevel>().ok()
            } else if let Some(n) = v.as_u64() {
                // Numeric log levels (0=trace, 1=debug, 2=info, 3=warn, 4=error)
                match n {
                    0 => Some(LogLevel::Trace),
                    1 => Some(LogLevel::Debug),
                    2 => Some(LogLevel::Info),
                    3 => Some(LogLevel::Warn),
                    4 => Some(LogLevel::Error),
                    _ => Some(LogLevel::Unknown),
                }
            } else {
                None
            }
        })
        .unwrap_or(LogLevel::Unknown)
}

fn extract_message(json: &Value) -> String {
    json.get("message")
        .or_else(|| json.get("msg"))
        .or_else(|| json.get("text"))
        .or_else(|| json.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // If no message field, try to construct one from other fields
            let mut parts = Vec::new();
            if let Some(method) = json.get("method").and_then(|v| v.as_str()) {
                parts.push(method);
            }
            if let Some(path) = json.get("path").and_then(|v| v.as_str()) {
                parts.push(path);
            }
            if let Some(status) = json.get("status").and_then(|v| v.as_str()) {
                parts.push(status);
            }

            if parts.is_empty() {
                "No message".to_string()
            } else {
                parts.join(" ")
            }
        })
}

fn extract_fields(json: &Value) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let reserved_fields = [
        "timestamp",
        "time",
        "@timestamp",
        "datetime",
        "level",
        "severity",
        "priority",
        "loglevel",
        "message",
        "msg",
        "text",
        "description",
    ];

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if !reserved_fields.contains(&key.as_str()) {
                if let Some(s) = value.as_str() {
                    fields.insert(key.clone(), s.to_string());
                } else if let Some(n) = value.as_u64() {
                    fields.insert(key.clone(), n.to_string());
                } else if let Some(b) = value.as_bool() {
                    fields.insert(key.clone(), b.to_string());
                } else if let Some(arr) = value.as_array() {
                    fields.insert(key.clone(), format!("{:?}", arr));
                } else {
                    fields.insert(key.clone(), value.to_string());
                }
            }
        }
    }

    fields
}
