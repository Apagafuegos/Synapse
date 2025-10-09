use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    CodeRelated {
        file: Option<String>,
        function: Option<String>,
        line: Option<u32>,
        exception_type: Option<String>,
    },
    InfrastructureRelated {
        component: String, // "database", "network", "filesystem", "memory"
        severity: Severity,
        service: Option<String>,
    },
    ConfigurationRelated {
        config_file: Option<String>,
        missing_setting: Option<String>,
        invalid_value: Option<String>,
    },
    ExternalServiceRelated {
        service: String,
        endpoint: Option<String>,
        status_code: Option<u32>,
    },
    UnknownRelated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorClassification {
    pub category: ErrorCategory,
    pub confidence: f32, // 0.0 to 1.0
    pub reason: String,
    pub patterns_matched: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ErrorClassifier {
    code_patterns: Vec<(Regex, &'static str)>,
    infrastructure_patterns: Vec<(Regex, &'static str)>,
    config_patterns: Vec<(Regex, &'static str)>,
    external_service_patterns: Vec<(Regex, &'static str)>,
    database_patterns: Vec<(Regex, &'static str)>,
}

impl ErrorClassifier {
    pub fn new() -> Self {
        Self {
            code_patterns: vec![
                (Regex::new(r"(?i)(exception|error)\s+at\s+(.+?):(\d+)").unwrap(), "stack_trace"),
                (Regex::new(r"(?i)null\s*pointer|segmentation\s*fault|access\s*violation").unwrap(), "memory_error"),
                (Regex::new(r"(?i)(class|method|function)\s+(.+?)\s+not\s+(found|defined)").unwrap(), "missing_symbol"),
                (Regex::new(r"(?i)syntax\s*error|parse\s*error|compilation\s*error").unwrap(), "syntax_error"),
                (Regex::new(r"(?i)assertion\s+(failed|error)|test\s+failed").unwrap(), "test_failure"),
                (Regex::new(r"(?i)type\s*error|type\s*mismatch").unwrap(), "type_error"),
                (Regex::new(r"(?i)index\s+out\s+of\s+bounds|array\s+index").unwrap(), "bounds_error"),
                (Regex::new(r"(?i)import\s*error|module\s+not\s+found").unwrap(), "import_error"),
            ],
            infrastructure_patterns: vec![
                (Regex::new(r"(?i)connection\s+(refused|timeout|reset|failed)").unwrap(), "network"),
                (Regex::new(r"(?i)disk\s+(full|space|quota|io\s+error)").unwrap(), "filesystem"),
                (Regex::new(r"(?i)out\s+of\s+memory|memory\s+exhausted|oom").unwrap(), "memory"),
                (Regex::new(r"(?i)port\s+(\d+)\s+(already\s+in\s+use|unavailable)").unwrap(), "network"),
                (Regex::new(r"(?i)permission\s+denied|access\s+denied|unauthorized").unwrap(), "permissions"),
                (Regex::new(r"(?i)ssl\s+(error|handshake|certificate)").unwrap(), "ssl"),
                (Regex::new(r"(?i)dns\s+(resolution|lookup|error)|host\s+not\s+found").unwrap(), "dns"),
                (Regex::new(r"(?i)timeout|timed\s+out").unwrap(), "timeout"),
            ],
            config_patterns: vec![
                (Regex::new(r"(?i)configuration\s+(error|invalid|missing)").unwrap(), "config_error"),
                (Regex::new(r"(?i)property\s+(.+?)\s+not\s+(found|set|defined)").unwrap(), "missing_property"),
                (Regex::new(r"(?i)invalid\s+(value|setting|parameter)").unwrap(), "invalid_config"),
                (Regex::new(r"(?i)environment\s+variable\s+(.+?)\s+not\s+set").unwrap(), "missing_env_var"),
                (Regex::new(r"(?i)config\s+file\s+(.+?)\s+not\s+found").unwrap(), "missing_config_file"),
                (Regex::new(r"(?i)yaml\s+parse\s+error|json\s+parse\s+error|toml\s+parse\s+error").unwrap(), "config_parse_error"),
            ],
            external_service_patterns: vec![
                (Regex::new(r"(?i)http\s+(\d{3})\s+(.+?)").unwrap(), "http_error"),
                (Regex::new(r"(?i)api\s+(error|failure|unavailable)").unwrap(), "api_error"),
                (Regex::new(r"(?i)service\s+(unavailable|down|unreachable)").unwrap(), "service_down"),
                (Regex::new(r"(?i)(rest|soap|graphql)\s+(error|failure)").unwrap(), "web_service_error"),
                (Regex::new(r"(?i)authentication\s+(failed|error)|unauthorized").unwrap(), "auth_error"),
                (Regex::new(r"(?i)rate\s+limit|quota\s+exceeded").unwrap(), "rate_limit"),
            ],
            database_patterns: vec![
                (Regex::new(r"(?i)sql\s+(error|exception|syntax)").unwrap(), "sql_error"),
                (Regex::new(r"(?i)table\s+(.+?)\s+(not\s+found|doesn't\s+exist)").unwrap(), "table_missing"),
                (Regex::new(r"(?i)column\s+(.+?)\s+(not\s+found|unknown)").unwrap(), "column_missing"),
                (Regex::new(r"(?i)constraint\s+(violation|failed)").unwrap(), "constraint_violation"),
                (Regex::new(r"(?i)deadlock|lock\s+timeout").unwrap(), "locking_issue"),
                (Regex::new(r"(?i)connection\s+pool\s+(exhausted|full)").unwrap(), "connection_pool"),
                (Regex::new(r"(?i)migration\s+(failed|error)").unwrap(), "migration_error"),
                (Regex::new(r"(?i)duplicate\s+(key|entry)").unwrap(), "duplicate_key"),
            ],
        }
    }

    pub fn classify_error(&self, log_message: &str, user_context: Option<&str>) -> ErrorClassification {
        let mut matched_patterns = Vec::new();
        let mut confidence = 0.0;
        let mut category = ErrorCategory::UnknownRelated;
        let mut reason = String::new();

        // Check database patterns first (most specific)
        for (pattern, pattern_type) in &self.database_patterns {
            if let Some(captures) = pattern.captures(log_message) {
                matched_patterns.push(format!("database_{}", pattern_type));
                confidence = 0.9;
                category = ErrorCategory::InfrastructureRelated {
                    component: "database".to_string(),
                    severity: self.determine_severity(log_message),
                    service: captures.get(1).map(|m| m.as_str().to_string()),
                };
                reason = format!("Database {} detected in log message", pattern_type);
                break;
            }
        }

        // Check code patterns
        if confidence < 0.8 {
            for (pattern, pattern_type) in &self.code_patterns {
                if let Some(captures) = pattern.captures(log_message) {
                    matched_patterns.push(format!("code_{}", pattern_type));
                    confidence = 0.85;

                    let file = captures.get(2).map(|m| m.as_str().to_string());
                    let line = captures.get(3).and_then(|m| m.as_str().parse().ok());

                    category = ErrorCategory::CodeRelated {
                        file,
                        function: None, // TODO: Extract function name
                        line,
                        exception_type: captures.get(1).map(|m| m.as_str().to_string()),
                    };
                    reason = format!("Code {} pattern matched", pattern_type);
                    break;
                }
            }
        }

        // Check infrastructure patterns
        if confidence < 0.8 {
            for (pattern, pattern_type) in &self.infrastructure_patterns {
                if pattern.is_match(log_message) {
                    matched_patterns.push(format!("infrastructure_{}", pattern_type));
                    confidence = 0.8;
                    category = ErrorCategory::InfrastructureRelated {
                        component: pattern_type.to_string(),
                        severity: self.determine_severity(log_message),
                        service: None,
                    };
                    reason = format!("Infrastructure {} issue detected", pattern_type);
                    break;
                }
            }
        }

        // Check configuration patterns
        if confidence < 0.7 {
            for (pattern, pattern_type) in &self.config_patterns {
                if let Some(captures) = pattern.captures(log_message) {
                    matched_patterns.push(format!("config_{}", pattern_type));
                    confidence = 0.75;

                    category = ErrorCategory::ConfigurationRelated {
                        config_file: None,
                        missing_setting: captures.get(1).map(|m| m.as_str().to_string()),
                        invalid_value: None,
                    };
                    reason = format!("Configuration {} detected", pattern_type);
                    break;
                }
            }
        }

        // Check external service patterns
        if confidence < 0.7 {
            for (pattern, pattern_type) in &self.external_service_patterns {
                if let Some(captures) = pattern.captures(log_message) {
                    matched_patterns.push(format!("external_{}", pattern_type));
                    confidence = 0.7;

                    let status_code = captures.get(1).and_then(|m| m.as_str().parse().ok());

                    category = ErrorCategory::ExternalServiceRelated {
                        service: "unknown".to_string(),
                        endpoint: None,
                        status_code,
                    };
                    reason = format!("External service {} detected", pattern_type);
                    break;
                }
            }
        }

        // Boost confidence if user context aligns
        if let Some(context) = user_context {
            confidence = self.adjust_confidence_for_context(confidence, &category, context);
        }

        ErrorClassification {
            category,
            confidence,
            reason,
            patterns_matched: matched_patterns,
        }
    }

    fn determine_severity(&self, log_message: &str) -> Severity {
        let message_lower = log_message.to_lowercase();

        if message_lower.contains("critical") || message_lower.contains("fatal") || message_lower.contains("emergency") {
            Severity::Critical
        } else if message_lower.contains("error") || message_lower.contains("fail") {
            Severity::High
        } else if message_lower.contains("warn") || message_lower.contains("warning") {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    fn adjust_confidence_for_context(&self, base_confidence: f32, category: &ErrorCategory, user_context: &str) -> f32 {
        let context_lower = user_context.to_lowercase();

        let boost = match category {
            ErrorCategory::CodeRelated { .. } => {
                if context_lower.contains("function") || context_lower.contains("method") ||
                   context_lower.contains("class") || context_lower.contains("bug") {
                    0.1
                } else { 0.0 }
            },
            ErrorCategory::InfrastructureRelated { component, .. } => {
                if context_lower.contains(component) ||
                   context_lower.contains("server") || context_lower.contains("service") {
                    0.15
                } else { 0.0 }
            },
            ErrorCategory::ConfigurationRelated { .. } => {
                if context_lower.contains("config") || context_lower.contains("setting") ||
                   context_lower.contains("environment") {
                    0.1
                } else { 0.0 }
            },
            ErrorCategory::ExternalServiceRelated { .. } => {
                if context_lower.contains("api") || context_lower.contains("service") ||
                   context_lower.contains("endpoint") {
                    0.1
                } else { 0.0 }
            },
            ErrorCategory::UnknownRelated => 0.0,
        };

        (base_confidence + boost).min(1.0)
    }
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}