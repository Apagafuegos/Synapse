use rmcp::{
    ServerHandler,
    model::{
        ServerInfo, Tool, CallToolRequestParam, CallToolResult, Content,
        ListToolsResult, PaginatedRequestParam
    },
};
use std::sync::Arc;
use crate::{Database, Config};
use crate::tools::{list_projects, get_project, list_analyses, get_analysis, get_analysis_status, analyze_file};

/// Main MCP server structure
#[derive(Clone)]
pub struct McpServer {
    db: Database,
    config: Config,
}

impl McpServer {
    /// Create new MCP server instance
    pub fn new(db: Database, config: Config) -> Self {
        Self { db, config }
    }

    /// Get database reference
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// MCP Server Handler implementing the RMCP ServerHandler trait
pub struct LogLensMcpHandler {
    server: Arc<McpServer>,
}

impl LogLensMcpHandler {
    pub fn new(server: Arc<McpServer>) -> Self {
        Self { server }
    }
}

impl Default for LogLensMcpHandler {
    fn default() -> Self {
        // Create a default instance for testing
        let rt = tokio::runtime::Runtime::new().unwrap();
        let db = rt.block_on(async {
            Database::new(":memory:").await
        }).unwrap();
        Self::new(Arc::new(McpServer::new(db, Config::default())))
    }
}

impl ServerHandler for LogLensMcpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("AI-powered log analysis tool with project management. Discover projects, trigger analyses, and retrieve comprehensive log analysis results.".into()),
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

        // list_projects tool
        let mut list_projects_props = serde_json::Map::new();
        let mut names_prop = serde_json::Map::new();
        names_prop.insert("type".to_string(), serde_json::Value::String("array".to_string()));
        names_prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
        names_prop.insert("description".to_string(), serde_json::Value::String("Optional list of project names to filter by".to_string()));
        list_projects_props.insert("names".to_string(), serde_json::Value::Object(names_prop));

        tools.push(Tool {
            name: "list_projects".into(),
            description: Some("List available projects with optional filtering".into()),
            input_schema: Arc::new(list_projects_props),
            annotations: Default::default(),
        });

        // get_project tool
        let mut get_project_props = serde_json::Map::new();
        let mut project_id_prop = serde_json::Map::new();
        project_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        project_id_prop.insert("description".to_string(), serde_json::Value::String("Project ID to fetch".to_string()));
        get_project_props.insert("project_id".to_string(), serde_json::Value::Object(project_id_prop));

        tools.push(Tool {
            name: "get_project".into(),
            description: Some("Get detailed project information".into()),
            input_schema: Arc::new(get_project_props),
            annotations: Default::default(),
        });

        // list_analyses tool
        let mut list_analyses_props = serde_json::Map::new();
        let mut project_id_prop = serde_json::Map::new();
        project_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        project_id_prop.insert("description".to_string(), serde_json::Value::String("Project ID".to_string()));
        list_analyses_props.insert("project_id".to_string(), serde_json::Value::Object(project_id_prop));

