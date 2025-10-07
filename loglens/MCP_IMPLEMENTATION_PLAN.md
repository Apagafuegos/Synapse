# MCP Server Implementation Plan for LogLens

## Context
Replace non-functional JSON-RPC implementation with proper MCP SDK server using the official Rust SDK (https://github.com/modelcontextprotocol/rust-sdk). Support STDIO/HTTP transports, provide AI clients project-level insights through structured tools.

## Architecture

```rust
loglens-mcp/
├── src/
│   ├── lib.rs           // Public API: McpServer struct, create_server()
│   ├── server.rs        // Core: tool handler implementation, request routing
│   ├── transport/
│   │   ├── mod.rs       // Transport enum, factory function
│   │   ├── stdio.rs     // StdioServerTransport wrapper
│   │   └── http.rs      // HttpServerTransport wrapper (SSE-based)
│   ├── tools/
│   │   ├── mod.rs       // Tool definitions, schemas, execution handlers
│   │   ├── projects.rs  // list_projects, get_project implementations
│   │   ├── analyses.rs  // list_analyses, get_analysis, get_analysis_status
│   │   └── analyze.rs   // analyze_file implementation
│   └── schema.rs        // JSON schema builders for each tool
└── Cargo.toml           // Dependencies: mcp-sdk, tokio, serde_json
```

## Core Dependencies

```toml
[dependencies]
mcp-sdk = { git = "https://github.com/modelcontextprotocol/rust-sdk" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
```

## Tool Specifications

### 1. list_projects

**Purpose**: Discover available projects with optional name filtering

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "names": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Optional list of project names to filter by"
    }
  }
}
```

**Output**:
```json
{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "id": { "type": "string" },
      "name": { "type": "string" },
      "description": { "type": "string" },
      "file_count": { "type": "integer" },
      "analysis_count": { "type": "integer" },
      "last_analysis_date": { "type": "string", "format": "date-time" }
    }
  }
}
```

**Implementation**:
- Query `projects` table
- If `names` provided: `WHERE name IN (?)`
- Join with `files` and `analyses` tables for counts
- Return all fields with aggregated statistics

### 2. get_project

**Purpose**: Fetch detailed project information

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project_id": { "type": "string" }
  },
  "required": ["project_id"]
}
```

**Output**: Single project object with same structure as `list_projects` item plus `created_at`, `updated_at`

**Implementation**:
- Query `projects` WHERE `id = ?`
- Aggregate file count, analysis count, latest analysis date
- Return 404 error if project not found

### 3. list_analyses

**Purpose**: Get analyses for a specific project

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project_id": { "type": "string" },
    "limit": { "type": "integer", "default": 50, "maximum": 200 },
    "offset": { "type": "integer", "default": 0 }
  },
  "required": ["project_id"]
}
```

**Output**:
```json
{
  "type": "object",
  "properties": {
    "analyses": {
      "type": "array",
      "items": {
        "id": { "type": "string" },
        "project_id": { "type": "string" },
        "file_id": { "type": "string" },
        "status": { "type": "string", "enum": ["pending", "running", "completed", "failed"] },
        "created_at": { "type": "string" },
        "completed_at": { "type": "string" }
      }
    },
    "total": { "type": "integer" }
  }
}
```

**Implementation**:
- Query `analyses` WHERE `project_id = ?` ORDER BY `created_at DESC`
- Apply LIMIT and OFFSET for pagination
- Return array with total count for pagination

### 4. get_analysis

**Purpose**: Retrieve complete analysis results

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "analysis_id": { "type": "string" }
  },
  "required": ["analysis_id"]
}
```

**Output**:
```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "project_id": { "type": "string" },
    "file_id": { "type": "string" },
    "status": { "type": "string" },
    "summary": { "type": "string" },
    "errors": { "type": "array" },
    "warnings": { "type": "array" },
    "recommendations": { "type": "array" },
    "patterns": { "type": "array" },
    "performance_metrics": { "type": "object" },
    "metadata": { "type": "object" }
  }
}
```

**Implementation**:
- Query `analyses` JOIN `analysis_results` WHERE `analyses.id = ?`
- Deserialize JSON fields: `errors`, `recommendations`, `patterns`, etc.
- Return complete analysis report structure

### 5. get_analysis_status

