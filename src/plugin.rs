//! Plugin System Implementation
//! 
//! Provides a comprehensive plugin architecture for extending LogLens
//! with custom parsers, filters, and analytics components.

use crate::model::LogEntry;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use thiserror::Error;

/// Plugin errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin loading failed: {0}")]
    LoadFailed(String),
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Plugin version incompatible: {0}")]
    VersionIncompatible(String),
    #[error("Plugin dependencies not satisfied: {0}")]
    DependenciesNotSatisfied(String),
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub config: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
    pub max_memory_mb: Option<u64>,
    pub timeout_seconds: Option<u64>,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub entry_point: String,
}

/// Plugin types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Parser,
    Filter,
    Analytics,
    Output,
    Input,
    Custom(String),
}

/// Plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    fn cleanup(&mut self) -> Result<(), PluginError>;
    fn get_info(&self) -> PluginInfo;
}

/// Parser plugin trait
pub trait ParserPlugin: Plugin {
    fn can_parse(&self, sample: &str) -> f64;
    fn parse(&self, line: &str) -> Result<LogEntry, PluginError>;
    fn get_supported_formats(&self) -> Vec<String>;
}

/// Filter plugin trait
pub trait FilterPlugin: Plugin {
    fn apply(&self, entry: &LogEntry) -> bool;
    fn description(&self) -> &str;
    fn get_parameters(&self) -> Vec<PluginParameter>;
}

/// Analytics plugin trait
pub trait AnalyticsPlugin: Plugin {
    fn analyze(&self, entries: &[LogEntry]) -> Result<AnalyticsResult, PluginError>;
    fn get_visualization_data(&self, result: &AnalyticsResult) -> Option<serde_json::Value>;
}

/// Output plugin trait
pub trait OutputPlugin: Plugin {
    fn write<T: std::io::Write>(&self, entries: &[LogEntry], output: &mut T) -> Result<(), PluginError>;
    fn get_supported_formats(&self) -> Vec<String>;
}

/// Input plugin trait
pub trait InputPlugin: Plugin {
    fn can_handle(&self, source: &str) -> bool;
    fn read(&self, source: &str) -> Result<Box<dyn std::io::Read>, PluginError>;
    fn get_supported_sources(&self) -> Vec<String>;
}

/// Plugin parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: String,
    pub default_value: Option<serde_json::Value>,
    pub required: bool,
    pub validation: Option<ParameterValidation>,
}

/// Parameter validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// Analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResult {
    pub summary: String,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub processing_time_ms: u64,
}

