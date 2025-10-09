use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: ProviderConfig,
    pub defaults: DefaultConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub openrouter: Option<ProviderSettings>,
    pub openai: Option<ProviderSettings>,
    pub claude: Option<ProviderSettings>,
    pub gemini: Option<ProviderSettings>,
    pub anthropic: Option<ProviderSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub model: Option<String>,
    pub timeout: Option<u64>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub provider: Option<String>,
    pub log_level: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            providers: ProviderConfig {
                openrouter: Some(ProviderSettings {
                    model: Some("deepseek/deepseek-chat-v3.1:free".to_string()),
                    timeout: Some(120), // Increased from 30 to 120 seconds for large files
                    max_tokens: Some(2000), // Increased for better analysis
                    temperature: Some(0.1),
                    api_key: None,
                }),
                openai: Some(ProviderSettings {
                    model: Some("gpt-5-nano".to_string()),
                    timeout: Some(120), // Increased from 30 to 120 seconds for large files
                    max_tokens: Some(2000), // Increased for better analysis
                    temperature: Some(0.1),
                    api_key: None,
                }),
                claude: Some(ProviderSettings {
                    model: Some("claude-4-haiku".to_string()),
                    timeout: Some(120), // Increased from 30 to 120 seconds for large files
                    max_tokens: Some(2000), // Increased for better analysis
                    temperature: Some(0.1),
                    api_key: None,
                }),
                gemini: Some(ProviderSettings {
                    model: Some("gemini-2.5-flash".to_string()),
                    timeout: Some(120), // Increased from 30 to 120 seconds for large files
                    max_tokens: Some(2000), // Increased for better analysis
                    temperature: Some(0.1),
                    api_key: None,
                }),
                anthropic: None,
            },
            defaults: DefaultConfig {
                provider: Some("openrouter".to_string()),
                log_level: Some("ERROR".to_string()),
            },
        }
    }
}

impl Config {
    /// Get the unified LogLens data directory
    pub fn get_data_dir() -> PathBuf {
        crate::db_path::get_data_dir()
    }

    /// Get the unified database path
    pub fn get_database_path() -> PathBuf {
        crate::db_path::get_database_path()
    }