**Purpose**: Poll analysis progress for async operations

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "analysis_id": { "type": "string" }
  },
  "required": ["analysis_id"]
}
```

**Output**:
```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "status": { "type": "string", "enum": ["pending", "running", "completed", "failed"] },
    "progress": { "type": "integer", "minimum": 0, "maximum": 100 },
    "error_message": { "type": "string" }
  }
}
```

**Implementation**:
- Query `analyses` table for `status` field
- If failed, include error message from `error_message` column
- Return current status for polling

### 6. analyze_file

**Purpose**: Trigger new analysis on existing file

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project_id": { "type": "string" },
    "file_id": { "type": "string" },
    "provider": { "type": "string", "enum": ["openrouter", "openai", "claude", "gemini"], "default": "openrouter" }
  },
  "required": ["project_id", "file_id"]
}
```

**Output**:
```json
{
  "type": "object",
  "properties": {
    "analysis_id": { "type": "string" },
    "status": { "type": "string", "enum": ["pending"] }
  }
}
```

**Implementation**:
- Validate `project_id` and `file_id` exist in database
- Create new `analyses` record with status "pending"
- Spawn async task calling `loglens_core::analyze_lines()`
- Return `analysis_id` immediately for polling
- Update status to "running" → "completed"/"failed" in background task

## Server Implementation Steps

### Phase 1: Core Server Structure

**File: `loglens-mcp/src/lib.rs`**
```rust
use mcp_sdk::{Server, handler::RequestHandler};
use std::sync::Arc;

pub struct McpServer {
    db: Database,
    config: Config,
}

pub async fn create_server(db: Database, config: Config) -> Result<McpServer> {
    Ok(McpServer { db, config })
}
```

**File: `loglens-mcp/src/server.rs`**
```rust
use mcp_sdk::{Tool, ToolHandler, transport::Transport};

impl McpServer {
    pub async fn start(&self, transport: Transport) -> Result<()> {
        let server = Server::new()
            .with_tool(self.list_projects_tool())
            .with_tool(self.get_project_tool())
            .with_tool(self.list_analyses_tool())
            .with_tool(self.get_analysis_tool())
            .with_tool(self.get_analysis_status_tool())
            .with_tool(self.analyze_file_tool());

        server.run(transport).await
    }

    fn list_projects_tool(&self) -> Tool {
        // Return Tool with schema + handler
    }

    // ... other tool constructors
}
```

### Phase 2: Tool Implementations

**File: `loglens-mcp/src/tools/projects.rs`**
```rust
use serde_json::Value;
use crate::Database;

pub async fn list_projects(db: &Database, params: Value) -> Result<Value> {
    let names: Option<Vec<String>> = params.get("names")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let mut query = String::from(
        "SELECT p.id, p.name, p.description,
         COUNT(DISTINCT f.id) as file_count,
         COUNT(DISTINCT a.id) as analysis_count,
         MAX(a.created_at) as last_analysis_date
         FROM projects p
         LEFT JOIN files f ON p.id = f.project_id
         LEFT JOIN analyses a ON p.id = a.project_id"
    );

    if let Some(names) = names {
        let placeholders = names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        query.push_str(&format!(" WHERE p.name IN ({})", placeholders));
    }

    query.push_str(" GROUP BY p.id");

    // Execute query, deserialize to Vec<Project>
    // Convert to serde_json::Value
}

pub async fn get_project(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())?;

    // Similar query but with WHERE p.id = ?
    // Return single project or 404 error
}
```

**File: `loglens-mcp/src/tools/analyses.rs`**
```rust
pub async fn list_analyses(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())?;
    let limit: i64 = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    let analyses = sqlx::query_as!(
        Analysis,
        "SELECT id, project_id, file_id, status, created_at, completed_at
         FROM analyses
         WHERE project_id = ?
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        project_id, limit, offset
    ).fetch_all(&db.pool).await?;

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM analyses WHERE project_id = ?",
        project_id
    ).fetch_one(&db.pool).await?;

    Ok(json!({ "analyses": analyses, "total": total }))
}

pub async fn get_analysis(db: &Database, params: Value) -> Result<Value> {
    let analysis_id: String = serde_json::from_value(params["analysis_id"].clone())?;

    let result = sqlx::query!(
        "SELECT a.*, ar.errors, ar.recommendations, ar.patterns, ar.performance_metrics
         FROM analyses a
         LEFT JOIN analysis_results ar ON a.id = ar.analysis_id
         WHERE a.id = ?",
        analysis_id
    ).fetch_one(&db.pool).await?;

    // Deserialize JSON columns and construct response
}

pub async fn get_analysis_status(db: &Database, params: Value) -> Result<Value> {
    let analysis_id: String = serde_json::from_value(params["analysis_id"].clone())?;

    let status = sqlx::query!(
        "SELECT id, status, error_message FROM analyses WHERE id = ?",
        analysis_id
    ).fetch_one(&db.pool).await?;

    Ok(json!({
        "id": status.id,
        "status": status.status,
        "progress": calculate_progress(&status.status),
        "error_message": status.error_message
    }))
}
```

