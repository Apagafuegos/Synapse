use crate::ai_provider::{AIProvider, AnalysisRequest, AnalysisResponse, AIError, ModelInfo};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
    response_mime_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

pub struct GeminiProvider {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            model: "gemini-2.5-flash".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    #[allow(dead_code)]
    fn get_endpoint(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        )
    }
}

#[async_trait::async_trait]
impl AIProvider for GeminiProvider {
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

        // For Gemini, we combine system and user prompts into a single content
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        let content = GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: combined_prompt,
            }],
        };

        let gemini_request = GeminiRequest {
            contents: vec![content],
            generation_config: GeminiGenerationConfig {
                temperature: 0.1, // Low temperature for factual responses
                max_output_tokens: 2000,
                response_mime_type: "application/json".to_string(),
            },
        };

        let response = self
            .client
            .post(self.get_endpoint())
            .header("Content-Type", "application/json")
            .json(&gemini_request)
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

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AIError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if gemini_response.candidates.is_empty() {
            return Err(AIError::InvalidResponse(
                "No candidates in response".to_string(),
            ));
        }

        let candidate = &gemini_response.candidates[0];
        if candidate.content.parts.is_empty() {
            return Err(AIError::InvalidResponse(
                "No content parts in response".to_string(),
            ));
        }

        let content = &candidate.content.parts[0].text;

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
                id: "gemini-2.5-flash".to_string(),
                name: "Gemini 2.5 Flash".to_string(),
                description: Some("Fast multimodal model".to_string()),
                context_length: Some(1000000),
                pricing_tier: Some("low".to_string()),
                capabilities: vec!["chat".to_string(), "vision".to_string(), "analysis".to_string()],
                supports_streaming: true,
                provider: "gemini".to_string(),
            },
            ModelInfo {
                id: "gemini-2.5-pro".to_string(),
                name: "Gemini 2.5 Pro".to_string(),
                description: Some("Advanced reasoning model".to_string()),
                context_length: Some(2000000),
                pricing_tier: Some("high".to_string()),
                capabilities: vec!["chat".to_string(), "vision".to_string(), "analysis".to_string()],
                supports_streaming: true,
                provider: "gemini".to_string(),
            },
        ];

        Ok(models)
    }

    fn get_provider_name(&self) -> &str {
        "gemini"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_creation() {
        let provider = GeminiProvider::new("test_key".to_string());
        assert_eq!(provider.api_key, "test_key");
        assert_eq!(provider.model, "gemini-2.5-flash");
    }

    #[test]
    fn test_gemini_provider_with_model() {
        let provider = GeminiProvider::new("test_key".to_string())
            .with_model("gemini-1.5-pro".to_string());
        assert_eq!(provider.model, "gemini-1.5-pro");
    }

    #[test]
    fn test_gemini_endpoint() {
        let provider = GeminiProvider::new("test_key".to_string());
        let endpoint = provider.get_endpoint();
        assert!(endpoint.contains("generativelanguage.googleapis.com"));
        assert!(endpoint.contains("gemini-2.5-flash"));
        assert!(endpoint.contains("test_key"));
    }
}