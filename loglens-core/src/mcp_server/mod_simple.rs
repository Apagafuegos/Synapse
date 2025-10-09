use rmcp::{
    ServerHandler,
    model::{
        ServerInfo, Tool, CallToolRequestParam, CallToolResult, Content,
        ListToolsResult, PaginatedRequestParam
    },
};
use std::path::PathBuf;
use std::env;

use crate::mcp_server::error::{McpError, invalid_params, internal_error, method_not_found};

pub mod async_analysis;
pub mod error;
pub mod types;

#[derive(Debug, Clone)]
pub struct LogLensServer {
    // We'll create LogLens on demand instead of storing it
}

impl LogLensServer {
    pub fn new() -> Self {
        Self {}
    }

    fn create_loglens() -> Result<crate::LogLens, rmcp::Error> {
        crate::LogLens::new().map_err(|e| internal_error(e.to_string()))
    }
}

impl Default for LogLensServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for LogLensServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("AI-powered log analysis tool that can parse, filter, and analyze logs using various AI providers.".into()),
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListToolsResult, rmcp::Error> {
        let mut tools = Vec::new();

        // analyze_logs tool
        let mut analyze_logs_props = serde_json::Map::new();
        let mut logs_prop = serde_json::Map::new();
        logs_prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
        logs_prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
        logs_prop.insert("description".to_string(), serde_json::Value::String("Array of log lines to analyze".to_string()));
        analyze_logs_props.insert("logs".to_string(), serde_json::Value::Object(logs_prop));

        let mut level_prop = serde_json::Map::new();
        level_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        level_prop.insert("enum".to_string(), serde_json::json!(["ERROR", "WARN", "INFO", "DEBUG"]));
        level_prop.insert("description".to_string(), serde_json::Value::String("Minimum log level to analyze".to_string()));
        analyze_logs_props.insert("level".to_string(), serde_json::Value::Object(level_prop));

        let mut provider_prop = serde_json::Map::new();
        provider_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        provider_prop.insert("enum".to_string(), serde_json::json!(["openrouter", "openai", "claude", "gemini"]));
        provider_prop.insert("description".to_string(), serde_json::Value::String("AI provider for analysis (optional if set in config/env)".to_string()));
        analyze_logs_props.insert("provider".to_string(), serde_json::Value::Object(provider_prop));

        let mut api_key_prop = serde_json::Map::new();
        api_key_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        api_key_prop.insert("description".to_string(), serde_json::Value::String("API key for the provider (optional if set in config/env)".to_string()));
        analyze_logs_props.insert("api_key".to_string(), serde_json::Value::Object(api_key_prop));

        tools.push(Tool {
            name: "analyze_logs".to_string(),
            description: Some("Analyze log lines using AI to identify patterns, issues, and insights".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": analyze_logs_props,
                "required": ["logs"]
            }),
        });

        // parse_logs tool
        let mut parse_logs_props = serde_json::Map::new();
        parse_logs_props.insert("logs".to_string(), serde_json::Value::Object({
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
            prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
            prop.insert("description".to_string(), serde_json::Value::String("Array of log lines to parse".to_string()));
            prop
        }));

        tools.push(Tool {
            name: "parse_logs".to_string(),
            description: Some("Parse log lines into structured format with timestamps, levels, and messages".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": parse_logs_props,
                "required": ["logs"]
            }),
        });

        // filter_logs tool
        let mut filter_props = serde_json::Map::new();
        filter_props.insert("logs".to_string(), serde_json::Value::Object({
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
            prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
            prop.insert("description".to_string(), serde_json::Value::String("Array of log lines to filter".to_string()));
            prop
        }));

        filter_props.insert("level".to_string(), serde_json::Value::Object({
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            prop.insert("enum".to_string(), serde_json::json!(["ERROR", "WARN", "INFO", "DEBUG"]));
            prop.insert("description".to_string(), serde_json::Value::String("Minimum log level to include".to_string()));
            prop
        }));

        tools.push(Tool {
            name: "filter_logs".to_string(),
            description: Some("Filter log lines by minimum log level".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": filter_props,
                "required": ["logs", "level"]
            }),
        });

        Ok(ListToolsResult { tools })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        match request.name.as_str() {
            "analyze_logs" => self.handle_analyze_logs(request.arguments).await.map_err(McpError::into),
            "parse_logs" => self.handle_parse_logs(request.arguments).await.map_err(McpError::into),
            "filter_logs" => self.handle_filter_logs(request.arguments).await.map_err(McpError::into),
            _ => Err(method_not_found::<rmcp::model::CallToolRequestMethod>()),
        }
    }
}

impl LogLensServer {
    async fn handle_analyze_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(|| 
            McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        let level = args.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO");

        let provider = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter");

        let api_key = args.get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create LogLens instance
        let mut loglens = Self::create_loglens()
            .map_err(|e| McpError::InternalError(format!("Failed to create LogLens: {}", e)))?;

        // Set API key if provided
        if let Some(key) = api_key {
            loglens = loglens.with_api_key(&key);
        }

        // Analyze logs
        let analysis_result = loglens.analyze_lines(&logs, level, provider)
            .await
            .map_err(|e| McpError::AnalysisFailed(format!("Analysis failed: {}", e)))?;

        // Return results
        let response = serde_json::json!({
            "success": true,
            "analysis": analysis_result,
            "logs_processed": logs.len(),
            "level": level,
            "provider": provider
        });

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&response).unwrap())]))
    }

    async fn handle_parse_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(|| 
            McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        // Create LogLens instance
        let loglens = Self::create_loglens()
            .map_err(|e| McpError::InternalError(format!("Failed to create LogLens: {}", e)))?;

        // Parse logs
        let parsed_logs = loglens.parse_lines(&logs)
            .await
            .map_err(|e| McpError::InternalError(format!("Parsing failed: {}", e)))?;

        // Return results
        let response = serde_json::json!({
            "success": true,
            "parsed_logs": parsed_logs,
            "logs_processed": logs.len()
        });

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&response).unwrap())]))
    }

    async fn handle_filter_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(|| 
            McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        let level = args.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing level parameter".to_string()))?;

        // Create LogLens instance
        let loglens = Self::create_loglens()
            .map_err(|e| McpError::InternalError(format!("Failed to create LogLens: {}", e)))?;

        // Filter logs
        let filtered_logs = loglens.filter_logs(&logs, level)
            .await
            .map_err(|e| McpError::InternalError(format!("Filtering failed: {}", e)))?;

        // Return results
        let response = serde_json::json!({
            "success": true,
            "filtered_logs": filtered_logs,
            "original_count": logs.len(),
            "filtered_count": filtered_logs.len(),
            "level": level
        });

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&response).unwrap())]))
    }
}