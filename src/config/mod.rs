//! Configuration Module
//! 
//! Handles loading, managing, and validating LogLens configuration
//! including AI provider settings and analysis preferences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use toml;
use dirs;

/// Main AI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub ai: AiSection,
    pub providers: HashMap<String, ProviderConfig>,
    pub process_monitoring: Option<ProcessMonitoringConfig>,
}

/// AI-specific configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSection {
    pub default_provider: String,
    pub analysis_depth: AnalysisDepth,
    pub auto_analyze: bool,
    pub context_window: usize,
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_rpm: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_name: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub additional_params: HashMap<String, String>,
}

/// Process monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMonitoringConfig {
    pub enabled: bool,
    pub auto_start_analysis: bool,
    pub analysis_trigger_patterns: Vec<String>,
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub default_analysis: Option<DefaultAnalysisConfig>,
}

/// Default analysis configuration for process monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultAnalysisConfig {
    pub provider: String,
    pub model: String,
    pub depth: AnalysisDepth,
}

/// Analysis depth levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,
    Detailed,
    Comprehensive,
}

impl Default for AnalysisDepth {
    fn default() -> Self {
        AnalysisDepth::Detailed
    }
}

impl std::str::FromStr for AnalysisDepth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(AnalysisDepth::Basic),
            "detailed" => Ok(AnalysisDepth::Detailed),
            "comprehensive" => Ok(AnalysisDepth::Comprehensive),
            _ => Err(format!("Invalid analysis depth: {}. Must be 'basic', 'detailed', or 'comprehensive'", s)),
        }
    }
}

impl std::fmt::Display for AnalysisDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisDepth::Basic => write!(f, "basic"),
            AnalysisDepth::Detailed => write!(f, "detailed"),
            AnalysisDepth::Comprehensive => write!(f, "comprehensive"),
        }
    }
}

