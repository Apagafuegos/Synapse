use rmcp::{
    ServerHandler,
    model::{
        ServerInfo, Tool, CallToolRequestParam, CallToolResult, Content,
        ListToolsResult, PaginatedRequestParam
    },
    Error as McpError
};

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

    fn create_loglens() -> Result<crate::LogLens, McpError> {
        crate::LogLens::new().map_err(|e| McpError::internal_error(e.to_string(), None))
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
    ) -> Result<ListToolsResult, McpError> {
        // Create proper JSON schemas for tools
        let analyze_logs_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // logs property
            let mut logs_prop = serde_json::Map::new();
            logs_prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
            logs_prop.insert("description".to_string(), serde_json::Value::String("Array of log lines to analyze".to_string()));
            let mut items = serde_json::Map::new();
            items.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            logs_prop.insert("items".to_string(), serde_json::Value::Object(items));
            properties.insert("logs".to_string(), serde_json::Value::Object(logs_prop));

            // level property
            let mut level_prop = serde_json::Map::new();
            level_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            level_prop.insert("description".to_string(), serde_json::Value::String("Log level to filter by (ERROR, WARN, INFO, DEBUG)".to_string()));
            level_prop.insert("default".to_string(), serde_json::Value::String("ERROR".to_string()));
            properties.insert("level".to_string(), serde_json::Value::Object(level_prop));

            // provider property
            let mut provider_prop = serde_json::Map::new();
            provider_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            provider_prop.insert("description".to_string(), serde_json::Value::String("AI provider to use (openrouter, openai, claude, gemini)".to_string()));
            provider_prop.insert("default".to_string(), serde_json::Value::String("openrouter".to_string()));
            properties.insert("provider".to_string(), serde_json::Value::Object(provider_prop));

            // api_key property (optional)
            let mut api_key_prop = serde_json::Map::new();
            api_key_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            api_key_prop.insert("description".to_string(), serde_json::Value::String("API key for the provider (optional if set in config/env)".to_string()));
            properties.insert("api_key".to_string(), serde_json::Value::Object(api_key_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("logs".to_string())
            ]));

            schema
        });

        let parse_logs_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // logs property
            let mut logs_prop = serde_json::Map::new();
            logs_prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
            logs_prop.insert("description".to_string(), serde_json::Value::String("Array of raw log lines to parse".to_string()));
            let mut items = serde_json::Map::new();
            items.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            logs_prop.insert("items".to_string(), serde_json::Value::Object(items));
            properties.insert("logs".to_string(), serde_json::Value::Object(logs_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("logs".to_string())
            ]));

            schema
        });

        let filter_logs_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // logs property
            let mut logs_prop = serde_json::Map::new();
            logs_prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
            logs_prop.insert("description".to_string(), serde_json::Value::String("Array of log lines to filter".to_string()));
            let mut items = serde_json::Map::new();
            items.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            logs_prop.insert("items".to_string(), serde_json::Value::Object(items));
            properties.insert("logs".to_string(), serde_json::Value::Object(logs_prop));

            // level property
            let mut level_prop = serde_json::Map::new();
            level_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            level_prop.insert("description".to_string(), serde_json::Value::String("Minimum log level to filter by".to_string()));
            properties.insert("level".to_string(), serde_json::Value::Object(level_prop));

            // pattern property (optional)
            let mut pattern_prop = serde_json::Map::new();
            pattern_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            pattern_prop.insert("description".to_string(), serde_json::Value::String("Optional regex pattern to match".to_string()));
            properties.insert("pattern".to_string(), serde_json::Value::Object(pattern_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("logs".to_string()),
                serde_json::Value::String("level".to_string())
            ]));

            schema
        });

        let tools = vec![
            Tool {
                name: "analyze_logs".into(),
                description: Some("Analyze log entries using AI to identify issues and patterns".into()),
                input_schema: analyze_logs_schema,
                annotations: None,
            },
            Tool {
                name: "parse_logs".into(),
                description: Some("Parse raw log text into structured log entries".into()),
                input_schema: parse_logs_schema,
                annotations: None,
            },
            Tool {
                name: "filter_logs".into(),
                description: Some("Filter log entries by level and patterns".into()),
                input_schema: filter_logs_schema,
                annotations: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "analyze_logs" => self.handle_analyze_logs(request.arguments).await,
            "parse_logs" => self.handle_parse_logs(request.arguments).await,
            "filter_logs" => self.handle_filter_logs(request.arguments).await,
            _ => Err(McpError::method_not_found::<rmcp::model::CallToolRequestMethod>())
        }
    }
}

impl LogLensServer {
    async fn handle_analyze_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::invalid_params(format!("Invalid logs array: {}", e), None))?;

        let level = args.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("ERROR");

        let provider = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter");

        let api_key = args.get("api_key")
            .and_then(|v| v.as_str());

        let input_source = "mcp_tool";
        let output_format = crate::OutputFormat::Json;

        let loglens = Self::create_loglens()?;
        match loglens.generate_full_report(
            logs,
            level,
            provider,
            api_key,
            input_source,
            output_format,
        ).await {
            Ok(report) => Ok(CallToolResult::success(vec![Content::text(report)])),
            Err(e) => Err(McpError::internal_error(format!("Analysis failed: {}", e), None))
        }
    }

    async fn handle_parse_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::invalid_params(format!("Invalid logs array: {}", e), None))?;

        let parsed_entries = crate::parse_log_lines(&logs);
        let json_result = serde_json::to_string_pretty(&parsed_entries)
            .map_err(|e| McpError::internal_error(format!("Serialization failed: {}", e), None))?;
        Ok(CallToolResult::success(vec![Content::text(json_result)]))
    }

    async fn handle_filter_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let logs: Vec<String> = serde_json::from_value(
            args.get("logs").cloned().unwrap_or(serde_json::json!([]))
        ).map_err(|e| McpError::invalid_params(format!("Invalid logs array: {}", e), None))?;

        let level = args.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing level parameter".to_string(), None))?;

        let _pattern = args.get("pattern")
            .and_then(|v| v.as_str());

        // Parse logs first
        let parsed_entries = crate::parse_log_lines(&logs);

        // Filter by level
        let filtered_entries = crate::filter_logs_by_level(parsed_entries, level)
            .map_err(|e| McpError::internal_error(format!("Filtering failed: {}", e), None))?;

        let json_result = serde_json::to_string_pretty(&filtered_entries)
            .map_err(|e| McpError::internal_error(format!("Serialization failed: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json_result)]))
    }
}