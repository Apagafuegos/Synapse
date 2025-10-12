use crate::ai_provider::{AIProvider, AnalysisRequest, AnalysisResponse, AIError, ModelInfo};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
    response_format: Option<OpenAIResponseFormat>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct OpenAIResponseFormat {
    response_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

pub struct OpenAIProvider {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "gpt-5-nano".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait::async_trait]
impl AIProvider for OpenAIProvider {
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

        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.1, // Low temperature for factual responses
            max_tokens: 2000,
            response_format: Some(OpenAIResponseFormat {
                response_type: "json_object".to_string(),
            }),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
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
                status,
                error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if openai_response.choices.is_empty() {
            return Err(AIError::InvalidResponse(
                "No choices in response".to_string(),
            ));
        }

        let content = &openai_response.choices[0].message.content;

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
        // OpenAI has a well-known set of models - we'll use static data
        // In a production app, you'd call https://api.openai.com/v1/models
        let models = vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                description: Some("Most capable GPT-4 model".to_string()),
                context_length: Some(8192),
                pricing_tier: Some("high".to_string()),
                capabilities: vec!["chat".to_string(), "completion".to_string()],
                supports_streaming: true,
                provider: "openai".to_string(),
            },
            ModelInfo {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                description: Some("Latest GPT-4 model with improved speed".to_string()),
                context_length: Some(128000),
                pricing_tier: Some("high".to_string()),
                capabilities: vec!["chat".to_string(), "completion".to_string()],
                supports_streaming: true,
                provider: "openai".to_string(),
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                description: Some("Fast and efficient model".to_string()),
                context_length: Some(4096),
                pricing_tier: Some("medium".to_string()),
                capabilities: vec!["chat".to_string(), "completion".to_string()],
                supports_streaming: true,
                provider: "openai".to_string(),
            },
        ];

        Ok(models)
    }

    fn get_provider_name(&self) -> &str {
        "openai"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test_key".to_string());
        assert_eq!(provider.api_key, "test_key");
        assert_eq!(provider.model, "gpt-5-nano");
    }

    #[test]
    fn test_openai_provider_with_model() {
        let provider = OpenAIProvider::new("test_key".to_string())
            .with_model("gpt-4".to_string());
        assert_eq!(provider.model, "gpt-4");
    }
}