//! Python Bridge Implementation
//! 
//! Provides integration with external Python ML/AI services through
//! subprocess communication and JSON-based API.

use crate::ai::interface::*;
use crate::model::LogEntry;
use serde_json::json;
use std::process::{Command, Stdio};
use std::io::Write;
use std::time::Duration;
use std::thread;
use anyhow::{Context, Result};

/// Python bridge for AI/ML integration
pub struct PythonBridge {
    config: AiProviderConfig,
    python_path: String,
    script_path: String,
    timeout: Duration,
}

impl PythonBridge {
    /// Create a new Python bridge
    pub fn new(config: AiProviderConfig) -> Self {
        let python_path = config.custom_params.get("python_path")
            .cloned()
            .unwrap_or_else(|| "python3".to_string());
        
        let script_path = config.custom_params.get("script_path")
            .cloned()
            .unwrap_or_else(|| "loglens_ai.py".to_string());
        
        let timeout = Duration::from_secs(config.timeout_seconds);
        
        Self {
            config,
            python_path,
            script_path,
            timeout,
        }
    }
    
    /// Execute Python script with JSON input
    fn execute_python_script(&self, operation: &str, input_data: serde_json::Value) -> Result<serde_json::Value> {
        let mut cmd = Command::new(&self.python_path);
        cmd.arg(&self.script_path)
           .arg(operation)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .context("Failed to start Python process")?;
        
        // Send input data to Python script
        if let Some(stdin) = child.stdin.as_mut() {
            let input_json = serde_json::to_string(&input_data)
                .context("Failed to serialize input data")?;
            stdin.write_all(input_json.as_bytes())
                .context("Failed to write to Python stdin")?;
            stdin.write_all(b"\n")
                .context("Failed to write newline to Python stdin")?;
        }
        
        // Wait for completion with timeout
        let output = thread::spawn(move || child.wait_with_output()).join()
            .map_err(|_| anyhow::anyhow!("Thread panic while waiting for Python process"))?
            .context("Failed to get Python process output")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Python script failed: {}", stderr));
        }
        
        // Parse output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&output_str)
            .context("Failed to parse Python output as JSON")?;
        
        // Check for errors in the response
        if let Some(error) = result.get("error") {
            return Err(anyhow::anyhow!("Python script returned error: {}", error));
        }
        
        Ok(result)
    }
    
    /// Convert log entries to JSON format for Python
    fn entries_to_json(&self, entries: &[LogEntry]) -> serde_json::Value {
        let entries_json: Vec<serde_json::Value> = entries.iter().map(|entry| {
            json!({
                "timestamp": entry.timestamp.to_rfc3339(),
                "level": format!("{:?}", entry.level),
                "message": entry.message,
                "fields": entry.fields,
                "raw_line": entry.raw_line
            })
        }).collect();
        
        json!({ "entries": entries_json })
    }
}

impl AiProvider for PythonBridge {
    fn initialize(&mut self, config: AiProviderConfig) -> Result<(), AiError> {
        self.config = config;
        
        // Test Python bridge connectivity
        let test_input = json!({ "test": true });
        match self.execute_python_script("health_check", test_input) {
            Ok(_) => Ok(()),
            Err(e) => Err(AiError::Configuration(format!("Failed to initialize Python bridge: {}", e))),
        }
    }
    
    fn detect_anomalies(&self, entries: &[LogEntry], request: AnomalyDetectionRequest) -> Result<AnomalyDetectionResponse, AiError> {
        let input_data = self.entries_to_json(entries);
        let mut request_data = serde_json::to_value(request)
            .map_err(|e| AiError::InvalidInput(format!("Failed to serialize request: {}", e)))?;
        
        // Merge entries and request
        if let Some(obj) = request_data.as_object_mut() {
            obj.insert("entries".to_string(), input_data["entries"].clone());
        }
        
        match self.execute_python_script("detect_anomalies", request_data) {
            Ok(result) => {
                serde_json::from_value(result)
                    .map_err(|e| AiError::Processing(format!("Failed to parse anomaly detection response: {}", e)))
            }
            Err(e) => Err(AiError::Processing(format!("Anomaly detection failed: {}", e))),
        }
    }
    
