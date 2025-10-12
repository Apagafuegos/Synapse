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
    pub db: Database,
    pub config: Config,
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
#[derive(Clone)]
pub struct SynapseMcpHandler {
    pub server: Arc<McpServer>,
}

impl SynapseMcpHandler {
    pub fn new(server: Arc<McpServer>) -> Self {
        Self { server }
    }
}

impl Default for SynapseMcpHandler {
    fn default() -> Self {
        // Create a default instance for testing
        let rt = tokio::runtime::Runtime::new().unwrap();
        let db = rt.block_on(async {
            Database::new(":memory:").await
        }).unwrap();
        Self::new(Arc::new(McpServer::new(db, Config::default())))
    }
}

impl ServerHandler for SynapseMcpHandler {
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
        let tools = vec![
            Tool {
                name: "list_projects".into(),
                description: Some("List available projects with optional filtering".into()),
                input_schema: Arc::new(crate::schema::list_projects_schema()),
                annotations: Default::default(),
            },
            Tool {
                name: "get_project".into(),
                description: Some("Get detailed project information".into()),
                input_schema: Arc::new(crate::schema::get_project_schema()),
                annotations: Default::default(),
            },
            Tool {
                name: "list_analyses".into(),
                description: Some("List analyses for a project with pagination".into()),
                input_schema: Arc::new(crate::schema::list_analyses_schema()),
                annotations: Default::default(),
            },
            Tool {
                name: "get_analysis".into(),
                description: Some("Get complete analysis results".into()),
                input_schema: Arc::new(crate::schema::get_analysis_schema()),
                annotations: Default::default(),
            },
            Tool {
                name: "get_analysis_status".into(),
                description: Some("Get analysis status for polling".into()),
                input_schema: Arc::new(crate::schema::get_analysis_status_schema()),
                annotations: Default::default(),
            },
            Tool {
                name: "analyze_file".into(),
                description: Some("Trigger new analysis on existing file".into()),
                input_schema: Arc::new(crate::schema::analyze_file_schema()),
                annotations: Default::default(),
            },
        ];

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
        let arguments_value = serde_json::Value::Object(arguments);

        // Validate input parameters
        if let Err(e) = crate::validation::validate_tool_params(tool_name, &arguments_value) {
            return Ok(CallToolResult {
                content: vec![Content::text(format!("Validation error: {}", e))],
                is_error: Some(true),
            });
        }

        let result = match tool_name {
            "list_projects" => {
                list_projects(self.server.db(), arguments_value).await
            }
            "get_project" => {
                get_project(self.server.db(), arguments_value).await
            }
            "list_analyses" => {
                list_analyses(self.server.db(), arguments_value).await
            }
            "get_analysis" => {
                get_analysis(self.server.db(), arguments_value).await
            }
            "get_analysis_status" => {
                get_analysis_status(self.server.db(), arguments_value).await
            }
            "analyze_file" => {
                analyze_file(self.server.db(), arguments_value).await
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
    /// Create a handler for this server
    pub fn create_handler(&self) -> SynapseMcpHandler {
        SynapseMcpHandler::new(Arc::new(self.clone()))
    }

    /// Start the MCP server with stdio transport
    /// IMPORTANT: No logging is done here to avoid contaminating the JSON-RPC protocol on stdout
    pub async fn start_stdio(&self) -> anyhow::Result<()> {
        use crate::transport::{TransportType, create_and_run_transport};

        let handler = Arc::new(self.create_handler());

        // NO LOGGING - stdio transport requires pure JSON-RPC on stdout
        // Any log output will corrupt the protocol and break MCP clients

        create_and_run_transport(TransportType::Stdio, handler).await
    }

    /// Start the MCP server with HTTP transport
    pub async fn start_http(&self, port: u16) -> anyhow::Result<()> {
        use crate::transport::{TransportType, create_and_run_transport};
        
        let handler = Arc::new(self.create_handler());
        
        tracing::info!("Starting Synapse MCP server with HTTP transport on port {}", port);
        tracing::info!("Server name: {}", self.config.server_name);
        tracing::info!("Server version: {}", self.config.server_version);
        tracing::info!("Available tools: list_projects, get_project, list_analyses, get_analysis, get_analysis_status, analyze_file");

        create_and_run_transport(TransportType::Http { port }, handler).await
    }
}