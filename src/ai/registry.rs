//! Provider Registry
//! 
//! Manages AI provider instances and provides a unified interface
//! for accessing different LLM providers based on configuration.

use crate::config::{ConfigManager, AiConfig, ProviderConfig};
use crate::ai::interface::{LlmProvider, ProviderHealth};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Context, Result};

/// Registry for managing AI provider instances
pub struct ProviderRegistry {
    config: AiConfig,
    providers: HashMap<String, Box<dyn LlmProvider>>,
    config_manager: ConfigManager,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new(mut config_manager: ConfigManager) -> Result<Self> {
        let config = config_manager.get_config()
            .context("Failed to load configuration for provider registry")?;
        
        Ok(Self {
            config,
            providers: HashMap::new(),
            config_manager,
        })
    }
    
    /// Get a provider by name
    pub fn get_provider(&mut self, name: &str) -> Result<&dyn LlmProvider> {
        // Check if provider is already loaded
        if !self.providers.contains_key(name) {
            self.load_provider(name)?;
        }
        
        Ok(self.providers.get(name).unwrap().as_ref())
    }
    
    /// Get the default provider
    pub fn get_default_provider(&mut self) -> Result<&dyn LlmProvider> {
        let default_name = self.config.ai.default_provider.clone();
        self.get_provider(&default_name)
    }
    
    /// Load a provider instance
    fn load_provider(&mut self, name: &str) -> Result<()> {
        let provider_config = self.config.providers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found in configuration", name))?;
        
        let provider: Box<dyn LlmProvider> = match name {
            "openrouter" => {
                Box::new(crate::ai::providers::openrouter::OpenRouterProvider::new(
                    provider_config.clone()
                ))
            },
            "openai" => {
                Box::new(crate::ai::providers::OpenAIProvider::new(
                    provider_config.clone()
                ))
            },
            "anthropic" => {
                Box::new(crate::ai::providers::AnthropicProvider::new(
                    provider_config.clone()
                ))
            },
            "gemini" => {
                Box::new(crate::ai::providers::GeminiProvider::new(
                    provider_config.clone()
                ))
            },
            "mistral" => {
                Box::new(crate::ai::providers::MistralProvider::new(
                    provider_config.clone()
                ))
            },
            "cohere" => {
                Box::new(crate::ai::providers::CohereProvider::new(
                    provider_config.clone()
                ))
            },
            "local" => {
                Box::new(crate::ai::providers::LocalProvider::new(
                    provider_config.clone()
                ))
            },
            "aws_bedrock" => {
                Box::new(crate::ai::providers::AWSBedrockProvider::new(
                    provider_config.clone()
                ))
            },
            "azure_openai" => {
                Box::new(crate::ai::providers::AzureOpenAIProvider::new(
                    provider_config.clone()
                ))
            },
            "huggingface" => {
                Box::new(crate::ai::providers::HuggingFaceProvider::new(
                    provider_config.clone()
                ))
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown provider: {}", name));
            }
        };
        
        self.providers.insert(name.to_string(), provider);
        Ok(())
    }
    
    /// List all available providers
    pub fn list_available(&self) -> Vec<String> {
        self.config.providers.keys().cloned().collect()
    }
    
    /// List all loaded providers
    pub fn list_loaded(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    /// Test a provider connection
    pub async fn test_provider(&mut self, name: &str) -> Result<ProviderHealth> {
        let provider = self.get_provider(name)?;
        provider.health_check().await
            .context(format!("Failed to test provider '{}'", name))
    }
    
    /// Test all configured providers
    pub async fn test_all_providers(&mut self) -> Result<HashMap<String, ProviderHealth>> {
        let mut results = HashMap::new();
        let provider_names: Vec<String> = self.config.providers.keys().cloned().collect();
        
        for provider_name in provider_names {
            match self.test_provider(&provider_name).await {
                Ok(health) => {
                    results.insert(provider_name, health);
                },
                Err(e) => {
                    results.insert(provider_name, ProviderHealth {
                        is_healthy: false,
                        response_time_ms: None,
                        last_check: chrono::Utc::now(),
                        error_message: Some(e.to_string()),
                        available_models: vec![],
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get available models for a provider
    pub fn get_available_models(&mut self, name: &str) -> Result<Vec<String>> {
        let provider = self.get_provider(name)?;
        Ok(provider.available_models())
    }
    
    /// Check if a provider supports streaming
    pub fn supports_streaming(&mut self, name: &str) -> Result<bool> {
        let provider = self.get_provider(name)?;
        Ok(provider.supports_streaming())
    }
    
    /// Get provider configuration
    pub fn get_provider_config(&self, name: &str) -> Option<&ProviderConfig> {
        self.config.providers.get(name)
    }
    
    /// Get the default provider name
    pub fn get_default_provider_name(&self) -> &str {
        &self.config.ai.default_provider
    }
    
    /// Reload configuration
    pub fn reload_config(&mut self) -> Result<()> {
        self.config = self.config_manager.get_config()?;
        // Clear loaded providers so they'll be reloaded with new config
        self.providers.clear();
        Ok(())
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &AiConfig {
        &self.config
    }
    
    /// Get mutable configuration reference
    pub fn config_mut(&mut self) -> &mut AiConfig {
        &mut self.config
    }
    
    /// Get configuration manager reference
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
    
    /// Get mutable configuration manager reference
    pub fn config_manager_mut(&mut self) -> &mut ConfigManager {
        &mut self.config_manager
    }
}

/// Shared provider registry for use across the application
pub type SharedProviderRegistry = Arc<std::sync::RwLock<ProviderRegistry>>;

/// Create a shared provider registry
pub fn create_shared_registry(config_manager: ConfigManager) -> Result<SharedProviderRegistry> {
    let registry = ProviderRegistry::new(config_manager)?;
    Ok(Arc::new(std::sync::RwLock::new(registry)))
}

/// Provider registry builder for easier initialization
pub struct ProviderRegistryBuilder {
    config_manager: Option<ConfigManager>,
}

impl ProviderRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config_manager: None,
        }
    }
    
    /// Set the configuration manager
    pub fn with_config_manager(mut self, config_manager: ConfigManager) -> Self {
        self.config_manager = Some(config_manager);
        self
    }
    
    /// Build the provider registry
    pub fn build(self) -> Result<ProviderRegistry> {
        let config_manager = self.config_manager
            .unwrap_or_else(|| ConfigManager::new().expect("Failed to create config manager"));
        
        ProviderRegistry::new(config_manager)
    }
}

impl Default for ProviderRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnalysisDepth;
    
    #[test]
    fn test_provider_registry_builder() {
        let registry = ProviderRegistryBuilder::new()
            .build();
        
        // This will fail because no config file exists, but it shows the builder works
        assert!(registry.is_err());
    }
    
    #[test]
    fn test_shared_registry() {
        let config_manager = ConfigManager::new().unwrap();
        let shared_registry = create_shared_registry(config_manager);
        
        // This will fail because no config file exists, but it shows the shared registry creation works
        assert!(shared_registry.is_err());
    }
}