/// Plugin registry
pub struct PluginRegistry {
    plugins: HashMap<String, PluginInstance>,
    loaded_libraries: Vec<Library>,
    config: PluginManagerConfig,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(config: PluginManagerConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            loaded_libraries: Vec::new(),
            config,
        }
    }
    
    /// Load a plugin from file
    pub fn load_plugin(&mut self, plugin_config: &PluginConfig) -> Result<(), PluginError> {
        let library_path = &plugin_config.path;
        
        // Load the dynamic library
        let library = unsafe { Library::new(library_path) }
            .map_err(|e| PluginError::LoadFailed(format!("Failed to load library: {}", e)))?;
        
        // Get the plugin creation function
        let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn Plugin> = unsafe {
            library.get(b"create_plugin")
                .map_err(|e| PluginError::LoadFailed(format!("Failed to find create_plugin symbol: {}", e)))?
        };
        
        // Create the plugin instance
        let plugin_ptr = unsafe { create_plugin() };
        let plugin = unsafe { Box::from_raw(plugin_ptr) };
        
        // Initialize the plugin
        let mut plugin_instance = PluginInstance {
            plugin: Arc::new(Mutex::new(plugin)),
            config: plugin_config.clone(),
            state: PluginState::Loaded,
            load_time: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
            usage_count: 0,
        };
        
        // Initialize the plugin
        {
            let mut plugin = plugin_instance.plugin.lock().unwrap();
            plugin.initialize(plugin_config)?;
        }
        
        plugin_instance.state = PluginState::Active;
        
        // Add to registry
        self.plugins.insert(plugin_config.name.clone(), plugin_instance);
        self.loaded_libraries.push(library);
        
        Ok(())
    }
    
    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        let mut instance = self.plugins.remove(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
        
        // Cleanup the plugin
        {
            let mut plugin = instance.plugin.lock().unwrap();
            plugin.cleanup()?;
        }
        
        instance.state = PluginState::Unloaded;
        
        Ok(())
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, plugin_name: &str) -> Option<&PluginInstance> {
        self.plugins.get(plugin_name)
    }
    
    /// Get a mutable plugin by name
    pub fn get_plugin_mut(&mut self, plugin_name: &str) -> Option<&mut PluginInstance> {
        self.plugins.get_mut(plugin_name)
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginInstance> {
        self.plugins.values().collect()
    }
    
    /// Get plugins by type
    pub fn get_plugins_by_type(&self, plugin_type: PluginType) -> Vec<&PluginInstance> {
        self.plugins.values()
            .filter(|instance| {
                if let Ok(plugin) = instance.plugin.lock() {
                    plugin.plugin_type() == plugin_type
                } else {
                    false
                }
            })
            .collect()
    }
    
    /// Reload a plugin
    pub fn reload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        let config = self.get_plugin(plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?
            .config
            .clone();
        
        self.unload_plugin(plugin_name)?;
        self.load_plugin(&config)?;
        
        Ok(())
    }
    
    /// Check plugin dependencies
    pub fn check_dependencies(&self, plugin_config: &PluginConfig) -> Result<(), PluginError> {
        for dependency in &plugin_config.dependencies {
            if !self.plugins.contains_key(dependency) {
                return Err(PluginError::DependenciesNotSatisfied(
                    format!("Dependency '{}' not found", dependency)
                ));
            }
        }
        Ok(())
    }
    
    /// Get plugin statistics
    pub fn get_plugin_stats(&self) -> PluginStats {
        let total_plugins = self.plugins.len();
        let active_plugins = self.plugins.values()
            .filter(|instance| instance.state == PluginState::Active)
            .count();
        
        let total_usage: u64 = self.plugins.values()
            .map(|instance| instance.usage_count)
            .sum();
        
        PluginStats {
            total_plugins,
            active_plugins,
            total_usage,
            loaded_libraries: self.loaded_libraries.len(),
        }
    }
}

/// Plugin instance
pub struct PluginInstance {
    pub plugin: Arc<Mutex<Box<dyn Plugin>>>,
    pub config: PluginConfig,
    pub state: PluginState,
    pub load_time: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

impl std::fmt::Debug for PluginInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginInstance")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("load_time", &self.load_time)
            .field("last_used", &self.last_used)
            .field("usage_count", &self.usage_count)
            .field("plugin", &"<Plugin>")
            .finish()
    }
}

/// Plugin states
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Initializing,
    Active,
    Deactivating,
    Error(String),
}

/// Plugin manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    pub plugin_directory: PathBuf,
    pub auto_load: bool,
    pub enable_sandboxing: bool,
    pub max_memory_per_plugin: u64,
    pub plugin_timeout_seconds: u64,
    pub enable_hot_reload: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            plugin_directory: PathBuf::from("./plugins"),
            auto_load: false,
            enable_sandboxing: false,
            max_memory_per_plugin: 512, // 512MB
            plugin_timeout_seconds: 30,
            enable_hot_reload: false,
        }
    }
}

/// Plugin statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub total_usage: u64,
    pub loaded_libraries: usize,
}

/// Plugin manager
pub struct PluginManager {
    registry: PluginRegistry,
    config: PluginManagerConfig,
    watcher: Option<FileSystemWatcher>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(config: PluginManagerConfig) -> Self {
        let registry = PluginRegistry::new(config.clone());
        
        Self {
            registry,
            config,
            watcher: None,
        }
    }
    