**File: `loglens-mcp/src/tools/analyze.rs`**
```rust
pub async fn analyze_file(db: &Database, params: Value) -> Result<Value> {
    let project_id: String = serde_json::from_value(params["project_id"].clone())?;
    let file_id: String = serde_json::from_value(params["file_id"].clone())?;
    let provider: String = params.get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("openrouter")
        .to_string();

    // Validate project and file exist
    let file = sqlx::query!("SELECT * FROM files WHERE id = ? AND project_id = ?", file_id, project_id)
        .fetch_one(&db.pool).await?;

    // Create analysis record
    let analysis_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO analyses (id, project_id, file_id, status) VALUES (?, ?, ?, 'pending')",
        analysis_id, project_id, file_id
    ).execute(&db.pool).await?;

    // Spawn background analysis task
    let db_clone = db.clone();
    let file_path = file.path.clone();
    tokio::spawn(async move {
        run_analysis(&db_clone, &analysis_id, &file_path, &provider).await
    });

    Ok(json!({
        "analysis_id": analysis_id,
        "status": "pending"
    }))
}

async fn run_analysis(db: &Database, analysis_id: &str, file_path: &str, provider: &str) {
    // Update status to running
    sqlx::query!("UPDATE analyses SET status = 'running' WHERE id = ?", analysis_id)
        .execute(&db.pool).await.ok();

    // Call loglens_core::analyze_lines()
    let result = loglens_core::analyze_lines(file_path, provider).await;

    match result {
        Ok(analysis) => {
            // Store results in analysis_results table
            // Update status to completed
        }
        Err(e) => {
            // Update status to failed with error message
            sqlx::query!(
                "UPDATE analyses SET status = 'failed', error_message = ? WHERE id = ?",
                e.to_string(), analysis_id
            ).execute(&db.pool).await.ok();
        }
    }
}
```

### Phase 3: Transport Implementation

**File: `loglens-mcp/src/transport/mod.rs`**
```rust
use mcp_sdk::transport::{StdioServerTransport, SseServerTransport};

pub enum TransportType {
    Stdio,
    Http { port: u16 },
}

pub fn create_transport(transport_type: TransportType) -> Box<dyn Transport> {
    match transport_type {
        TransportType::Stdio => Box::new(StdioServerTransport::new()),
        TransportType::Http { port } => Box::new(SseServerTransport::new(port)),
    }
}
```

**File: `loglens-mcp/src/transport/stdio.rs`**
```rust
use mcp_sdk::transport::StdioServerTransport;

pub struct StdioTransport {
    inner: StdioServerTransport,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            inner: StdioServerTransport::new(),
        }
    }
}

// Wrapper implementation delegating to SDK transport
```

**File: `loglens-mcp/src/transport/http.rs`**
```rust
use mcp_sdk::transport::SseServerTransport;

pub struct HttpTransport {
    inner: SseServerTransport,
    port: u16,
}

impl HttpTransport {
    pub fn new(port: u16) -> Self {
        Self {
            inner: SseServerTransport::new(port),
            port,
        }
    }
}

// Wrapper implementation delegating to SDK transport
```

### Phase 4: CLI Integration

**File: `loglens-cli/src/main.rs`**
```rust
use clap::Parser;
use loglens_mcp::{create_server, TransportType, create_transport};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    mcp_server: bool,

    #[arg(long, value_enum)]
    transport: Option<TransportMode>,

    #[arg(long, default_value = "3000")]
    port: u16,

    // ... other CLI args
}

#[derive(Clone, ValueEnum)]
enum TransportMode {
    Stdio,
    Http,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.mcp_server {
        // Initialize database
        let db = Database::connect().await?;

        // Create server
        let server = create_server(db, Config::default()).await?;

        // Determine transport
        let transport_type = match cli.transport.unwrap_or(TransportMode::Stdio) {
            TransportMode::Stdio => TransportType::Stdio,
            TransportMode::Http => TransportType::Http { port: cli.port },
        };

        let transport = create_transport(transport_type);

        // Start server
        tracing::info!("Starting MCP server with transport: {:?}", transport_type);
        server.start(transport).await?;

        return Ok(());
    }

    // ... rest of CLI logic
}
```