    fn cluster_patterns(&self, entries: &[LogEntry], request: PatternClusteringRequest) -> Result<PatternClusteringResponse, AiError> {
        let input_data = self.entries_to_json(entries);
        let mut request_data = serde_json::to_value(request)
            .map_err(|e| AiError::InvalidInput(format!("Failed to serialize request: {}", e)))?;
        
        // Merge entries and request
        if let Some(obj) = request_data.as_object_mut() {
            obj.insert("entries".to_string(), input_data["entries"].clone());
        }
        
        match self.execute_python_script("cluster_patterns", request_data) {
            Ok(result) => {
                serde_json::from_value(result)
                    .map_err(|e| AiError::Processing(format!("Failed to parse pattern clustering response: {}", e)))
            }
            Err(e) => Err(AiError::Processing(format!("Pattern clustering failed: {}", e))),
        }
    }
    
    fn analyze_text(&self, entries: &[LogEntry], request: TextAnalysisRequest) -> Result<TextAnalysisResponse, AiError> {
        let input_data = self.entries_to_json(entries);
        let mut request_data = serde_json::to_value(request)
            .map_err(|e| AiError::InvalidInput(format!("Failed to serialize request: {}", e)))?;
        
        // Merge entries and request
        if let Some(obj) = request_data.as_object_mut() {
            obj.insert("entries".to_string(), input_data["entries"].clone());
        }
        
        match self.execute_python_script("analyze_text", request_data) {
            Ok(result) => {
                serde_json::from_value(result)
                    .map_err(|e| AiError::Processing(format!("Failed to parse text analysis response: {}", e)))
            }
            Err(e) => Err(AiError::Processing(format!("Text analysis failed: {}", e))),
        }
    }
    
    fn analyze_time_series(&self, entries: &[LogEntry], request: TimeSeriesRequest) -> Result<TimeSeriesResponse, AiError> {
        let input_data = self.entries_to_json(entries);
        let mut request_data = serde_json::to_value(request)
            .map_err(|e| AiError::InvalidInput(format!("Failed to serialize request: {}", e)))?;
        
        // Merge entries and request
        if let Some(obj) = request_data.as_object_mut() {
            obj.insert("entries".to_string(), input_data["entries"].clone());
        }
        
        match self.execute_python_script("analyze_time_series", request_data) {
            Ok(result) => {
                serde_json::from_value(result)
                    .map_err(|e| AiError::Processing(format!("Failed to parse time series response: {}", e)))
            }
            Err(e) => Err(AiError::Processing(format!("Time series analysis failed: {}", e))),
        }
    }
    
    fn get_capabilities(&self) -> AiProviderCapabilities {
        AiProviderCapabilities {
            anomaly_detection: true,
            pattern_clustering: true,
            text_analysis: true,
            time_series_analysis: true,
            classification: true,
            embedding_generation: true,
            real_time_processing: true,
            batch_processing: true,
        }
    }
    
    fn health_check(&self) -> Result<AiProviderHealth, AiError> {
        let test_input = json!({ "test": true });
        
        let start_time = std::time::Instant::now();
        match self.execute_python_script("health_check", test_input) {
            Ok(result) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                let available_models = result.get("available_models")
                    .and_then(|models| models.as_array())
                    .map(|models| models.iter()
                        .filter_map(|model| model.as_str().map(|s| s.to_string()))
                        .collect())
                    .unwrap_or_else(|| vec!["default".to_string()]);
                
                Ok(AiProviderHealth {
                    is_healthy: true,
                    response_time_ms: Some(response_time_ms),
                    last_check: chrono::Utc::now(),
                    error_message: None,
                    available_models,
                })
            }
            Err(e) => Ok(AiProviderHealth {
                is_healthy: false,
                response_time_ms: None,
                last_check: chrono::Utc::now(),
                error_message: Some(format!("Health check failed: {}", e)),
                available_models: vec![],
            }),
        }
    }
}