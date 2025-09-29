use crate::context_manager::AIAnalysisPayload;
use crate::ai_provider::AnalysisFocus;

pub struct SystemPromptGenerator;

impl SystemPromptGenerator {
    pub fn generate_system_prompt(
        _payload: &AIAnalysisPayload,
        user_context: Option<&str>,
        focus: &AnalysisFocus,
    ) -> String {
        let base_prompt = Self::get_base_system_prompt();
        let focus_prompt = Self::get_focus_specific_prompt(focus);
        let context_prompt = Self::get_context_prompt(user_context);
        let discrimination_prompt = Self::get_error_discrimination_prompt();
        let format_prompt = Self::get_format_prompt();

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            base_prompt, focus_prompt, context_prompt, discrimination_prompt, format_prompt
        )
    }

    fn get_base_system_prompt() -> &'static str {
        r#"You are an expert log analysis AI specializing in root cause identification and error discrimination. Your primary goal is to identify WHAT caused the failure or error, distinguishing between different types of issues.

CORE PRINCIPLES:
1. Focus on ROOT CAUSE identification, not just symptoms
2. Distinguish between code errors, infrastructure issues, configuration problems, and external service failures
3. Provide actionable insights for developers and system administrators
4. Be precise about confidence levels and uncertainty"#
    }

    fn get_focus_specific_prompt(focus: &AnalysisFocus) -> String {
        match focus {
            AnalysisFocus::RootCause => {
                r#"ANALYSIS FOCUS: ROOT CAUSE IDENTIFICATION
- Trace the sequence of events that led to the failure
- Identify the specific component, file, function, or configuration that caused the issue
- Distinguish between triggering events and underlying causes
- Look for cascade failures and their origins"#.to_string()
            }
            AnalysisFocus::Performance => {
                r#"ANALYSIS FOCUS: PERFORMANCE ISSUES
- Identify performance bottlenecks and slow operations
- Look for resource exhaustion (memory, CPU, disk, network)
- Analyze timing patterns and latency issues
- Identify inefficient algorithms or database queries"#.to_string()
            }
            AnalysisFocus::Security => {
                r#"ANALYSIS FOCUS: SECURITY ANALYSIS
- Identify authentication and authorization failures
- Look for potential security vulnerabilities or attacks
- Analyze access control issues and permission problems
- Identify suspicious patterns or anomalous behavior"#.to_string()
            }
            AnalysisFocus::General => {
                r#"ANALYSIS FOCUS: GENERAL ERROR ANALYSIS
- Provide comprehensive analysis across all error types
- Identify patterns and correlations between different issues
- Focus on the most critical and impactful errors"#.to_string()
            }
        }
    }

    fn get_context_prompt(user_context: Option<&str>) -> String {
        match user_context {
            Some(context) => format!(
                r#"USER CONTEXT: {}

IMPORTANT: Focus your analysis on issues related to this user-reported problem. Prioritize errors that could explain or contribute to this specific issue."#,
                context
            ),
            None => r#"USER CONTEXT: No specific context provided.

Analyze all errors with equal priority, focusing on the most severe and impactful issues."#.to_string(),
        }
    }

    fn get_error_discrimination_prompt() -> &'static str {
        r#"ERROR TYPE DISCRIMINATION RULES:

1. CODE-RELATED ERRORS:
   - Stack traces, exceptions, function calls
   - Syntax errors, type mismatches, null pointer exceptions
   - Logic errors, assertion failures, test failures
   - Import/module errors, missing dependencies

2. INFRASTRUCTURE-RELATED ERRORS:
   - Database schema issues, connection pool problems
   - Network timeouts, connection refused, DNS failures
   - Disk space, memory exhaustion, filesystem errors
   - SSL/TLS certificate problems, port conflicts

3. CONFIGURATION-RELATED ERRORS:
   - Missing environment variables, config files
   - Invalid configuration values, YAML/JSON parse errors
   - Property settings, application configuration

4. EXTERNAL SERVICE ERRORS:
   - HTTP status codes (4xx, 5xx), API failures
   - Third-party service outages, rate limiting
   - Authentication failures with external services

CONTEXT SIZE MANAGEMENT:
- If filtered logs exceed token limits, provide summaries with specific line references
- Use line numbers and specific error patterns for reference
- Flag if more context is needed for accurate analysis"#
    }

    fn get_format_prompt() -> &'static str {
        r#"OUTPUT STRUCTURE REQUIREMENTS:

