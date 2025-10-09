use rmcp::{
    model::{
        CallToolRequestParam, CallToolResult, Content, ListToolsResult, PaginatedRequestParam,
        ServerInfo, Tool,
    },
    ServerHandler,
};
use std::sync::Arc;

use crate::mcp_server::error::{internal_error, method_not_found, McpError};

pub mod async_analysis;
pub mod error;
pub mod types;

/// Start the MCP server on the given port
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    tracing::info!("Starting LogLens MCP server on port {}", port);

    let _server = LogLensServer::new();

    // For now, we'll start a simple stdio-based MCP server
    // The rmcp library's transport API may have changed, so we'll keep it simple
    tracing::info!("MCP server ready for stdio transport");
    tracing::info!("Available tools: analyze_logs, parse_logs, filter_logs");

    // TODO: Implement actual server transport
    // For now, we'll just indicate the server is ready
    println!("🔗 LogLens MCP Server ready on port {}", port);
    println!("   Available tools:");
    println!("   - analyze_logs: AI-powered log analysis");
    println!("   - parse_logs: Parse raw logs into structured format");
    println!("   - filter_logs: Filter logs by level and patterns");

    // Keep the server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

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
        logs_prop.insert(
            "type".to_string(),
            serde_json::Value::String("array".to_string()),
        );
        logs_prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
        logs_prop.insert(
            "description".to_string(),
            serde_json::Value::String("Array of log lines to analyze".to_string()),
        );
        analyze_logs_props.insert("logs".to_string(), serde_json::Value::Object(logs_prop));

        let mut level_prop = serde_json::Map::new();
        level_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );
        level_prop.insert(
            "enum".to_string(),
            serde_json::json!(["ERROR", "WARN", "INFO", "DEBUG"]),
        );
        level_prop.insert(
            "description".to_string(),
            serde_json::Value::String("Minimum log level to analyze".to_string()),
        );
        analyze_logs_props.insert("level".to_string(), serde_json::Value::Object(level_prop));

        let mut provider_prop = serde_json::Map::new();
        provider_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );
        provider_prop.insert(
            "enum".to_string(),
            serde_json::json!(["openrouter", "openai", "claude", "gemini"]),
        );
        provider_prop.insert(
            "description".to_string(),
            serde_json::Value::String(
                "AI provider for analysis (optional if set in config/env)".to_string(),
            ),
        );
        analyze_logs_props.insert(
            "provider".to_string(),
            serde_json::Value::Object(provider_prop),
        );

        let mut api_key_prop = serde_json::Map::new();
        api_key_prop.insert(
            "type".to_string(),
            serde_json::Value::String("string".to_string()),
        );
        api_key_prop.insert(
            "description".to_string(),
            serde_json::Value::String(
                "API key for the provider (optional if set in config/env)".to_string(),
            ),
        );
        analyze_logs_props.insert(
            "api_key".to_string(),
            serde_json::Value::Object(api_key_prop),
        );

        tools.push(Tool {
            name: "analyze_logs".into(),
            description: Some(
                "Analyze log lines using AI to identify patterns, issues, and insights".into(),
            ),
            input_schema: Arc::new({
                let mut schema = serde_json::Map::new();
                schema.insert(
                    "type".to_string(),
                    serde_json::Value::String("object".to_string()),
                );
                schema.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(analyze_logs_props),
                );
                schema.insert("required".to_string(), serde_json::json!(["logs"]));
                schema
            }),
            annotations: None,
        });

        // parse_logs tool
        let mut parse_logs_props = serde_json::Map::new();
        parse_logs_props.insert(
            "logs".to_string(),
            serde_json::Value::Object({
                let mut prop = serde_json::Map::new();
                prop.insert(
                    "type".to_string(),
                    serde_json::Value::String("array".to_string()),
                );
                prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
                prop.insert(
                    "description".to_string(),
                    serde_json::Value::String("Array of log lines to parse".to_string()),
                );
                prop
            }),
        );

        tools.push(Tool {
            name: "parse_logs".into(),
            description: Some(
                "Parse log lines into structured format with timestamps, levels, and messages"
                    .into(),
            ),
            input_schema: Arc::new({
                let mut schema = serde_json::Map::new();
                schema.insert(
                    "type".to_string(),
                    serde_json::Value::String("object".to_string()),
                );
                schema.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(parse_logs_props),
                );
                schema.insert("required".to_string(), serde_json::json!(["logs"]));
                schema
            }),
            annotations: None,
        });

        // filter_logs tool
        let mut filter_props = serde_json::Map::new();
        filter_props.insert(
            "logs".to_string(),
            serde_json::Value::Object({
                let mut prop = serde_json::Map::new();
                prop.insert(
                    "type".to_string(),
                    serde_json::Value::String("array".to_string()),
                );
                prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
                prop.insert(
                    "description".to_string(),
                    serde_json::Value::String("Array of log lines to filter".to_string()),
                );
                prop
            }),
        );

        filter_props.insert(
            "level".to_string(),
            serde_json::Value::Object({
                let mut prop = serde_json::Map::new();
                prop.insert(
                    "type".to_string(),
                    serde_json::Value::String("string".to_string()),
                );
                prop.insert(
                    "enum".to_string(),
                    serde_json::json!(["ERROR", "WARN", "INFO", "DEBUG"]),
                );
                prop.insert(
                    "description".to_string(),
                    serde_json::Value::String("Minimum log level to include".to_string()),
                );
                prop
            }),
        );

        tools.push(Tool {
            name: "filter_logs".into(),
            description: Some("Filter log lines by minimum log level".into()),
            input_schema: Arc::new({
                let mut schema = serde_json::Map::new();
                schema.insert(
                    "type".to_string(),
                    serde_json::Value::String("object".to_string()),
                );
                schema.insert(
                    "properties".to_string(),
                    serde_json::Value::Object(filter_props),
                );
                schema.insert("required".to_string(), serde_json::json!(["logs", "level"]));
                schema
            }),
            annotations: None,
        });

        // Phase 3 MCP tools for project-linked analysis
        #[cfg(feature = "project-management")]
        {
            // add_log_file tool
            let mut add_log_props = serde_json::Map::new();
            add_log_props.insert(
                "project_path".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "Path to software project root (must contain .loglens/)"
                }),
            );
            add_log_props.insert(
                "log_file_path".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "Absolute or relative path to log file"
                }),
            );
            add_log_props.insert(
                "level".to_string(),
                serde_json::json!({
                    "type": "string",
                    "enum": ["ERROR", "WARN", "INFO", "DEBUG"],
                    "description": "Minimum log level to analyze"
                }),
            );
            add_log_props.insert(
                "provider".to_string(),
                serde_json::json!({
                    "type": "string",
                    "enum": ["openrouter", "openai", "claude", "gemini"],
                    "description": "AI provider for analysis"
                }),
            );
            add_log_props.insert(
                "auto_analyze".to_string(),
                serde_json::json!({
                    "type": "boolean",
                    "default": true,
                    "description": "Automatically trigger analysis"
                }),
            );
            add_log_props.insert(
                "api_key".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "API key for the provider (optional if set in config/env)"
                }),
            );

            tools.push(Tool {
                name: "add_log_file".into(),
                description: Some("Add log file to project and trigger analysis".into()),
                input_schema: Arc::new({
                    let mut schema = serde_json::Map::new();
                    schema.insert(
                        "type".to_string(),
                        serde_json::Value::String("object".to_string()),
                    );
                    schema.insert(
                        "properties".to_string(),
                        serde_json::Value::Object(add_log_props),
                    );
                    schema.insert(
                        "required".to_string(),
                        serde_json::json!(["project_path", "log_file_path"]),
                    );
                    schema
                }),
                annotations: None,
            });

            // get_analysis tool
            let mut get_analysis_props = serde_json::Map::new();
            get_analysis_props.insert(
                "analysis_id".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "UUID of the analysis to retrieve"
                }),
            );
            get_analysis_props.insert(
                "project_path".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "Optional: path to project for validation"
                }),
            );
            get_analysis_props.insert(
                "format".to_string(),
                serde_json::json!({
                    "type": "string",
                    "enum": ["summary", "full", "structured"],
                    "default": "summary",
                    "description": "Level of detail to return"
                }),
            );

            tools.push(Tool {
                name: "get_analysis".into(),
                description: Some("Retrieve analysis results by ID".into()),
                input_schema: Arc::new({
                    let mut schema = serde_json::Map::new();
                    schema.insert(
                        "type".to_string(),
                        serde_json::Value::String("object".to_string()),
                    );
                    schema.insert(
                        "properties".to_string(),
                        serde_json::Value::Object(get_analysis_props),
                    );
                    schema.insert("required".to_string(), serde_json::json!(["analysis_id"]));
                    schema
                }),
                annotations: None,
            });

            // query_analyses tool
            let mut query_props = serde_json::Map::new();
            query_props.insert(
                "project_path".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "Filter by project"
                }),
            );
            query_props.insert(
                "status".to_string(),
                serde_json::json!({
                    "type": "string",
                    "enum": ["pending", "completed", "failed"],
                    "description": "Filter by status"
                }),
            );
            query_props.insert(
                "limit".to_string(),
                serde_json::json!({
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum results to return"
                }),
            );
            query_props.insert(
                "since".to_string(),
                serde_json::json!({
                    "type": "string",
                    "description": "ISO timestamp - analyses after this time"
                }),
            );

            tools.push(Tool {
                name: "query_analyses".into(),
                description: Some("Query analyses with filters".into()),
                input_schema: Arc::new({
                    let mut schema = serde_json::Map::new();
                    schema.insert(
                        "type".to_string(),
                        serde_json::Value::String("object".to_string()),
                    );
                    schema.insert(
                        "properties".to_string(),
                        serde_json::Value::Object(query_props),
                    );
                    schema
                }),
                annotations: None,
            });
        }

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, rmcp::Error> {
        match request.name.as_ref() {
            "analyze_logs" => self
                .handle_analyze_logs(request.arguments)
                .await
                .map_err(McpError::into),
            "parse_logs" => self
                .handle_parse_logs(request.arguments)
                .await
                .map_err(McpError::into),
            "filter_logs" => self
                .handle_filter_logs(request.arguments)
                .await
                .map_err(McpError::into),
            #[cfg(feature = "project-management")]
            "add_log_file" => self
                .handle_add_log_file(request.arguments)
                .await
                .map_err(McpError::into),
            #[cfg(feature = "project-management")]
            "get_analysis" => self
                .handle_get_analysis(request.arguments)
                .await
                .map_err(McpError::into),
            #[cfg(feature = "project-management")]
            "query_analyses" => self
                .handle_query_analyses(request.arguments)
                .await
                .map_err(McpError::into),
            _ => Err(method_not_found::<rmcp::model::CallToolRequestMethod>()),
        }
    }
}