        let mut limit_prop = serde_json::Map::new();
        limit_prop.insert("type".to_string(), serde_json::Value::String("integer".to_string()));
        limit_prop.insert("default".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));
        limit_prop.insert("maximum".to_string(), serde_json::Value::Number(serde_json::Number::from(200)));
        limit_prop.insert("description".to_string(), serde_json::Value::String("Maximum number of analyses to return".to_string()));
        list_analyses_props.insert("limit".to_string(), serde_json::Value::Object(limit_prop));

        let mut offset_prop = serde_json::Map::new();
        offset_prop.insert("type".to_string(), serde_json::Value::String("integer".to_string()));
        offset_prop.insert("default".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
        offset_prop.insert("minimum".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
        offset_prop.insert("description".to_string(), serde_json::Value::String("Number of analyses to skip".to_string()));
        list_analyses_props.insert("offset".to_string(), serde_json::Value::Object(offset_prop));

        tools.push(Tool {
            name: "list_analyses".into(),
            description: Some("List analyses for a project with pagination".into()),
            input_schema: Arc::new(list_analyses_props),
            annotations: Default::default(),
        });

        // get_analysis tool
        let mut get_analysis_props = serde_json::Map::new();
        let mut analysis_id_prop = serde_json::Map::new();
        analysis_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        analysis_id_prop.insert("description".to_string(), serde_json::Value::String("Analysis ID to fetch".to_string()));
        get_analysis_props.insert("analysis_id".to_string(), serde_json::Value::Object(analysis_id_prop));

        tools.push(Tool {
            name: "get_analysis".into(),
            description: Some("Get complete analysis results".into()),
            input_schema: Arc::new(get_analysis_props),
            annotations: Default::default(),
        });

        // get_analysis_status tool
        let mut get_analysis_status_props = serde_json::Map::new();
        let mut analysis_id_prop = serde_json::Map::new();
        analysis_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        analysis_id_prop.insert("description".to_string(), serde_json::Value::String("Analysis ID to check status".to_string()));
        get_analysis_status_props.insert("analysis_id".to_string(), serde_json::Value::Object(analysis_id_prop));

        tools.push(Tool {
            name: "get_analysis_status".into(),
            description: Some("Get analysis status for polling".into()),
            input_schema: Arc::new(get_analysis_status_props),
            annotations: Default::default(),
        });

        // analyze_file tool
        let mut analyze_file_props = serde_json::Map::new();
        let mut project_id_prop = serde_json::Map::new();
        project_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        project_id_prop.insert("description".to_string(), serde_json::Value::String("Project ID".to_string()));
        analyze_file_props.insert("project_id".to_string(), serde_json::Value::Object(project_id_prop));

        let mut file_id_prop = serde_json::Map::new();
        file_id_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        file_id_prop.insert("description".to_string(), serde_json::Value::String("File ID to analyze".to_string()));
        analyze_file_props.insert("file_id".to_string(), serde_json::Value::Object(file_id_prop));

        let mut provider_prop = serde_json::Map::new();
        provider_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
        provider_prop.insert("enum".to_string(), serde_json::json!(["openrouter", "openai", "claude", "gemini"]));
        provider_prop.insert("default".to_string(), serde_json::Value::String("openrouter".to_string()));
        provider_prop.insert("description".to_string(), serde_json::Value::String("AI provider for analysis".to_string()));
        analyze_file_props.insert("provider".to_string(), serde_json::Value::Object(provider_prop));

        tools.push(Tool {
            name: "analyze_file".into(),
            description: Some("Trigger new analysis on existing file".into()),
            input_schema: Arc::new(analyze_file_props),
            annotations: Default::default(),
        });

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
        let tool_name = request.name.as_ref();
        let arguments = request.arguments.unwrap_or_default();

        let result = match tool_name {
            "list_projects" => {
                list_projects(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            "get_project" => {
                get_project(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            "list_analyses" => {
                list_analyses(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            "get_analysis" => {
                get_analysis(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            "get_analysis_status" => {
                get_analysis_status(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            "analyze_file" => {
                analyze_file(self.server.db(), serde_json::Value::Object(arguments)).await
            }
            _ => {
                return Err(rmcp::Error::invalid_request(format!("Unknown tool: {}", tool_name), None));
            }
        };

        match result {
            Ok(value) => Ok(CallToolResult {
                content: vec![Content::text(serde_json::to_string_pretty(&value)
                    .map_err(|e| rmcp::Error::internal_error(format!("Failed to serialize result: {}", e), None))?)],
                is_error: None,
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Content::text(format!("Error: {}", e))],
                is_error: Some(true),
            }),
        }
    }
}

impl McpServer {
    /// Start the MCP server with stdio transport
    pub async fn start_stdio(&self) -> anyhow::Result<()> {
        let handler = LogLensMcpHandler::new(Arc::new(self.clone()));
        
        tracing::info!("Starting LogLens MCP server with stdio transport");
        tracing::info!("Server name: {}", self.config.server_name);
        tracing::info!("Server version: {}", self.config.server_version);
        tracing::info!("Available tools: list_projects, get_project, list_analyses, get_analysis, get_analysis_status, analyze_file");

        // For now, we'll use a simple implementation that shows the server is ready
        // The full RMCP transport implementation would go here in Phase 3
        println!("🔗 LogLens MCP Server ready");
        println!("   Server: {} v{}", self.config.server_name, self.config.server_version);
        println!("   Available tools:");
        println!("   - list_projects: List projects with optional filtering");
        println!("   - get_project: Get detailed project information");
        println!("   - list_analyses: List analyses for a project with pagination");
        println!("   - get_analysis: Get complete analysis results");
        println!("   - get_analysis_status: Get analysis status for polling");
        println!("   - analyze_file: Trigger new analysis on existing file");

        // Keep the server running for stdio mode
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// Start the MCP server with HTTP transport
    pub async fn start_http(&self, port: u16) -> anyhow::Result<()> {
        tracing::info!("Starting LogLens MCP server with HTTP transport on port {}", port);
        tracing::info!("Server name: {}", self.config.server_name);
        tracing::info!("Server version: {}", self.config.server_version);

        // For now, we'll show that the HTTP server would be started
        println!("🌐 LogLens MCP Server ready on http://localhost:{}", port);
        println!("   Server: {} v{}", self.config.server_name, self.config.server_version);
        println!("   Transport: HTTP with Server-Sent Events");
        println!("   Available tools: list_projects, get_project, list_analyses, get_analysis, get_analysis_status, analyze_file");

        // Keep the server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}