Provide your analysis in this exact JSON format:
{
  "sequence_of_events": "Step-by-step description of what happened",
  "root_cause": {
    "category": {
      "CodeRelated": {
        "file": "optional file path",
        "function": "optional function name",
        "line": 123,
        "exception_type": "optional exception type"
      }
    }
    OR
    "category": {
      "InfrastructureRelated": {
        "component": "database|network|filesystem|memory",
        "severity": "Critical|High|Medium|Low",
        "service": "optional service name"
      }
    }
    OR
    "category": {
      "ConfigurationRelated": {
        "config_file": "optional config file",
        "missing_setting": "optional missing setting",
        "invalid_value": "optional invalid value"
      }
    }
    OR
    "category": {
      "ExternalServiceRelated": {
        "service": "service name",
        "endpoint": "optional endpoint",
        "status_code": 500
      }
    }
    OR
    "category": "UnknownRelated",
    "description": "Specific description of the root cause",
    "file_location": "file path if code-related (optional)",
    "line_number": number if applicable (optional),
    "function_name": "function/method name if applicable (optional)",
    "confidence": 0.0-1.0
  },
  "recommendations": [
    "Specific actionable recommendation 1",
    "Specific actionable recommendation 2"
  ],
  "confidence": 0.0-1.0,
  "related_errors": [
    "List of error messages that are related to the main issue"
  ],
  "unrelated_errors": [
    "List of error messages that are unrelated to the main issue"
  ]
}

CATEGORY EXAMPLES:
- For database issues: {"InfrastructureRelated": {"component": "database", "severity": "High", "service": "PostgreSQL"}}
- For network issues: {"InfrastructureRelated": {"component": "network", "severity": "Critical", "service": null}}
- For code errors: {"CodeRelated": {"file": "auth.py", "function": "login", "line": 45, "exception_type": "NullPointerException"}}
- For config issues: {"ConfigurationRelated": {"config_file": ".env", "missing_setting": "DATABASE_URL", "invalid_value": null}}
- For external APIs: {"ExternalServiceRelated": {"service": "payment-api", "endpoint": "/charge", "status_code": 500}}
- For unknown: "UnknownRelated"

CRITICAL REQUIREMENTS:
- Always distinguish between related and unrelated errors
- Provide specific, actionable recommendations
- Include confidence scores for your analysis
- Reference specific lines, files, or components when possible
- Use the exact category structure shown above
- If you cannot determine the root cause with high confidence, say so and explain why"#
    }

    pub fn create_analysis_prompt(payload: &AIAnalysisPayload) -> String {
        let mut prompt = String::new();

        prompt.push_str("PRIORITY ERRORS (Most relevant to the issue):\n");
        for (i, entry) in payload.priority_entries.iter().enumerate() {
            prompt.push_str(&format!(
                "Entry {} (Score: {:.2}, Category: {:?}):\nLine {}: [{}] {}\n\n",
                i + 1,
                entry.relevance_score,
                entry.classification.category,
                entry.log_entry.line_number.unwrap_or(0),
                entry.log_entry.level.as_deref().unwrap_or("unknown"),
                entry.log_entry.message
            ));
        }

        if !payload.related_entries.is_empty() {
            prompt.push_str("\nRELATED ERRORS (Potentially connected to the issue):\n");
            for (i, entry) in payload.related_entries.iter().enumerate() {
                prompt.push_str(&format!(
                    "Entry {} (Score: {:.2}):\nLine {}: [{}] {}\n\n",
                    i + 1,
                    entry.relevance_score,
                    entry.log_entry.line_number.unwrap_or(0),
                    entry.log_entry.level.as_deref().unwrap_or("unknown"),
                    entry.log_entry.message
                ));
            }
        }

        if let Some(summary) = &payload.unrelated_summary {
            prompt.push_str("\nUNRELATED ERRORS SUMMARY:\n");
            prompt.push_str(summary);
            prompt.push_str("\n");
        }

        prompt.push_str(&format!(
            "\nCONTEXT METADATA:\n- Total entries analyzed: {}\n- Priority entries: {}\n- Estimated tokens: {}\n- Analysis truncated: {}\n",
            payload.context_meta.total_entries_analyzed,
            payload.context_meta.priority_count,
            payload.context_meta.estimated_tokens,
            payload.context_meta.truncated
        ));

        prompt
    }
}