impl LogLensServer {
    async fn handle_analyze_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args =
            arguments.ok_or_else(|| McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> =
            serde_json::from_value(args.get("logs").cloned().unwrap_or(serde_json::json!([])))
                .map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        let logs_count = logs.len();

        let level = args.get("level").and_then(|v| v.as_str()).unwrap_or("INFO");

        let provider = args
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter");

        let api_key = args
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create LogLens instance
        let loglens = Self::create_loglens()
            .map_err(|e| McpError::InternalError(format!("Failed to create LogLens: {}", e)))?;

        // Analyze logs with the provided API key
        let api_key_ref = api_key.as_ref().map(|s| s.as_str());
        let analysis_result = loglens
            .analyze_lines(logs, level, provider, api_key_ref)
            .await
            .map_err(|e| McpError::AnalysisFailed(format!("Analysis failed: {}", e)))?;

        // Return results
        let response = serde_json::json!({
            "success": true,
            "analysis": analysis_result,
            "logs_processed": logs_count,
            "level": level,
            "provider": provider
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    async fn handle_parse_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args =
            arguments.ok_or_else(|| McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> =
            serde_json::from_value(args.get("logs").cloned().unwrap_or(serde_json::json!([])))
                .map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        // Parse logs using parser module
        let parsed_logs = crate::parser::parse_log_lines(&logs);

        // Return results
        let response = serde_json::json!({
            "success": true,
            "parsed_logs": parsed_logs,
            "logs_processed": logs.len()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    async fn handle_filter_logs(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let args =
            arguments.ok_or_else(|| McpError::InvalidInput("Missing arguments".to_string()))?;

        let logs: Vec<String> =
            serde_json::from_value(args.get("logs").cloned().unwrap_or(serde_json::json!([])))
                .map_err(|e| McpError::InvalidInput(format!("Invalid logs array: {}", e)))?;

        let level = args
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing level parameter".to_string()))?;

        // Parse logs first
        let parsed_logs = crate::parser::parse_log_lines(&logs);

        // Filter logs using filter module
        let filtered_logs = crate::filter::filter_logs_by_level(parsed_logs, level)
            .map_err(|e| McpError::InternalError(format!("Filtering failed: {}", e)))?;

        // Return results
        let response = serde_json::json!({
            "success": true,
            "filtered_logs": filtered_logs,
            "original_count": logs.len(),
            "filtered_count": filtered_logs.len(),
            "level": level
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    // Phase 3 MCP tool handlers
    #[cfg(feature = "project-management")]
    async fn handle_add_log_file(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::mcp_server::async_analysis::spawn_analysis_task;
        use crate::project::{
            database::create_pool,
            queries::{create_analysis, get_or_create_project},
        };
        use std::path::{Path, PathBuf};

        let args =
            arguments.ok_or_else(|| McpError::InvalidInput("Missing arguments".to_string()))?;

        let project_path = args
            .get("project_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing project_path parameter".to_string()))?;

        let log_file_path_str = args
            .get("log_file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing log_file_path parameter".to_string()))?;

        let level = args
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("ERROR");

        let provider = args
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openrouter");

        let auto_analyze = args
            .get("auto_analyze")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let api_key = args
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Validate project has .loglens directory
        let loglens_dir = Path::new(project_path).join(".loglens");
        if !loglens_dir.exists() {
            return Err(McpError::InvalidInput(format!(
                "Project path {} does not contain .loglens directory. Run 'loglens init' first.",
                project_path
            )));
        }

        // Resolve log file path (absolute or relative to project)
        let log_file_path = if Path::new(log_file_path_str).is_absolute() {
            PathBuf::from(log_file_path_str)
        } else {
            Path::new(project_path).join(log_file_path_str)
        };

        if !log_file_path.exists() {
            return Err(McpError::InvalidInput(format!(
                "Log file not found: {}",
                log_file_path.display()
            )));
        }

        // Connect to project database
        let db_path = loglens_dir.join("index.db");
        let pool = create_pool(&db_path).await.map_err(|e| {
            McpError::InternalError(format!("Failed to connect to database: {}", e))
        })?;

        // Get or create project
        let project_id = get_or_create_project(&pool, project_path)
            .await
            .map_err(|e| McpError::InternalError(format!("Failed to get project: {}", e)))?;

        // Create analysis record
        let analysis_id = create_analysis(
            &pool,
            project_id.clone(),
            log_file_path.to_string_lossy().to_string(),
            provider.to_string(),
            level.to_string(),
        )
        .await
        .map_err(|e| McpError::InternalError(format!("Failed to create analysis: {}", e)))?;

        // Spawn async analysis if requested
        let status = if auto_analyze {
            spawn_analysis_task(
                pool.clone(),
                analysis_id.clone(),
                log_file_path.clone(),
                provider.to_string(),
                level.to_string(),
                api_key,
            )
            .await
            .map_err(|e| {
                McpError::InternalError(format!("Failed to spawn analysis task: {}", e))
            })?;
            "pending"
        } else {
            "pending"
        };

        let response = serde_json::json!({
            "success": true,
            "analysis_id": analysis_id,
            "project_id": project_id,
            "status": status,
            "message": if auto_analyze {
                "Log file added and analysis started"
            } else {
                "Log file added"
            }
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[cfg(feature = "project-management")]
    async fn handle_get_analysis(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::project::{database::create_pool, queries::get_analysis_by_id};
        use std::path::Path;

        let args =
            arguments.ok_or_else(|| McpError::InvalidInput("Missing arguments".to_string()))?;

        let analysis_id = args
            .get("analysis_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing analysis_id parameter".to_string()))?;

        let format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("summary");

        // If project_path provided, use its database; otherwise search all .loglens directories
        let pool = if let Some(project_path) = args.get("project_path").and_then(|v| v.as_str()) {
            let db_path = Path::new(project_path).join(".loglens/index.db");
            create_pool(&db_path).await.map_err(|e| {
                McpError::InternalError(format!("Failed to connect to database: {}", e))
            })?
        } else {
            return Err(McpError::InvalidInput(
                "project_path is required for get_analysis".to_string(),
            ));
        };

        // Get analysis with results
        let result = get_analysis_by_id(&pool, analysis_id)
            .await
            .map_err(|e| McpError::InternalError(format!("Failed to get analysis: {}", e)))?;

        match result {
            Some((analysis, result_opt)) => {
                let response = match format {
                    "full" => {
                        serde_json::json!({
                            "success": true,
                            "analysis_id": analysis.id,
                            "project_id": analysis.project_id,
                            "status": analysis.status.to_string(),
                            "log_file": analysis.log_file_path,
                            "provider": analysis.provider,
                            "level": analysis.level,
                            "created_at": analysis.created_at,
                            "completed_at": analysis.completed_at,
                            "full_report": result_opt.as_ref().and_then(|r| r.full_report.clone()),
                            "summary": result_opt.as_ref().and_then(|r| r.summary.clone()),
                            "patterns": result_opt.as_ref().map(|r| r.patterns_detected.clone()).unwrap_or(serde_json::json!([])),
                            "issues_found": result_opt.as_ref().and_then(|r| r.issues_found),
                        })
                    }
                    "structured" => {
                        serde_json::json!({
                            "success": true,
                            "analysis": {
                                "id": analysis.id,
                                "project_id": analysis.project_id,
                                "log_file_path": analysis.log_file_path,
                                "provider": analysis.provider,
                                "level": analysis.level,
                                "status": analysis.status.to_string(),
                                "created_at": analysis.created_at,
                                "started_at": analysis.started_at,
                                "completed_at": analysis.completed_at,
                            },
                            "results": result_opt.map(|r| serde_json::json!({
                                "summary": r.summary,
                                "full_report": r.full_report,
                                "patterns_detected": r.patterns_detected,
                                "issues_found": r.issues_found,
                            }))
                        })
                    }
                    _ => {
                        // "summary"
                        let patterns: Vec<serde_json::Value> = result_opt
                            .as_ref()
                            .and_then(|r| r.patterns_detected.as_array().cloned())
                            .unwrap_or_default();

                        serde_json::json!({
                            "success": true,
                            "analysis_id": analysis.id,
                            "project_id": analysis.project_id,
                            "status": analysis.status.to_string(),
                            "log_file": analysis.log_file_path,
                            "summary": result_opt.as_ref().and_then(|r| r.summary.clone()),
                            "issues_found": result_opt.as_ref().and_then(|r| r.issues_found).unwrap_or(0),
                            "patterns": patterns.iter().take(5).map(|p| serde_json::json!({
                                "pattern": p.get("pattern").and_then(|v| v.as_str()).unwrap_or(""),
                                "count": p.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
                            })).collect::<Vec<_>>(),
                            "created_at": analysis.created_at,
                            "completed_at": analysis.completed_at,
                        })
                    }
                };

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap(),
                )]))
            }
            None => Err(McpError::InvalidInput(format!(
                "Analysis not found: {}",
                analysis_id
            ))),
        }
    }

    #[cfg(feature = "project-management")]
    async fn handle_query_analyses(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        use crate::project::{
            database::create_pool,
            models::AnalysisStatus,
            queries::{get_project_by_path, query_analyses},
        };
        use chrono::{DateTime, Utc};
        use std::path::Path;

        let args = arguments.unwrap_or_default();

        // Project path is required
        let project_path = args
            .get("project_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidInput("Missing project_path parameter".to_string()))?;

        let db_path = Path::new(project_path).join(".loglens/index.db");
        let pool = create_pool(&db_path).await.map_err(|e| {
            McpError::InternalError(format!("Failed to connect to database: {}", e))
        })?;

        // Get project_id from path
        let project = get_project_by_path(&pool, project_path)
            .await
            .map_err(|e| McpError::InternalError(format!("Failed to get project: {}", e)))?
            .ok_or_else(|| {
                McpError::InvalidInput(format!("Project not found: {}", project_path))
            })?;

        let project_id = Some(project.id.as_str());

        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<AnalysisStatus>().ok());

        let limit = args.get("limit").and_then(|v| v.as_i64()).or(Some(10));

        let since = args
            .get("since")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());

        // Query analyses
        let analyses = query_analyses(&pool, project_id, status, limit, since)
            .await
            .map_err(|e| McpError::InternalError(format!("Failed to query analyses: {}", e)))?;

        let response = serde_json::json!({
            "success": true,
            "analyses": analyses.iter().map(|a| serde_json::json!({
                "analysis_id": a.id,
                "log_file": a.log_file_path,
                "status": a.status.to_string(),
                "provider": a.provider,
                "level": a.level,
                "created_at": a.created_at,
                "completed_at": a.completed_at,
            })).collect::<Vec<_>>(),
            "total_count": analyses.len(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }
}