    /// Initialize the plugin manager
    pub fn initialize(&mut self) -> Result<(), PluginError> {
        // Create plugin directory if it doesn't exist
        if !self.config.plugin_directory.exists() {
            std::fs::create_dir_all(&self.config.plugin_directory)
                .map_err(|e| PluginError::InitializationFailed(
                    format!("Failed to create plugin directory: {}", e)
                ))?;
        }
        
        // Auto-load plugins if enabled
        if self.config.auto_load {
            self.load_all_plugins()?;
        }
        
        // Setup file system watcher if hot reload is enabled
        if self.config.enable_hot_reload {
            self.setup_file_watcher()?;
        }
        
        Ok(())
    }
    
    /// Load all plugins from the plugin directory
    pub fn load_all_plugins(&mut self) -> Result<(), PluginError> {
        let entries = std::fs::read_dir(&self.config.plugin_directory)
            .map_err(|e| PluginError::InitializationFailed(
                format!("Failed to read plugin directory: {}", e)
            ))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| PluginError::InitializationFailed(
                format!("Failed to read directory entry: {}", e)
            ))?;
            
            let path = entry.path();
            
            // Only load .so files on Unix or .dll files on Windows
            if path.extension().map_or(false, |ext| {
                ext == "so" || ext == "dll" || ext == "dylib"
            }) {
                let plugin_config = PluginConfig {
                    name: path.file_stem().unwrap().to_string_lossy().to_string(),
                    version: "1.0.0".to_string(),
                    path: path.clone(),
                    enabled: true,
                    config: HashMap::new(),
                    dependencies: vec![],
                    max_memory_mb: Some(self.config.max_memory_per_plugin),
                    timeout_seconds: Some(self.config.plugin_timeout_seconds),
                };
                
                if let Err(e) = self.registry.load_plugin(&plugin_config) {
                    eprintln!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Setup file system watcher for hot reload
    fn setup_file_watcher(&mut self) -> Result<(), PluginError> {
        // This would implement file system watching for hot reload
        // For now, we'll just create a placeholder
        self.watcher = Some(FileSystemWatcher::new());
        Ok(())
    }
    
    /// Get a reference to the plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
    
    /// Get a mutable reference to the plugin registry
    pub fn registry_mut(&mut self) -> &mut PluginRegistry {
        &mut self.registry
    }
    
    /// Get plugin manager configuration
    pub fn config(&self) -> &PluginManagerConfig {
        &self.config
    }
    
    /// Shutdown the plugin manager
    pub fn shutdown(&mut self) -> Result<(), PluginError> {
        // Unload all plugins
        let plugin_names: Vec<String> = self.registry.plugins.keys().cloned().collect();
        
        for plugin_name in plugin_names {
            if let Err(e) = self.registry.unload_plugin(&plugin_name) {
                eprintln!("Failed to unload plugin {}: {}", plugin_name, e);
            }
        }
        
        Ok(())
    }
}

/// Placeholder for file system watcher
struct FileSystemWatcher;

impl FileSystemWatcher {
    fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_config_creation() {
        let config = PluginConfig {
            name: "test_plugin".to_string(),
            version: "1.0.0".to_string(),
            path: PathBuf::from("./test_plugin.so"),
            enabled: true,
            config: HashMap::new(),
            dependencies: vec![],
            max_memory_mb: None,
            timeout_seconds: None,
        };
        
        assert_eq!(config.name, "test_plugin");
        assert_eq!(config.version, "1.0.0");
        assert!(config.enabled);
    }
    
    #[test]
    fn test_plugin_manager_config_default() {
        let config = PluginManagerConfig::default();
        
        assert_eq!(config.plugin_directory, PathBuf::from("./plugins"));
        assert!(!config.auto_load);
        assert!(!config.enable_sandboxing);
        assert_eq!(config.max_memory_per_plugin, 512);
        assert_eq!(config.plugin_timeout_seconds, 30);
    }
    
    #[test]
    fn test_plugin_stats() {
        let stats = PluginStats {
            total_plugins: 5,
            active_plugins: 3,
            total_usage: 100,
            loaded_libraries: 4,
        };
        
        assert_eq!(stats.total_plugins, 5);
        assert_eq!(stats.active_plugins, 3);
        assert_eq!(stats.total_usage, 100);
        assert_eq!(stats.loaded_libraries, 4);
    }
}