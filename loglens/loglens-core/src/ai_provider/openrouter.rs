use crate::ai_provider::{
    AIError, AIProvider, AnalysisRequest, AnalysisResponse, RootCauseAnalysis, ModelInfo,
};
use crate::classification::ErrorCategory;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, error, warn, debug};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    temperature: f32,
    max_tokens: u32,
    response_format: Option<OpenRouterResponseFormat>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenRouterResponseFormat {
    r#type: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterResponseMessage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenRouterResponseMessage {
    content: String,
}

pub struct OpenRouterProvider {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    api_key: String,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "x-ai/grok-4-fast:free".to_string(), // Default model
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    fn create_fallback_response(content: &str, _json_error: serde_json::Error) -> AnalysisResponse {
        // Extract meaningful information from natural language response
        let lines: Vec<&str> = content.lines().collect();
        let mut sequence_of_events = String::new();
        let mut root_cause_description = String::new();
        let mut recommendations = Vec::new();

        // Simple parsing to extract key information
        let mut in_sequence = false;
        let mut in_recommendations = false;

        for line in lines {
            let line_lower = line.to_lowercase();

            // Look for sequence/analysis sections
            if line_lower.contains("sequence")
                || line_lower.contains("analysis")
                || line_lower.contains("what happened")
            {
                in_sequence = true;
                in_recommendations = false;
                continue;
            }

            // Look for recommendation sections
            if line_lower.contains("recommend")
                || line_lower.contains("suggest")
                || line_lower.contains("should")
            {
                in_recommendations = true;
                in_sequence = false;
                if !line.trim().is_empty() {
                    recommendations.push(line.trim().to_string());
                }
                continue;
            }

            // Look for root cause indicators
            if line_lower.contains("root cause")
                || line_lower.contains("caused by")
                || line_lower.contains("issue is")
            {
                root_cause_description = line.trim().to_string();
                in_sequence = false;
                in_recommendations = false;
                continue;
            }

            // Collect sequence information
            if in_sequence && !line.trim().is_empty() {
                if !sequence_of_events.is_empty() {
                    sequence_of_events.push(' ');
                }
                sequence_of_events.push_str(line.trim());
            }

            // Collect additional recommendations
            if in_recommendations
                && !line.trim().is_empty()
                && (line.starts_with('-')
                    || line.starts_with('*')
                    || line.starts_with("•")
                    || char::is_numeric(line.chars().next().unwrap_or(' ')))
            {
                let rec = line
                    .trim_start_matches(&['-', '*', '•', ' ', '\t'][..])
                    .trim();
                if !rec.is_empty() {
                    recommendations.push(rec.to_string());
                }
            }
        }

        // Fallback values if parsing failed
        if sequence_of_events.is_empty() {
            sequence_of_events = "Analysis completed but response format was not structured JSON. The AI provided natural language analysis.".to_string();
        }

        if root_cause_description.is_empty() && !content.is_empty() {
            root_cause_description = content.chars().take(200).collect::<String>() + "...";
        }

        AnalysisResponse {
            sequence_of_events,
            root_cause: RootCauseAnalysis {
                category: ErrorCategory::UnknownRelated,
                description: root_cause_description,
                file_location: None,
                line_number: None,
                function_name: None,
                confidence: 0.5, // Moderate confidence for fallback parsing
            },
            recommendations,
            confidence: 0.5,
            related_errors: Vec::new(),
            unrelated_errors: Vec::new(),
            errors_found: None,
            patterns: None,
            performance: None,
            anomalies: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModelData>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelData {
    id: String,
    name: String,
    description: Option<String>,
    context_length: Option<u32>,
    pricing: Option<OpenRouterPricing>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
}

#[async_trait::async_trait]
impl AIProvider for OpenRouterProvider {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResponse, AIError> {
        info!("Starting OpenRouter analysis with model: {}", self.model);
        debug!("Analysis focus: {:?}", request.analysis_focus);
        
        use crate::ai_provider::prompts::SystemPromptGenerator;

        // Generate system prompt based on analysis focus
        let system_prompt = SystemPromptGenerator::generate_system_prompt(
            &request.payload,
            request.user_context.as_deref(),
            &request.analysis_focus,
        );
        debug!("Generated system prompt ({} chars)", system_prompt.len());

        // Generate user prompt with log analysis
        let user_prompt = SystemPromptGenerator::create_analysis_prompt(&request.payload);
        debug!("Generated user prompt ({} chars)", user_prompt.len());

        let messages = vec![
            OpenRouterMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OpenRouterMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        let openrouter_request = OpenRouterRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.1, // Low temperature for factual responses
            max_tokens: 2000,
            response_format: Some(OpenRouterResponseFormat {
                r#type: "json_object".to_string(),
            }),
        };

        debug!("Sending OpenRouter request");
        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/loglens/loglens")
            .header("X-Title", "LogLens")
            .header("Content-Type", "application/json")
            .json(&openrouter_request)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send OpenRouter request: {}", e);
                AIError::RequestError(e)
            })?;

        debug!("OpenRouter response status: {}", response.status());
        
        if response.status() == 401 {
            error!("OpenRouter authentication failed");
            return Err(AIError::AuthenticationError);
        }

        if response.status() == 429 {
            warn!("OpenRouter rate limit exceeded");
            return Err(AIError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenRouter API error: {} - {}", status, error_text);
            return Err(AIError::InvalidResponse(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        debug!("Parsing OpenRouter response");
        let openrouter_response: OpenRouterResponse = response
            .json()
            .await
            .map_err(|e| {
                error!("Failed to parse OpenRouter response: {}", e);
                AIError::InvalidResponse(format!("Failed to parse response: {}", e))
            })?;

        if openrouter_response.choices.is_empty() {
            error!("No choices in OpenRouter response");
            return Err(AIError::InvalidResponse(
                "No choices in response".to_string(),
            ));
        }

        let content = &openrouter_response.choices[0].message.content;

        // Parse the JSON response from the AI with fallback for natural language
        let analysis: AnalysisResponse = match serde_json::from_str(content) {
            Ok(parsed) => parsed,
            Err(json_error) => {
                // Fallback: create a structured response from natural language
                Self::create_fallback_response(content, json_error)
            }
        };

        Ok(analysis)
    }

    async fn get_available_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        let response = self
            .client
            .get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/loglens/loglens")
            .header("X-Title", "LogLens")
            .send()
            .await?;

        if response.status() == 401 {
            return Err(AIError::AuthenticationError);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::InvalidResponse(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let models_response: OpenRouterModelsResponse = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to parse models: {}", e)))?;

        let models = models_response
            .data
            .into_iter()
            .map(|model| {
                let pricing_tier = model.pricing.as_ref()
                    .map(|p| {
                        let prompt_price: f64 = p.prompt.parse().unwrap_or(0.0);
                        if prompt_price == 0.0 {
                            "free".to_string()
                        } else if prompt_price < 0.001 {
                            "low".to_string()
                        } else if prompt_price < 0.01 {
                            "medium".to_string()
                        } else {
                            "high".to_string()
                        }
                    });

                ModelInfo {
                    id: model.id.clone(),
                    name: model.name,
                    description: model.description,
                    context_length: model.context_length,
                    pricing_tier,
                    capabilities: vec!["chat".to_string(), "completion".to_string()],
                    provider: "openrouter".to_string(),
                }
            })
            .collect();

        Ok(models)
    }

    fn get_provider_name(&self) -> &str {
        "openrouter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_openrouter_provider_creation() {
        let provider = OpenRouterProvider::new("test_key".to_string());
        assert_eq!(provider.api_key, "test_key");
        assert_eq!(provider.model, "deepseek/deepseek-chat-v3.1:free");
    }

    #[test]
    fn test_openrouter_provider_with_model() {
        let provider =
            OpenRouterProvider::new("test_key".to_string()).with_model("gpt-4".to_string());
        assert_eq!(provider.model, "gpt-4");
    }

    #[test]
    fn test_fallback_response_creation() {
        let natural_language_response = r#"Based on the log analysis, I can see that there are database connection issues.

The sequence of events appears to be:
1. Initial database connection attempt failed
2. System attempted to retry the connection
3. Connection eventually timed out after 30 seconds

Root cause: This appears to be a database connectivity issue, possibly due to network problems or the database being unavailable.

Recommendations:
- Check if the database server is running
- Verify network connectivity to the database
- Consider increasing connection timeout values
- Implement connection pooling for better reliability"#;

        use std::io;

        let response = OpenRouterProvider::create_fallback_response(
            natural_language_response,
            serde_json::Error::io(io::Error::new(io::ErrorKind::Other, "test error")),
        );

        // Verify the fallback response structure
        assert!(!response.sequence_of_events.is_empty());
        assert_eq!(
            response.root_cause.category,
            crate::classification::ErrorCategory::UnknownRelated
        );
        assert!(!response.root_cause.description.is_empty());
        assert_eq!(response.confidence, 0.5);
        assert!(!response.recommendations.is_empty());
        assert_eq!(response.related_errors, Vec::<String>::new());
        assert_eq!(response.unrelated_errors, Vec::<String>::new());

        // Verify specific content extraction
        // Check that content was extracted (more lenient checks)
        assert!(response.sequence_of_events.len() > 10); // Should have some content
        assert!(response.root_cause.description.len() > 10); // Should have some description
        assert!(response.recommendations.len() > 0); // Should have recommendations
        assert!(response
            .recommendations
            .iter()
            .any(|r| r.to_lowercase().contains("database")));
    }

    #[test]
    fn test_fallback_response_empty_content() {
        let empty_response = "";
        let response = OpenRouterProvider::create_fallback_response(
            empty_response,
            serde_json::Error::io(io::Error::new(io::ErrorKind::Other, "test error")),
        );

        // Should handle empty content gracefully
        assert!(!response.sequence_of_events.is_empty());
        assert_eq!(response.confidence, 0.5);
        assert_eq!(response.recommendations, Vec::<String>::new());
    }

    #[test]
    fn test_fallback_response_with_realistic_content() {
        let realistic_content = r#"That is one of the most profound and enduring questions humanity has ever asked. There isn't one single, definitive answer, and the meaning of life is something people have explored through philosophy, religion, science, and art for millennia.

Here's a breakdown of different ways to approach this question:

### 1. The Philosophical Perspectives

Philosophers have proposed many theories:

*   **Existentialism (e.g., Sartre, Camus):** This school of thought argues that life has no *inherent* meaning. We are "condemned to be free," thrown into existence without a pre-ordained purpose. Therefore, the meaning of life is not something you *find*, but something you *create* for yourself through your choices, actions, and passions.

### 2. The Root Cause Analysis

The sequence of events appears to be:
1. Initial database connection attempt failed
2. System attempted to retry the connection
3. Connection eventually timed out after 30 seconds

Root cause: This appears to be a database connectivity issue, possibly due to network problems or the database being unavailable.

Recommendations:
- Check if the database server is running
- Verify network connectivity to the database
- Consider increasing connection timeout values"#;

        let response = OpenRouterProvider::create_fallback_response(
            realistic_content,
            serde_json::Error::io(io::Error::new(io::ErrorKind::Other, "test error")),
        );

        // Should handle realistic content gracefully
        assert!(!response.sequence_of_events.is_empty());
        assert_eq!(response.confidence, 0.5);
        assert!(response.recommendations.len() > 0);

        // Check that specific content was extracted
        assert!(response.sequence_of_events.contains("sequence of events"));
        assert!(response
            .root_cause
            .description
            .to_lowercase()
            .contains("database"));
        assert!(response
            .recommendations
            .iter()
            .any(|r| r.to_lowercase().contains("database")));
    }
}