/// Configuration manager
#[derive(Debug)]
pub struct ConfigManager {
    config_path: PathBuf,
    config: Option<AiConfig>,
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            config_path: self.config_path.clone(),
            config: self.config.clone(),
        }
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("loglens");
        
        let config_path = config_dir.join("config.toml");
        
        Ok(Self {
            config_path,
            config: None,
        })
    }
    
    /// Get the configuration file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
    
    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }
    
    /// Load configuration from file
    pub fn load_config(&mut self) -> Result<AiConfig> {
        if !self.config_exists() {
            return Err(anyhow::anyhow!("Configuration file not found at {:?}", self.config_path));
        }
        
        let content = std::fs::read_to_string(&self.config_path)
            .context("Failed to read configuration file")?;
        
        let config: AiConfig = toml::from_str(&content)
            .context("Failed to parse configuration file")?;
        
        self.config = Some(config.clone());
        Ok(config)
    }
    
    /// Get configuration, loading if necessary
    pub fn get_config(&mut self) -> Result<AiConfig> {
        if self.config.is_none() {
            self.load_config()?;
        }
        self.config.clone().ok_or_else(|| anyhow::anyhow!("No configuration loaded"))
    }
    
    /// Create default configuration file
    pub fn create_default_config(&self) -> Result<()> {
        if self.config_exists() {
            return Err(anyhow::anyhow!("Configuration file already exists at {:?}", self.config_path));
        }
        
        let default_config = Self::generate_default_config();
        
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default configuration")?;
        
        std::fs::write(&self.config_path, toml_string)
            .context("Failed to write default configuration")?;
        
        println!("Created default configuration at {:?}", self.config_path);
        Ok(())
    }
    
    /// Generate default configuration
    pub fn generate_default_config() -> AiConfig {
        let mut providers = HashMap::new();
        
        // OpenRouter configuration (primary for testing)
        providers.insert("openrouter".to_string(), ProviderConfig {
            api_key: None, // User must add their API key
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-3.5-sonnet".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rpm: Some(60),
            access_key: None,
            secret_key: None,
            region: None,
            endpoint: None,
            api_version: None,
            deployment_name: None,
            additional_params: HashMap::new(),
        });
        
        // OpenAI configuration
        providers.insert("openai".to_string(), ProviderConfig {
            api_key: None,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4-turbo".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rpm: None,
            access_key: None,
            secret_key: None,
            region: None,
            endpoint: None,
            api_version: None,
            deployment_name: None,
            additional_params: HashMap::new(),
        });
        
        // Anthropic configuration
        providers.insert("anthropic".to_string(), ProviderConfig {
            api_key: None,
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-3-opus-20240229".to_string(),
            timeout_seconds: 45,
            max_retries: 2,
            rate_limit_rpm: None,
            access_key: None,
            secret_key: None,
            region: None,
            endpoint: None,
            api_version: None,
            deployment_name: None,
            additional_params: HashMap::new(),
        });
        
        // Gemini configuration
        providers.insert("gemini".to_string(), ProviderConfig {
            api_key: None,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            model: "gemini-1.5-pro".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
            rate_limit_rpm: None,
            access_key: None,
            secret_key: None,
            region: None,
            endpoint: None,
            api_version: None,
            deployment_name: None,
            additional_params: HashMap::new(),
        });
        
        // Local Ollama configuration
        providers.insert("local".to_string(), ProviderConfig {
            api_key: None,
            base_url: "http://localhost:11434".to_string(),
            model: "llama3:70b".to_string(),
            timeout_seconds: 120,
            max_retries: 1,
            rate_limit_rpm: None,
            access_key: None,
            secret_key: None,
            region: None,
            endpoint: None,
            api_version: None,
            deployment_name: None,
            additional_params: HashMap::new(),
        });
        
        AiConfig {
            ai: AiSection {
                default_provider: "openrouter".to_string(),
                analysis_depth: AnalysisDepth::Detailed,
                auto_analyze: true,
                context_window: 32000,
            },
            providers,
            process_monitoring: Some(ProcessMonitoringConfig {
                enabled: true,
                auto_start_analysis: true,
                analysis_trigger_patterns: vec!["ERROR".to_string(), "FATAL".to_string(), "CRITICAL".to_string()],
                buffer_size: 1000,
                flush_interval_seconds: 5,
                default_analysis: Some(DefaultAnalysisConfig {
                    provider: "openrouter".to_string(),
                    model: "anthropic/claude-3.5-sonnet".to_string(),
                    depth: AnalysisDepth::Detailed,
                }),
            }),
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AiConfig) -> Result<()> {
        let toml_string = toml::to_string_pretty(&config)
            .context("Failed to serialize configuration")?;
        
        std::fs::write(&self.config_path, toml_string)
            .context("Failed to write configuration")?;
        
        self.config = Some(config);
        Ok(())
    }
    
    /// Set default provider
    pub fn set_default_provider(&mut self, provider: &str) -> Result<()> {
        let mut config = self.get_config()?;
        if !config.providers.contains_key(provider) {
            return Err(anyhow::anyhow!("Provider '{}' not found in configuration", provider));
        }
        
        config.ai.default_provider = provider.to_string();
        self.update_config(config)?;
        println!("Set default provider to: {}", provider);
        Ok(())
    }
    
    /// Add or update a provider
    pub fn update_provider(&mut self, name: &str, provider_config: ProviderConfig) -> Result<()> {
        let mut config = self.get_config()?;
        config.providers.insert(name.to_string(), provider_config);
        self.update_config(config)?;
        println!("Updated provider configuration for: {}", name);
        Ok(())
    }
    
    /// Remove a provider
    pub fn remove_provider(&mut self, name: &str) -> Result<()> {
        let mut config = self.get_config()?;
        if !config.providers.contains_key(name) {
            return Err(anyhow::anyhow!("Provider '{}' not found in configuration", name));
        }
        
        config.providers.remove(name);
        
        // If this was the default provider, change to the first available
        if config.ai.default_provider == name {
            if let Some(new_default) = config.providers.keys().next() {
                config.ai.default_provider = new_default.clone();
            } else {
                return Err(anyhow::anyhow!("Cannot remove last provider - at least one provider is required"));
            }
        }
        
        self.update_config(config)?;
        println!("Removed provider: {}", name);
        Ok(())
    }
    
    /// List all configured providers
    pub fn list_providers(&mut self) -> Result<Vec<String>> {
        let config = self.get_config()?;
        Ok(config.providers.keys().cloned().collect())
    }
    
    /// Get provider configuration
    pub fn get_provider_config(&mut self, name: &str) -> Result<ProviderConfig> {
        let config = self.get_config()?;
        config.providers.get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found in configuration", name))
    }
    
    /// Validate configuration
    pub fn validate_config(&mut self) -> Result<Vec<String>> {
        let config = self.get_config()?;
        let mut warnings = Vec::new();
        
        // Check if default provider exists
        if !config.providers.contains_key(&config.ai.default_provider) {
            warnings.push(format!("Default provider '{}' not found in providers", config.ai.default_provider));
        }
        
        // Check each provider configuration
        for (name, provider) in &config.providers {
            if provider.api_key.is_none() && !name.starts_with("local") {
                warnings.push(format!("Provider '{}' has no API key configured", name));
            }
            
            if provider.base_url.is_empty() {
                warnings.push(format!("Provider '{}' has empty base URL", name));
            }
            
            if provider.model.is_empty() {
                warnings.push(format!("Provider '{}' has no model configured", name));
            }
        }
        
        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_analysis_depth_from_str() {
        assert_eq!("basic".parse::<AnalysisDepth>().unwrap(), AnalysisDepth::Basic);
        assert_eq!("detailed".parse::<AnalysisDepth>().unwrap(), AnalysisDepth::Detailed);
        assert_eq!("comprehensive".parse::<AnalysisDepth>().unwrap(), AnalysisDepth::Comprehensive);
        assert!("invalid".parse::<AnalysisDepth>().is_err());
    }

    #[test]
    fn test_analysis_depth_display() {
        assert_eq!(AnalysisDepth::Basic.to_string(), "basic");
        assert_eq!(AnalysisDepth::Detailed.to_string(), "detailed");
        assert_eq!(AnalysisDepth::Comprehensive.to_string(), "comprehensive");
    }

    #[test]
    fn test_config_creation() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let config_content = r#"
[ai]
default_provider = "openrouter"
analysis_depth = "detailed"
auto_analyze = true
context_window = 32000

[providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
model = "anthropic/claude-3.5-sonnet"
timeout_seconds = 30
max_retries = 3
rate_limit_rpm = 60
"#;
        
        temp_file.write_all(config_content.as_bytes())?;
        temp_file.flush()?;
        
        let config: AiConfig = toml::from_str(config_content)?;
        assert_eq!(config.ai.default_provider, "openrouter");
        assert_eq!(config.ai.analysis_depth, AnalysisDepth::Detailed);
        assert!(config.providers.contains_key("openrouter"));
        
        Ok(())
    }
}