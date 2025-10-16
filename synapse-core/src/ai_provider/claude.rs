use crate::ai_provider::{AIError, AIProvider, AnalysisRequest, AnalysisResponse, ModelInfo};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

pub struct ClaudeProvider {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "claude-haiku-3".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait::async_trait]
impl AIProvider for ClaudeProvider {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResponse, AIError> {
        use crate::ai_provider::prompts::SystemPromptGenerator;

        // Generate system prompt based on analysis focus
        let system_prompt = SystemPromptGenerator::generate_system_prompt(
            &request.payload,
            request.user_context.as_deref(),
            &request.analysis_focus,
        );

        // Generate user prompt with log analysis
        let user_prompt = SystemPromptGenerator::create_analysis_prompt(&request.payload);

        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: user_prompt,
        }];

        let claude_request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 2000,
            temperature: 0.1, // Low temperature for factual responses
            system: system_prompt,
            messages,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&claude_request)
            .send()
            .await?;

        if response.status() == 401 {
            return Err(AIError::AuthenticationError);
        }

        if response.status() == 429 {
            return Err(AIError::RateLimited);
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

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if claude_response.content.is_empty() {
            return Err(AIError::InvalidResponse(
                "No content in response".to_string(),
            ));
        }

        let content = &claude_response.content[0].text;

        // Parse the JSON response from the AI
        let analysis: AnalysisResponse = serde_json::from_str(content).map_err(|e| {
            AIError::InvalidResponse(format!(
                "Failed to parse AI response as JSON: {}. Content: {}",
                e, content
            ))
        })?;

        Ok(analysis)
    }

    async fn get_available_models(&self) -> Result<Vec<ModelInfo>, AIError> {
        let models = vec![
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                description: Some("Most intelligent model".to_string()),
                context_length: Some(200000),
                pricing_tier: Some("high".to_string()),
                capabilities: vec![
                    "chat".to_string(),
                    "analysis".to_string(),
                    "vision".to_string(),
                ],
                supports_streaming: true,
                provider: "claude".to_string(),
            },
            ModelInfo {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                description: Some("Fast and efficient model".to_string()),
                context_length: Some(200000),
                pricing_tier: Some("low".to_string()),
                capabilities: vec!["chat".to_string(), "analysis".to_string()],
                supports_streaming: true,
                provider: "claude".to_string(),
            },
        ];

        Ok(models)
    }

    fn get_provider_name(&self) -> &str {
        "claude"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider_creation() {
        let provider = ClaudeProvider::new("test_key".to_string());
        assert_eq!(provider.api_key, "test_key");
        assert_eq!(provider.model, "claude-haiku-3");
    }

    #[test]
    fn test_claude_provider_with_model() {
        let provider = ClaudeProvider::new("test_key".to_string())
            .with_model("claude-3-opus-20240229".to_string());
        assert_eq!(provider.model, "claude-3-opus-20240229");
    }
}