    pub fn load() -> Result<Self> {
        // Try to load from config file first
        if let Some(config_path) = Self::get_config_path() {
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                
                // Try to parse as our format, ignore if it fails
                if let Ok(mut config) = toml::from_str::<Config>(&content) {
                    // Merge with defaults for any missing values
                    let default = Config::default();
                    config.merge_with_defaults(&default);
                    return Ok(config);
                }
            }
        }
        // Return default config if no file found or parse failed
        Ok(Config::default())
    }
    
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        // Priority: environment variable > config file > None
        if let Ok(key) = env::var(format!("{}_API_KEY", provider.to_uppercase())) {
            return Some(key);
        }
        
        match provider.to_lowercase().as_str() {
            "openrouter" => self.providers.openrouter.as_ref().and_then(|p| p.api_key.clone()),
            "openai" => self.providers.openai.as_ref().and_then(|p| p.api_key.clone()),
            "claude" | "anthropic" => {
                self.providers.claude.as_ref()
                    .or(self.providers.anthropic.as_ref())
                    .and_then(|p| p.api_key.clone())
            },
            "gemini" => self.providers.gemini.as_ref().and_then(|p| p.api_key.clone()),
            _ => None,
        }
    }
    
    pub fn get_provider_settings(&self, provider: &str) -> Option<&ProviderSettings> {
        match provider.to_lowercase().as_str() {
            "openrouter" => self.providers.openrouter.as_ref(),
            "openai" => self.providers.openai.as_ref(),
            "claude" | "anthropic" => {
                self.providers.claude.as_ref()
                    .or(self.providers.anthropic.as_ref())
            },
            "gemini" => self.providers.gemini.as_ref(),
            _ => None,
        }
    }
    
    pub fn get_default_provider(&self) -> String {
        self.defaults.provider.as_deref()
            .unwrap_or("openrouter")
            .to_string()
    }
    
    pub fn get_default_log_level(&self) -> String {
        self.defaults.log_level.as_deref()
            .unwrap_or("ERROR")
            .to_string()
    }
    
    fn get_config_path() -> Option<PathBuf> {
        // Check for project-level config first
        if let Ok(current_dir) = env::current_dir() {
            let project_config = current_dir.join(".loglens.toml");
            if project_config.exists() {
                return Some(project_config);
            }
        }
        
        // Check for user-level config
        if let Some(home_dir) = dirs::home_dir() {
            let user_config = home_dir.join(".config").join("loglens").join("config.toml");
            if user_config.exists() {
                return Some(user_config);
            }
        }
        
        None
    }
    
    fn merge_with_defaults(&mut self, defaults: &Config) {
        // Merge provider settings
        if self.providers.openrouter.is_none() {
            self.providers.openrouter = defaults.providers.openrouter.clone();
        }
        if self.providers.openai.is_none() {
            self.providers.openai = defaults.providers.openai.clone();
        }
        if self.providers.claude.is_none() {
            self.providers.claude = defaults.providers.claude.clone();
        }
        if self.providers.gemini.is_none() {
            self.providers.gemini = defaults.providers.gemini.clone();
        }
        
        // Merge defaults
        if self.defaults.provider.is_none() {
            self.defaults.provider = defaults.defaults.provider.clone();
        }
        if self.defaults.log_level.is_none() {
            self.defaults.log_level = defaults.defaults.log_level.clone();
        }
    }
    
    pub fn save(&self) -> Result<()> {
        // Determine where to save the config
        let config_path = Self::get_config_path()
            .unwrap_or_else(|| {
                // Default to user-level config if none exists
                if let Some(home_dir) = dirs::home_dir() {
                    home_dir.join(".config").join("loglens").join("config.toml")
                } else {
                    PathBuf::from(".loglens.toml")
                }
            });
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize config to TOML
        let toml_content = toml::to_string_pretty(self)?;
        
        // Write to file
        fs::write(&config_path, toml_content)?;
        
        Ok(())
    }
    
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize config to TOML
        let toml_content = toml::to_string_pretty(self)?;
        
        // Write to file
        fs::write(path, toml_content)?;
        
        Ok(())
    }
    
    pub fn set_api_key(&mut self, provider: &str, api_key: String) {
        match provider.to_lowercase().as_str() {
            "openrouter" => {
                if let Some(ref mut settings) = self.providers.openrouter {
                    settings.api_key = Some(api_key);
                } else {
                    self.providers.openrouter = Some(ProviderSettings {
                        model: Some("openai/gpt-3.5-turbo".to_string()),
                        timeout: Some(120),
                        max_tokens: Some(2000),
                        temperature: Some(0.1),
                        api_key: Some(api_key),
                    });
                }
            },
            "openai" => {
                if let Some(ref mut settings) = self.providers.openai {
                    settings.api_key = Some(api_key);
                } else {
                    self.providers.openai = Some(ProviderSettings {
                        model: Some("gpt-3.5-turbo".to_string()),
                        timeout: Some(120),
                        max_tokens: Some(2000),
                        temperature: Some(0.1),
                        api_key: Some(api_key),
                    });
                }
            },
            "claude" => {
                if let Some(ref mut settings) = self.providers.claude {
                    settings.api_key = Some(api_key);
                } else {
                    self.providers.claude = Some(ProviderSettings {
                        model: Some("claude-3-sonnet-20240229".to_string()),
                        timeout: Some(120),
                        max_tokens: Some(2000),
                        temperature: Some(0.1),
                        api_key: Some(api_key),
                    });
                }
            },
            "gemini" => {
                if let Some(ref mut settings) = self.providers.gemini {
                    settings.api_key = Some(api_key);
                } else {
                    self.providers.gemini = Some(ProviderSettings {
                        model: Some("gemini-pro".to_string()),
                        timeout: Some(120),
                        max_tokens: Some(2000),
                        temperature: Some(0.1),
                        api_key: Some(api_key),
                    });
                }
            },
            _ => {}
        }
    }
    
    pub fn set_default_provider(&mut self, provider: String) {
        self.defaults.provider = Some(provider);
    }
    
    pub fn set_default_log_level(&mut self, level: String) {
        self.defaults.log_level = Some(level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.get_default_provider(), "openrouter");
        assert_eq!(config.get_default_log_level(), "ERROR");
    }
    
    #[test]
    fn test_provider_settings() {
        let config = Config::default();
        let openrouter_settings = config.get_provider_settings("openrouter");
        assert!(openrouter_settings.is_some());
        assert_eq!(openrouter_settings.unwrap().model, Some("deepseek/deepseek-chat-v3.1:free".to_string()));
    }
}