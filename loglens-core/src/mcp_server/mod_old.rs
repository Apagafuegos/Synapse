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

#[cfg(feature = "project-management")]
use crate::project::{
    database::create_pool,
    queries::{create_analysis, get_analysis_by_id, query_analyses, get_or_create_project, store_analysis_results, update_analysis_status},
    models::{Analysis, AnalysisResult}
};

#[cfg(feature = "project-management")]
use crate::mcp_server::async_analysis::spawn_analysis_task;

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
        crate::LogLens::new().map_err(|e| rmcp::Error::internal_error(e.to_string(), None))
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

        let add_log_file_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // project_path property
            let mut project_path_prop = serde_json::Map::new();
            project_path_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            project_path_prop.insert("description".to_string(), serde_json::Value::String("Path to the project directory containing .loglens/".to_string()));
            properties.insert("project_path".to_string(), serde_json::Value::Object(project_path_prop));

            // log_file_path property
            let mut log_file_path_prop = serde_json::Map::new();
            log_file_path_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            log_file_path_prop.insert("description".to_string(), serde_json::Value::String("Path to the log file (absolute or relative to project)".to_string()));
            properties.insert("log_file_path".to_string(), serde_json::Value::Object(log_file_path_prop));

            // level property
            let mut level_prop = serde_json::Map::new();
            level_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            level_prop.insert("enum".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("ERROR".to_string()),
                serde_json::Value::String("WARN".to_string()),
                serde_json::Value::String("INFO".to_string()),
                serde_json::Value::String("DEBUG".to_string()),
            ]));
            level_prop.insert("default".to_string(), serde_json::Value::String("ERROR".to_string()));
            properties.insert("level".to_string(), serde_json::Value::Object(level_prop));

            // provider property
            let mut provider_prop = serde_json::Map::new();
            provider_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            provider_prop.insert("enum".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("openrouter".to_string()),
                serde_json::Value::String("openai".to_string()),
                serde_json::Value::String("claude".to_string()),
                serde_json::Value::String("gemini".to_string()),
            ]));
            provider_prop.insert("default".to_string(), serde_json::Value::String("openrouter".to_string()));
            properties.insert("provider".to_string(), serde_json::Value::Object(provider_prop));

            // auto_analyze property
            let mut auto_analyze_prop = serde_json::Map::new();
            auto_analyze_prop.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
            auto_analyze_prop.insert("default".to_string(), serde_json::Value::Bool(true));
            properties.insert("auto_analyze".to_string(), serde_json::Value::Object(auto_analyze_prop));

            // api_key property (optional)
            let mut api_key_prop = serde_json::Map::new();
            api_key_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            api_key_prop.insert("description".to_string(), serde_json::Value::String("API key for the provider (optional if set in config/env)".to_string()));
            properties.insert("api_key".to_string(), serde_json::Value::Object(api_key_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("project_path".to_string()),
                serde_json::Value::String("log_file_path".to_string()),
            ]));

            schema
        });

        let get_analysis_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // analysis_id property
            let mut analysis_id_prop = serde_json::Map::new();
            analysis_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            analysis_id_prop.insert("description".to_string(), serde_json::Value::String("UUID of the analysis to retrieve".to_string()));
            properties.insert("analysis_id".to_string(), serde_json::Value::Object(analysis_id_prop));

            // project_path property (optional for validation)
            let mut project_path_prop = serde_json::Map::new();
            project_path_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            project_path_prop.insert("description".to_string(), serde_json::Value::String("Optional project path for validation".to_string()));
            properties.insert("project_path".to_string(), serde_json::Value::Object(project_path_prop));

            // format property
            let mut format_prop = serde_json::Map::new();
            format_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            format_prop.insert("enum".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("summary".to_string()),
                serde_json::Value::String("full".to_string()),
                serde_json::Value::String("structured".to_string()),
            ]));
            format_prop.insert("default".to_string(), serde_json::Value::String("summary".to_string()));
            properties.insert("format".to_string(), serde_json::Value::Object(format_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("analysis_id".to_string()),
            ]));

            schema
        });

        let query_analyses_schema = std::sync::Arc::new({
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));

            let mut properties = serde_json::Map::new();

            // project_path property (optional)
            let mut project_path_prop = serde_json::Map::new();
            project_path_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            project_path_prop.insert("description".to_string(), serde_json::Value::String("Filter by project path".to_string()));
            properties.insert("project_path".to_string(), serde_json::Value::Object(project_path_prop));

            // status property (optional)
            let mut status_prop = serde_json::Map::new();
            status_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            status_prop.insert("enum".to_string(), serde_json::Value::Array(vec![
                serde_json::Value::String("pending".to_string()),
                serde_json::Value::String("completed".to_string()),
                serde_json::Value::String("failed".to_string()),
            ]));
            properties.insert("status".to_string(), serde_json::Value::Object(status_prop));

            // limit property (optional)
            let mut limit_prop = serde_json::Map::new();
            limit_prop.insert("type".to_string(), serde_json::Value::String("integer".to_string()));
            limit_prop.insert("default".to_string(), serde_json::Value::Number(serde_json::Number::from(10)));
            properties.insert("limit".to_string(), serde_json::Value::Object(limit_prop));

            // since property (optional)
            let mut since_prop = serde_json::Map::new();
            since_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            since_prop.insert("description".to_string(), serde_json::Value::String("ISO timestamp to filter analyses since this time".to_string()));
            properties.insert("since".to_string(), serde_json::Value::Object(since_prop));

            schema.insert("properties".to_string(), serde_json::Value::Object(properties));

            schema
        });

        let mut tools = vec![
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

        // Add new project-linked tools if feature is enabled
        #[cfg(feature = "project-management")]
        {
            tools.extend(vec![
                Tool {
                    name: "add_log_file".into(),
                    description: Some("Add a log file to a project for analysis with background processing".into()),
                    input_schema: add_log_file_schema,
                    annotations: None,
                },
                Tool {
                    name: "get_analysis".into(),
                    description: Some("Retrieve analysis results by ID with different format options".into()),
                    input_schema: get_analysis_schema,
                    annotations: None,
                },
                Tool {
                    name: "query_analyses".into(),
                    description: Some("Query analyses with filters for project, status, and time".into()),
                    input_schema: query_analyses_schema,
                    annotations: None,
                },
            ]);
        }

        Ok(ListToolsResult {
            tools,
            next_cursor: None
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        match request.name.as_ref() {
            "analyze_logs" => self.handle_analyze_logs(request.arguments).await,
            "parse_logs" => self.handle_parse_logs(request.arguments).await,
            "filter_logs" => self.handle_filter_logs(request.arguments).await,
            #[cfg(feature = "project-management")]
            "add_log_file" => self.handle_add_log_file(request.arguments).await,
            #[cfg(feature = "project-management")]
            "get_analysis" => self.handle_get_analysis(request.arguments).await,
            #[cfg(feature = "project-management")]
            "query_analyses" => self.handle_query_analyses(request.arguments).await,
            _ => Err(method_not_found::<rmcp::model::CallToolRequestMethod>())
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

    #[cfg(feature = "project-management")]
    async fn handle_add_log_file(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::project::{database::create_pool, queries::create_analysis, get_or_create_project};
        use crate::mcp_server::async_analysis::spawn_analysis_task;
        use std::path::PathBuf;

        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let project_path = args.get("project_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing project_path parameter".to_string(), None))?;

        let log_file_path = args.get("log_file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing log_file_path parameter".to_string(), None))?;

        let level = args.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("ERROR");

        let provider = args.get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter");

        let auto_analyze = args.get("auto_analyze")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let api_key = args.get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Validate project path
        let validated_project_path = self.validate_project_path(project_path)?;
        
        // Resolve log file path
        let resolved_log_path = self.resolve_log_file_path(&validated_project_path, log_file_path)?;

        // Open database pool
        let pool = create_pool(&validated_project_path.join(".loglens"))
            .await
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Get or create project
        let project_id = get_or_create_project(&pool, &validated_project_path.to_string_lossy())
            .await
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Create analysis record
        let analysis_id = create_analysis(
            &pool,
            project_id,
            resolved_log_path.to_string_lossy().to_string(),
            provider.to_string(),
            level.to_string(),
        ).await
        .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Spawn analysis task if auto_analyze is enabled
        if auto_analyze {
            spawn_analysis_task(
                pool,
                analysis_id.clone(),
                resolved_log_path,
                provider.to_string(),
                level.to_string(),
                api_key,
            ).await
            .map_err(|e| McpError::InternalError(e.to_string()))?;
        }

        let response = serde_json::json!({
            "success": true,
            "analysis_id": analysis_id,
            "status": "pending",
            "message": "Log file added successfully"
        });

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&response).unwrap())]))
    }

    #[cfg(feature = "project-management")]
    async fn handle_get_analysis(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::project::{database::create_pool, queries::get_analysis_by_id};

        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let analysis_id = args.get("analysis_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing analysis_id parameter".to_string(), None))?;

        let project_path = args.get("project_path")
            .and_then(|v| v.as_str());

        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("summary");

        // Validate project path if provided
        let validated_project_path = if let Some(path) = project_path {
            Some(self.validate_project_path(path)?)
        } else {
            None
        };

        // Open database pool (use the provided project path or try to find the analysis)
        let pool = if let Some(project_path) = validated_project_path {
            create_pool(&project_path.join(".loglens"))
                .await
                .map_err(|e| McpError::DatabaseError(e.to_string()))?
        } else {
            // Try to open a pool in the current directory's .loglens
            create_pool(&env::current_dir().unwrap().join(".loglens"))
                .await
                .map_err(|e| McpError::DatabaseError(e.to_string()))?
        };

        // Get analysis by ID
        let result = get_analysis_by_id(&pool, analysis_id)
            .await
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        match result {
            Some((analysis, analysis_result)) => {
                // Validate project match if project_path was provided
                if let Some(project_path) = validated_project_path {
                    let project_str = project_path.to_string_lossy();
                    if analysis.project_id != project_str {
                        return Err(McpError::InvalidInput("Analysis does not belong to the specified project".to_string()));
                    }
                }

                // Format response based on format parameter
                let formatted_response = self.format_analysis_response(analysis, analysis_result, format)?;
                Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&formatted_response).unwrap())]))
            },
            None => Err(McpError::AnalysisNotFound(analysis_id.to_string()))
        }
    }

    #[cfg(feature = "project-management")]
    async fn handle_query_analyses(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::project::{database::create_pool, queries::query_analyses, get_project_by_path};
        use chrono::DateTime;

        let args = arguments.ok_or_else(||
            McpError::invalid_params("Missing arguments".to_string(), None))?;

        let project_path = args.get("project_path")
            .and_then(|v| v.as_str());

        let status_str = args.get("status")
            .and_then(|v| v.as_str());

        let limit = args.get("limit")
            .and_then(|v| v.as_u64())
            .map(|i| i as i64);

        let since_str = args.get("since")
            .and_then(|v| v.as_str());

        // Parse filters
        let project_id_opt = if let Some(project_path) = project_path {
            let validated_path = self.validate_project_path(project_path)?;
            let pool = create_pool(&validated_path.join(".loglens"))
                .await
                .map_err(|e| McpError::DatabaseError(e.to_string()))?;
            
            let project = get_project_by_path(&pool, &validated_path.to_string_lossy())
                .await
                .map_err(|e| McpError::DatabaseError(e.to_string()))?;
            
            project.map(|p| p.id)
        } else {
            None
        };

        let status_opt = if let Some(status_str) = status_str {
            match status_str {
                "pending" => Some(crate::project::models::AnalysisStatus::Pending),
                "completed" => Some(crate::project::models::AnalysisStatus::Completed),
                "failed" => Some(crate::project::models::AnalysisStatus::Failed),
                _ => return Err(McpError::InvalidInput(format!("Invalid status: {}", status_str)))
            }
        } else {
            None
        };

        let since_opt = if let Some(since_str) = since_str {
            Some(DateTime::parse_from_rfc3339(since_str)
                .map_err(|e| McpError::InvalidInput(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&chrono::Utc))
        } else {
            None
        };

        // Open database pool
        let pool_path = if let Some(project_path) = project_path {
            self.validate_project_path(project_path)?.join(".loglens")
        } else {
            env::current_dir().unwrap().join(".loglens")
        };

        let pool = create_pool(&pool_path)
            .await
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Query analyses
        let analyses = query_analyses(&pool, project_id_opt.as_deref(), status_opt, limit, since_opt)
            .await
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        let response = serde_json::json!({
            "success": true,
            "analyses": analyses,
            "count": analyses.len()
        });

        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&response).unwrap())]))
    }

    #[cfg(feature = "project-management")]
    fn validate_project_path(&self, path: &str) -> Result<PathBuf, McpError> {
        use std::fs;
        
        let project_path = PathBuf::from(path);

        // Must be absolute or resolve to absolute
        let abs_path = if project_path.is_absolute() {
            project_path
        } else {
            std::env::current_dir()
                .map_err(|e| McpError::InvalidProjectPath(e.to_string()))?
                .join(project_path)
        };

        // Must exist
        if !abs_path.exists() {
            return Err(McpError::ProjectNotFound(abs_path.display().to_string()));
        }

        // Must contain .loglens/ directory
        let loglens_dir = abs_path.join(".loglens");
        if !loglens_dir.exists() || !loglens_dir.is_dir() {
            return Err(McpError::InvalidProjectPath(
                format!("{} does not contain .loglens/ directory", abs_path.display())
            ));
        }

        // Must contain valid metadata.json
        let metadata_path = loglens_dir.join("metadata.json");
        if !metadata_path.exists() {
            return Err(McpError::InvalidProjectPath(
                "Missing .loglens/metadata.json".to_string()
            ));
        }

        Ok(abs_path)
    }

    #[cfg(feature = "project-management")]
    fn resolve_log_file_path(&self, project_path: &PathBuf, log_file: &str) -> Result<PathBuf, McpError> {
        let log_path = PathBuf::from(log_file);

        let abs_log_path = if log_path.is_absolute() {
            log_path
        } else {
            project_path.join(log_path)
        };

        if !abs_log_path.exists() {
            return Err(McpError::FileNotFound(abs_log_path.display().to_string()));
        }

        Ok(abs_log_path)
    }

    #[cfg(feature = "project-management")]
    fn format_analysis_response(
        &self,
        analysis: crate::project::models::Analysis,
        result: Option<crate::project::models::AnalysisResult>,
        format: &str,
    ) -> Result<serde_json::Value, McpError> {
        match format {
            "summary" => {
                Ok(serde_json::json!({
                    "success": true,
                    "analysis_id": analysis.id,
                    "project_id": analysis.project_id,
                    "status": analysis.status,
                    "log_file": analysis.log_file_path,
                    "summary": result.as_ref().and_then(|r| r.summary.clone()),
                    "issues_found": result.as_ref().and_then(|r| r.issues_found),
                    "patterns": result.as_ref().map(|r| &r.patterns_detected).unwrap_or(&vec![]),
                    "created_at": analysis.created_at,
                    "completed_at": analysis.completed_at,
                }))
            },
            "full" => {
                Ok(serde_json::json!({
                    "success": true,
                    "analysis": analysis,
                    "result": result,
                }))
            },
            "structured" => {
                Ok(serde_json::json!({
                    "success": true,
                    "analysis_id": analysis.id,
                    "project_id": analysis.project_id,
                    "status": analysis.status,
                    "log_file": analysis.log_file_path,
                    "provider": analysis.provider,
                    "level": analysis.level,
                    "summary": result.as_ref().and_then(|r| r.summary.clone()),
                    "full_report": result.as_ref().and_then(|r| r.full_report.clone()),
                    "issues_found": result.as_ref().and_then(|r| r.issues_found),
                    "patterns_detected": result.as_ref().map(|r| &r.patterns_detected).unwrap_or(&vec![]),
                    "metadata": result.as_ref().and_then(|r| r.metadata.clone()),
                    "created_at": analysis.created_at,
                    "completed_at": analysis.completed_at,
                }))
            },
            _ => Err(McpError::InvalidInput(format!("Invalid format: {}", format)))
        }
    }
}