### Phase 5: Schema Definitions

**File: `loglens-mcp/src/schema.rs`**
```rust
use serde_json::json;

pub fn list_projects_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "names": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional list of project names to filter by"
            }
        }
    })
}

pub fn get_project_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "project_id": { "type": "string" }
        },
        "required": ["project_id"]
    })
}

// ... similar functions for all other tools
```

## Usage Documentation

### STDIO Mode (Claude Desktop)

Add to Claude Desktop config (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "loglens": {
      "command": "loglens",
      "args": ["--mcp-server"]
    }
  }
}
```

Start server:
```bash
loglens --mcp-server
```

### HTTP Mode

Start HTTP server:
```bash
loglens --mcp-server --transport http --port 3000
```

Configure MCP client to connect to `http://localhost:3000/sse`

### Tool Usage Examples

**List all projects**:
```json
{
  "tool": "list_projects",
  "arguments": {}
}
```

**Filter projects by name**:
```json
{
  "tool": "list_projects",
  "arguments": {
    "names": ["my-app", "backend-service"]
  }
}
```

**Get project details**:
```json
{
  "tool": "get_project",
  "arguments": {
    "project_id": "abc123"
  }
}
```

**List analyses for project**:
```json
{
  "tool": "list_analyses",
  "arguments": {
    "project_id": "abc123",
    "limit": 10,
    "offset": 0
  }
}
```

**Get complete analysis**:
```json
{
  "tool": "get_analysis",
  "arguments": {
    "analysis_id": "xyz789"
  }
}
```

**Trigger new analysis**:
```json
{
  "tool": "analyze_file",
  "arguments": {
    "project_id": "abc123",
    "file_id": "file456",
    "provider": "openrouter"
  }
}
```

**Poll analysis status**:
```json
{
  "tool": "get_analysis_status",
  "arguments": {
    "analysis_id": "xyz789"
  }
}
```

## Implementation Checklist

### Core Server
- [ ] Create `loglens-mcp` crate in workspace
- [ ] Add MCP SDK dependency from GitHub
- [ ] Implement `McpServer` struct with database connection
- [ ] Implement `create_server()` function
- [ ] Register all 6 tools with server

### Tool Implementations
- [ ] `list_projects` with optional name filtering
- [ ] `get_project` with aggregated statistics
- [ ] `list_analyses` with pagination
- [ ] `get_analysis` with full report data
- [ ] `get_analysis_status` for polling
- [ ] `analyze_file` with async background execution

### Transport Layer
- [ ] STDIO transport wrapper for `StdioServerTransport`
- [ ] HTTP transport wrapper for `SseServerTransport`
- [ ] Transport factory function
- [ ] Graceful shutdown handling (SIGTERM/SIGINT)

### CLI Integration
- [ ] Add `--mcp-server` flag to `loglens-cli`
- [ ] Add `--transport` enum flag (stdio/http)
- [ ] Add `--port` flag for HTTP mode
- [ ] Route to MCP server on flag detection
- [ ] Initialize database before server start

### Schema & Validation
- [ ] JSON schemas for all 6 tools
- [ ] Input parameter validation
- [ ] Error responses for invalid inputs
- [ ] Proper error codes (404 for not found, 400 for invalid params)

### Quality Gates
- [ ] All tools execute successfully with valid inputs
- [ ] Invalid parameters return proper error responses
- [ ] STDIO mode works with MCP inspector tool
- [ ] HTTP mode accepts SSE connections
- [ ] Database queries use prepared statements (SQL injection safe)
- [ ] Concurrent requests handled properly
- [ ] Background analysis tasks update status correctly

## Key Design Principles

1. **Single Responsibility**: Each tool does one thing, delegates to existing `loglens-core` logic
2. **Transport Agnostic**: Core tool logic independent of STDIO vs HTTP transport
3. **Async First**: All I/O operations non-blocking using Tokio
4. **Extensible**: New tools added by implementing tool trait and registering with server
5. **Database Reuse**: Use existing `Database` struct, no duplication
6. **Error Handling**: Proper MCP error responses with context
7. **Type Safety**: Leverage SQLx compile-time query validation
