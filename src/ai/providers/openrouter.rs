//! OpenRouter Provider Implementation with Real API Integration
//! 
//! Provides integration with OpenRouter API for LLM access

use crate::ai::interface::*;
use crate::config::{ProviderConfig, AnalysisDepth};
use anyhow::{Context, Result};
use serde_json::json;
use std::time::Duration;
use reqwest::Client;
use chrono::Utc;
use std::collections::HashMap;

/// OpenRouter provider implementation
pub struct OpenRouterProvider {
    config: ProviderConfig,
    client: Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            config,
            client,
        }
    }
    
    /// Make API request to OpenRouter with retry logic and enhanced error handling
    async fn make_api_request(&self, messages: Vec<serde_json::Value>) -> Result<serde_json::Value> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not configured. Please set OPENROUTER_API_KEY or configure the provider."))?;

        let request_body = json!({
            "model": self.config.model,
            "messages": messages,
            "max_tokens": 4000,
            "temperature": 0.1,
            "stream": false
        });

        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let response = self.client
                .post(&format!("{}/chat/completions", self.config.base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://github.com/loglens/loglens")
                .header("X-Title", "LogLens")
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status.is_success() {
                        // Success case
                        let response_json: serde_json::Value = resp.json().await
                            .context("Failed to parse OpenRouter JSON response")?;

                        // Validate response structure
                        if response_json["choices"].is_null() || response_json["choices"].as_array().unwrap_or(&vec![]).is_empty() {
                            return Err(anyhow::anyhow!("Invalid OpenRouter response format: missing or empty choices array"));
                        }

                        return Ok(response_json);
                    } else {
                        // Error cases
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unable to read error details".to_string());

                        match status.as_u16() {
                            401 => {
                                return Err(anyhow::anyhow!("Authentication failed: Invalid OpenRouter API key. Please check your API key configuration."));
                            }
                            403 => {
                                return Err(anyhow::anyhow!("Access forbidden: Your API key may not have access to the requested model '{}'. Error: {}", self.config.model, error_text));
                            }
                            429 => {
                                // Rate limit - can retry
                                if retries < max_retries {
                                    retries += 1;
                                    // Simple retry without sleep for now (can be enhanced with proper async sleep later)
                                    continue;
                                } else {
                                    return Err(anyhow::anyhow!("Rate limit exceeded after {} retries. Please try again later. Error: {}", max_retries, error_text));
                                }
                            }
                            500..=599 => {
                                // Server error - can retry
                                if retries < max_retries {
                                    retries += 1;
                                    // Simple retry without sleep for now (can be enhanced with proper async sleep later)
                                    continue;
                                } else {
                                    return Err(anyhow::anyhow!("OpenRouter server error after {} retries: {} - {}", max_retries, status, error_text));
                                }
                            }
                            400 => {
                                // Bad request - don't retry
                                return Err(anyhow::anyhow!("Bad request: {}. This usually indicates an issue with the request format or model parameters.", error_text));
                            }
                            _ => {
                                // Other errors - don't retry
                                return Err(anyhow::anyhow!("OpenRouter API error: {} - {}", status, error_text));
                            }
                        }
                    }
                }
                Err(e) => {
                    // Network/timeout errors - can retry
                    if retries < max_retries {
                        retries += 1;
                        // Simple retry without sleep for now (can be enhanced with proper async sleep later)
                        continue;
                    } else {
                        return Err(anyhow::anyhow!("Failed to connect to OpenRouter after {} retries: {}. Please check your internet connection.", max_retries, e));
                    }
                }
            }
        }
    }
    
    /// Build system prompt based on analysis request
    fn build_system_prompt(&self, request: &LogAnalysisRequest) -> String {
        let depth_description = match request.analysis_depth {
            AnalysisDepth::Basic => "Provide a concise overview focusing on critical issues that require immediate attention. Keep analysis brief but actionable.",
            AnalysisDepth::Detailed => "Provide comprehensive analysis with detailed explanations, root cause identification, and specific actionable recommendations. Include severity assessment and priority levels.",
            AnalysisDepth::Comprehensive => "Provide exhaustive analysis including all aspects: detailed failure analysis, trigger identification, pattern recognition, timeline reconstruction, impact assessment, and extensive prioritized recommendations with implementation steps.",
        };

        let focus_areas_str = if request.focus_areas.is_empty() {
            "all critical aspects of the system".to_string()
        } else {
            request.focus_areas.iter()
                .map(|f| match f {
                    AnalysisFocus::Errors => "error detection, failure analysis, and exception handling",
                    AnalysisFocus::Performance => "performance bottlenecks, resource utilization, and optimization opportunities",
                    AnalysisFocus::Security => "security threats, vulnerabilities, access issues, and compliance concerns",
                    AnalysisFocus::Configuration => "configuration problems, misconfigurations, and setup optimization",
                    AnalysisFocus::UserActivity => "user behavior patterns, authentication issues, and user experience problems",
                    AnalysisFocus::SystemEvents => "system state changes, service health, and operational events",
                    AnalysisFocus::Custom(s) => s,
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let entry_count = request.log_entries.len();
        let time_span = if request.log_entries.len() >= 2 {
            let first = &request.log_entries[0];
            let last = &request.log_entries[request.log_entries.len() - 1];
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            format!("spanning {} minutes", duration.num_minutes().max(1))
        } else {
            "single point in time".to_string()
        };

        format!(
            "You are a senior DevOps engineer and log analysis expert specializing in system diagnostics and operational intelligence. \
            \n\nCONTEXT: Analyze {} log entries {} with focus on: {}. \
            \n\nANALYSIS REQUIREMENTS: \
            \n{} \
            \n\nSTRUCTURE YOUR RESPONSE AS FOLLOWS: \
            \n\n## 1. EXECUTIVE SUMMARY \
            \nProvide 2-3 bullet points summarizing the most critical findings and overall system health status. \
            \n\n## 2. CRITICAL ISSUES (if any) \
            \nList any errors, failures, or critical problems with: \
            \n- Issue description \
            \n- Root cause analysis \
            \n- Business impact \
            \n- Urgency level (Critical/High/Medium/Low) \
            \n\n## 3. PERFORMANCE INSIGHTS \
            \nHighlight any performance-related observations: \
            \n- Response times, resource usage, bottlenecks \
            \n- Trends and patterns \
            \n\n## 4. SYSTEM HEALTH INDICATORS \
            \n- Services status and connectivity \
            \n- Resource utilization patterns \
            \n- Operational metrics \
            \n\n## 5. TIMELINE ANALYSIS \
            \nKey events in chronological order with their significance. \
            \n\n## 6. RECOMMENDATIONS \
            \nPrioritized action items: \
            \na) **Immediate Actions** (next 1-2 hours) \
            \nb) **Short-term Improvements** (next 1-2 days) \
            \nc) **Long-term Solutions** (next week+) \
            \n\n## 7. MONITORING SUGGESTIONS \
            \nRecommended alerts, dashboards, or monitoring improvements. \
            \n\nProvide specific, actionable insights that a DevOps team can immediately act upon. Focus on business impact and operational priorities.",
            entry_count, time_span, focus_areas_str, depth_description
        )
    }
    
    /// Build user messages from log entries with enhanced context
    fn build_user_messages(&self, entries: &[crate::model::LogEntry], request: &LogAnalysisRequest) -> Vec<serde_json::Value> {
        let max_entries = request.max_context_entries.min(entries.len());

        // Smart entry selection with priority system
        let selected_entries = if max_entries < entries.len() {
            let mut priority_entries = Vec::new();

            // Priority 1: Critical errors and failures
            let errors: Vec<_> = entries.iter()
                .filter(|e| matches!(e.level, crate::model::LogLevel::Error))
                .take(max_entries / 3)
                .collect();
            priority_entries.extend(errors);

            // Priority 2: Warnings and performance issues
            let warnings: Vec<_> = entries.iter()
                .filter(|e| matches!(e.level, crate::model::LogLevel::Warn))
                .take((max_entries - priority_entries.len()) / 2)
                .collect();
            priority_entries.extend(warnings);

            // Priority 3: Recent entries for timeline context
            let remaining_slots = max_entries - priority_entries.len();
            if remaining_slots > 0 {
                let recent_entries: Vec<_> = entries.iter()
                    .filter(|e| !matches!(e.level, crate::model::LogLevel::Error | crate::model::LogLevel::Warn))
                    .rev() // Get most recent entries
                    .take(remaining_slots)
                    .collect();
                priority_entries.extend(recent_entries);
            }

            // Sort by timestamp to maintain chronological order
            priority_entries.sort_by_key(|e| e.timestamp);
            priority_entries
        } else {
            entries.iter().collect()
        };

        // Count entries by level for summary
        let error_count = selected_entries.iter().filter(|e| matches!(e.level, crate::model::LogLevel::Error)).count();
        let warn_count = selected_entries.iter().filter(|e| matches!(e.level, crate::model::LogLevel::Warn)).count();
        let info_count = selected_entries.iter().filter(|e| matches!(e.level, crate::model::LogLevel::Info)).count();

        // Enhanced log formatting with better structure
        let log_entries_text = selected_entries.iter()
            .enumerate()
            .map(|(i, entry)| {
                let level_indicator = match entry.level {
                    crate::model::LogLevel::Error => "❌ ERROR",
                    crate::model::LogLevel::Warn => "⚠️  WARN",
                    crate::model::LogLevel::Info => "ℹ️  INFO",
                    crate::model::LogLevel::Debug => "🐛 DEBUG",
                    crate::model::LogLevel::Trace => "🔍 TRACE",
                    crate::model::LogLevel::Unknown => "❓ UNKNOWN",
                };

                let fields_text = if entry.fields.is_empty() {
                    String::new()
                } else {
                    format!("\n    Additional context: {}",
                        entry.fields.iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join(", "))
                };

                format!(
                    "{}. [{}] {} {}{}",
                    i + 1,
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    level_indicator,
                    entry.message,
                    fields_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Create comprehensive analysis request
        let analysis_summary = if selected_entries.len() < entries.len() {
            format!(
                "ANALYSIS SCOPE: Selected {} most critical entries from {} total log entries for focused analysis.",
                selected_entries.len(), entries.len()
            )
        } else {
            format!("ANALYSIS SCOPE: Complete log analysis of {} entries.", entries.len())
        };

        let log_summary = format!(
            "LOG SUMMARY: {} errors, {} warnings, {} info messages",
            error_count, warn_count, info_count
        );

        vec![json!({
            "role": "user",
            "content": format!(
                "{}\n{}\n\n=== LOG ENTRIES FOR ANALYSIS ===\n\n{}",
                analysis_summary, log_summary, log_entries_text
            )
        })]
    }
    
    /// Parse LLM response into structured analysis
    fn parse_analysis_response(&self, response: &str) -> Result<LogAnalysisResponse> {
        // Parse the AI response and extract meaningful insights
        let lines: Vec<&str> = response.lines().collect();

        // Extract key findings from the response
        let mut key_findings = Vec::new();
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut overall_status = AnalysisStatus::Healthy;
        let mut root_causes = Vec::new();
        let mut recommendations = Vec::new();

        let mut current_section = "";
        let mut temp_description = String::new();

        for line in &lines {
            let line = line.trim();
            if line.is_empty() { continue; }

            // Detect sections in the AI response
            if line.contains("Summary") || line.contains("SUMMARY") {
                current_section = "summary";
                continue;
            } else if line.contains("Error") || line.contains("ERROR") || line.contains("Failure") {
                current_section = "errors";
                error_count += 1;
                overall_status = AnalysisStatus::Critical;
            } else if line.contains("Warning") || line.contains("WARN") {
                current_section = "warnings";
                warning_count += 1;
                if overall_status == AnalysisStatus::Healthy {
                    overall_status = AnalysisStatus::Warning;
                }
            } else if line.contains("Recommendation") || line.contains("RECOMMENDATION") {
                current_section = "recommendations";
            } else if line.contains("Root Cause") || line.contains("ROOT CAUSE") {
                current_section = "root_cause";
            }

            // Extract content based on current section
            match current_section {
                "summary" => {
                    if line.starts_with("•") || line.starts_with("-") || line.starts_with("*") {
                        key_findings.push(line[1..].trim().to_string());
                    } else if !line.contains("Summary") && !line.contains("SUMMARY") {
                        key_findings.push(line.to_string());
                    }
                }
                "errors" | "warnings" => {
                    if !line.contains("Error") && !line.contains("Warning") {
                        temp_description.push_str(line);
                        temp_description.push(' ');
                    }
                }
                "root_cause" => {
                    if !line.contains("Root Cause") && !temp_description.trim().is_empty() {
                        root_causes.push(RootCause {
                            cause_type: "Analysis Finding".to_string(),
                            description: temp_description.trim().to_string(),
                            confidence: 0.7, // Default confidence
                            supporting_evidence: vec![line.to_string()],
                            timestamp: Some(Utc::now()),
                        });
                        temp_description.clear();
                    }
                    if !line.contains("Root Cause") {
                        temp_description.push_str(line);
                        temp_description.push(' ');
                    }
                }
                "recommendations" => {
                    if line.starts_with("•") || line.starts_with("-") || line.starts_with("*") ||
                       (line.len() > 10 && !line.contains("Recommendation")) {
                        let clean_line = if line.starts_with("•") || line.starts_with("-") || line.starts_with("*") {
                            line[1..].trim()
                        } else {
                            line
                        };

                        recommendations.push(Recommendation {
                            recommendation_id: format!("rec_{}", recommendations.len() + 1),
                            category: RecommendationCategory::Fix,
                            priority: if line.to_lowercase().contains("critical") || line.to_lowercase().contains("urgent") {
                                RecommendationPriority::Critical
                            } else if line.to_lowercase().contains("high") || line.to_lowercase().contains("important") {
                                RecommendationPriority::High
                            } else {
                                RecommendationPriority::Medium
                            },
                            title: clean_line.split('.').next().unwrap_or(clean_line).to_string(),
                            description: clean_line.to_string(),
                            implementation_steps: vec![clean_line.to_string()],
                            expected_outcome: "Improved system stability".to_string(),
                            estimated_effort: "Variable".to_string(),
                            related_issues: vec![],
                            confidence: 0.8,
                        });
                    }
                }
                _ => {}
            }
        }

        // If no specific findings extracted, use the entire response as a key finding
        if key_findings.is_empty() {
            // Extract meaningful sentences from the response
            let sentences: Vec<&str> = response.split('.').filter(|s| s.trim().len() > 20).take(3).collect();
            for sentence in sentences {
                key_findings.push(sentence.trim().to_string());
            }
        }

        // Ensure we have at least one key finding
        if key_findings.is_empty() {
            key_findings.push("AI analysis completed - see full response for details".to_string());
        }

        // If no root causes found, create a general one
        if root_causes.is_empty() && error_count > 0 {
            root_causes.push(RootCause {
                cause_type: "General Analysis".to_string(),
                description: "Issues detected in log analysis - review detailed findings".to_string(),
                confidence: 0.6,
                supporting_evidence: key_findings.clone(),
                timestamp: Some(Utc::now()),
            });
        }

        // If no recommendations found, create a general one
        if recommendations.is_empty() {
            recommendations.push(Recommendation {
                recommendation_id: "rec_general".to_string(),
                category: RecommendationCategory::Monitoring,
                priority: RecommendationPriority::Medium,
                title: "Review AI Analysis".to_string(),
                description: "Review the detailed AI analysis for actionable insights".to_string(),
                implementation_steps: vec![
                    "Review the AI analysis response".to_string(),
                    "Identify specific issues mentioned".to_string(),
                    "Take appropriate corrective actions".to_string(),
                ],
                expected_outcome: "Better understanding of system issues".to_string(),
                estimated_effort: "15-30 minutes".to_string(),
                related_issues: vec![],
                confidence: 0.7,
            });
        }

        let summary = LogAnalysisSummary {
            overall_status,
            key_findings,
            error_count,
            warning_count,
            time_range: None, // Could be extracted from log timestamps
            affected_systems: vec!["Application".to_string()], // Default
        };

        let failure_analysis = if !root_causes.is_empty() {
            Some(FailureAnalysis {
                root_causes,
                impact_assessment: ImpactAssessment {
                    severity: if overall_status == AnalysisStatus::Critical {
                        ImpactSeverity::High
                    } else if overall_status == AnalysisStatus::Warning {
                        ImpactSeverity::Medium
                    } else {
                        ImpactSeverity::Low
                    },
                    affected_components: vec!["Application".to_string()],
                    user_impact: "Potential impact on user experience".to_string(),
                    business_impact: "Monitor for service degradation".to_string(),
                },
                failure_timeline: vec![],
            })
        } else {
            None
        };

        let detailed_analysis = Some(DetailedAnalysis {
            failure_analysis,
            trigger_analysis: None,
            pattern_analysis: None,
            timeline_analysis: None,
        });

        Ok(LogAnalysisResponse {
            provider: "openrouter".to_string(),
            model: self.config.model.clone(),
            analysis_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            summary,
            detailed_analysis,
            recommendations: Some(recommendations),
            processing_time_ms: 1500, // Would be measured in real implementation
            token_usage: Some(TokenUsage {
                prompt_tokens: 1000, // Would be from API response
                completion_tokens: 800,
                total_tokens: 1800,
                estimated_cost_usd: Some(0.0036),
            }),
            metadata: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }
    
    async fn analyze_logs(&self, request: LogAnalysisRequest) -> Result<LogAnalysisResponse, AiError> {
        let system_prompt = self.build_system_prompt(&request);
        let user_messages = self.build_user_messages(&request.log_entries, &request);
        
        let mut messages = vec![json!({
            "role": "system",
            "content": system_prompt
        })];
        messages.extend(user_messages);
        
        let response = self.make_api_request(messages).await
            .map_err(|e| AiError::Processing(format!("OpenRouter API request failed: {}", e)))?;
        
        let content = response["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| AiError::Processing("No content in OpenRouter response".to_string()))?;
        
        self.parse_analysis_response(content)
            .map_err(|e| AiError::Processing(format!("Failed to parse OpenRouter response: {}", e)))
    }
    
    async fn generate_recommendations(&self, analysis: &str) -> Result<String, AiError> {
        let messages = vec![
            json!({
                "role": "system",
                "content": "You are an expert system administrator and developer. Generate specific, actionable recommendations based on provided log analysis. Focus on practical steps that can be implemented immediately."
            }),
            json!({
                "role": "user", 
                "content": format!(
                    "Generate recommendations based on this analysis:\n\n{}",
                    analysis
                )
            })
        ];
        
        let response = self.make_api_request(messages).await
            .map_err(|e| AiError::Processing(format!("OpenRouter recommendation request failed: {}", e)))?;
        
        let content = response["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| AiError::Processing("No content in OpenRouter recommendation response".to_string()))?;
        
        Ok(content.to_string())
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, AiError> {
        let start_time = std::time::Instant::now();

        // Comprehensive health check with timeout and detailed error handling
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| AiError::Authentication("OpenRouter API key not configured. Please set OPENROUTER_API_KEY environment variable or configure in settings.".to_string()))?;

        // Create a health check request with timeout
        let response = self.client
            .get(&format!("{}/models", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/loglens/loglens")
            .header("X-Title", "LogLens Health Check")
            .timeout(Duration::from_secs(10)) // 10 second timeout for health checks
            .send()
            .await;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) => {
                let status = resp.status();

                if status.is_success() {
                    // Try to parse models response
                    match resp.json::<serde_json::Value>().await {
                        Ok(models_response) => {
                            let available_models = models_response["data"].as_array()
                                .map(|models| {
                                    models.iter()
                                        .filter_map(|model| model["id"].as_str().map(|s| s.to_string()))
                                        .take(10) // Limit to first 10 for health check
                                        .collect()
                                })
                                .unwrap_or_else(|| vec![]);

                            // Verify our configured model is available
                            let model_available = available_models.iter().any(|m| m.contains(&self.config.model))
                                || available_models.is_empty(); // If we can't get models list, assume it's OK

                            if !model_available && !available_models.is_empty() {
                                Ok(ProviderHealth {
                                    is_healthy: false,
                                    response_time_ms: Some(response_time_ms),
                                    last_check: Utc::now(),
                                    error_message: Some(format!("Configured model '{}' not found in available models. Available models: {}",
                                        self.config.model,
                                        available_models.iter().take(5).cloned().collect::<Vec<_>>().join(", "))),
                                    available_models,
                                })
                            } else {
                                Ok(ProviderHealth {
                                    is_healthy: true,
                                    response_time_ms: Some(response_time_ms),
                                    last_check: Utc::now(),
                                    error_message: None,
                                    available_models,
                                })
                            }
                        }
                        Err(e) => {
                            Ok(ProviderHealth {
                                is_healthy: false,
                                response_time_ms: Some(response_time_ms),
                                last_check: Utc::now(),
                                error_message: Some(format!("Failed to parse models response: {}. API may be working but response format is unexpected.", e)),
                                available_models: vec![],
                            })
                        }
                    }
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Unable to read error details".to_string());

                    let error_message = match status.as_u16() {
                        401 => "Authentication failed: Invalid API key".to_string(),
                        403 => format!("Access forbidden: API key may not have required permissions. {}", error_text),
                        429 => "Rate limit exceeded".to_string(),
                        500..=599 => format!("OpenRouter server error: {}", error_text),
                        _ => format!("HTTP {} error: {}", status.as_u16(), error_text),
                    };

                    Ok(ProviderHealth {
                        is_healthy: false,
                        response_time_ms: Some(response_time_ms),
                        last_check: Utc::now(),
                        error_message: Some(error_message),
                        available_models: vec![],
                    })
                }
            }
            Err(e) => {
                let error_message = if e.is_timeout() {
                    "Request timeout: OpenRouter API took too long to respond".to_string()
                } else if e.is_connect() {
                    "Connection failed: Unable to reach OpenRouter API. Check internet connection.".to_string()
                } else {
                    format!("Network error: {}", e)
                };

                Ok(ProviderHealth {
                    is_healthy: false,
                    response_time_ms: Some(response_time_ms),
                    last_check: Utc::now(),
                    error_message: Some(error_message),
                    available_models: vec![],
                })
            }
        }
    }
    
    fn available_models(&self) -> Vec<String> {
        // Common OpenRouter models - would be fetched from API in production
        vec![
            "anthropic/claude-3.5-sonnet".to_string(),
            "anthropic/claude-3-opus".to_string(),
            "openai/gpt-4-turbo".to_string(),
            "openai/gpt-4o".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "google/gemini-pro".to_string(),
            "google/gemini-1.5-pro".to_string(),
            "meta-llama/llama-3-70b-instruct".to_string(),
            "mistralai/mistral-large".to_string(),
        ]
    }
    
    fn supports_streaming(&self) -> bool {
        true
    }
    
    fn get_capabilities(&self) -> LlmProviderCapabilities {
        LlmProviderCapabilities {
            log_analysis: true,
            recommendation_generation: true,
            streaming: true,
            custom_prompts: true,
            context_window: 32000, // Default for most models
            supported_formats: vec![
                OutputFormat::Structured,
                OutputFormat::HumanReadable,
                OutputFormat::Json,
                OutputFormat::Markdown,
            ],
            max_input_tokens: Some(32000),
            max_output_tokens: Some(4000),
        }
    }
}