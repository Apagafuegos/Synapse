# LogLens MCP Integration - Complete Architecture & Implementation Plan

## Executive Overview

This plan details a comprehensive Model Context Protocol (MCP) integration for LogLens, enabling deep workflow integration with developer codebases and LLM-powered analysis capabilities.

```
Architecture Flow:
┌─────────────────┐
│ Developer       │
│ Codebase        │
└────────┬────────┘
         │
         │ loglens init
         ▼
┌─────────────────────────────────────────┐
│ .loglens/ Project Configuration         │
├─────────────────────────────────────────┤
│ • config.toml                           │
│ • metadata.json (project_id, linkage)   │
│ • index.db (SQLite)                     │
│ • analyses/ (results storage)           │
└────────┬────────────────────────────────┘
         │
         │ bidirectional link
         ▼
┌─────────────────────────────────────────┐
│ ~/.config/loglens/projects.json        │
│ Global Project Registry                 │
└────────┬────────────────────────────────┘
         │
         │ MCP Protocol
         ▼
┌─────────────────────────────────────────┐
│ MCP Server (JSON-RPC)                   │
├─────────────────────────────────────────┤
│ Tools:                                  │
│ • add_log_file                          │
│ • get_analysis                          │
│ • query_analyses                        │
└────────┬────────────────────────────────┘
         │
         │ context injection
         ▼
┌─────────────────┐
│ LLM (Claude,    │
│ ChatGPT, etc)   │
└─────────────────┘
```

---

## Component 1: Project Initialization System

### Purpose
Bootstrap LogLens integration into existing software projects with automatic project type detection and configuration generation.

### CLI Command
```bash
loglens init [--path <project_path>]
```

### Directory Structure Created
```
<project_root>/
└── .loglens/
    ├── config.toml          # Project-specific settings
    ├── metadata.json        # Project ID and linkage data
    ├── index.db             # SQLite database
    ├── analyses/            # Analysis results storage
    │   └── <analysis_id>/   # Individual analysis directories
    └── logs/                # Optional log file cache
```

### Configuration Schema

**config.toml**
```toml
[project]
name = "my-project"
type = "rust"              # auto-detected: rust, java, python, node
root_path = "/absolute/path/to/project"
created_at = "2025-10-06T12:00:00Z"

[loglens]
auto_analyze = true
default_provider = "openrouter"
default_level = "ERROR"

[mcp]
enabled = true
server_port = 3000         # optional
```

**metadata.json**
```json
{
  "project_id": "uuid-v4-generated",
  "project_name": "my-project",
  "project_type": "rust",
  "root_path": "/absolute/path/to/project",
  "loglens_version": "0.1.0",
  "created_at": "2025-10-06T12:00:00Z",
  "last_updated": "2025-10-06T12:00:00Z",
  "linked_analyses": [],
  "git_remote": "https://github.com/user/repo.git"
}
```

### Database Schema (index.db)

```sql
-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    root_path TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_updated TIMESTAMP NOT NULL
);

-- Analyses tracking
CREATE TABLE analyses (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    log_file_path TEXT NOT NULL,
    status TEXT NOT NULL,              -- pending, completed, failed
    created_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    provider TEXT NOT NULL,
    level TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Analysis results storage
CREATE TABLE analysis_results (
    analysis_id TEXT PRIMARY KEY,
    summary TEXT,
    full_report TEXT,
    issues_found INTEGER,
    patterns_detected TEXT,            -- JSON array
    metadata TEXT,                     -- JSON object
    FOREIGN KEY (analysis_id) REFERENCES analyses(id)
);

-- Performance indexes
CREATE INDEX idx_analyses_project ON analyses(project_id);
CREATE INDEX idx_analyses_status ON analyses(status);
CREATE INDEX idx_analyses_created ON analyses(created_at DESC);
```

### Implementation Steps

1. **Project Type Detection**
   - Scan for language markers (Cargo.toml, pom.xml, package.json, requirements.txt)
   - Set project type in metadata
   - Use type for context-aware defaults

2. **Directory Creation**
   - Create `.loglens/` if not exists
   - Generate all required files
   - Initialize SQLite database with schema

3. **Configuration Generation**
   - Generate unique project_id (UUID v4)
   - Populate config.toml with sensible defaults
   - Create metadata.json with project information

4. **Validation**
   - Verify all files created successfully
   - Test database connectivity
   - Register project in global registry

---

## Component 2: Hard Link Persistence Layer

### Purpose
Establish permanent bidirectional association between software projects and LogLens configurations.

### Architecture

```
Bidirectional Reference:

┌──────────────────────────────────┐
│ Project Codebase                 │
│                                  │
│ .loglens/metadata.json           │
│ └─> project_id: "uuid-1"         │
└──────────┬───────────────────────┘
           │
           │ links to
           ▼
┌──────────────────────────────────┐
│ ~/.config/loglens/projects.json  │
│                                  │
│ "uuid-1": {                      │
│   "root_path": "/path/to/proj"   │
│   "loglens_config": ".../.loglens"│
│ }                                │
└──────────────────────────────────┘
```

### Global Project Registry

**Location:** `~/.config/loglens/projects.json`

**Schema:**
```json
{
  "projects": {
    "uuid-1": {
      "name": "my-project",
      "root_path": "/absolute/path/to/project",
      "loglens_config": "/absolute/path/to/project/.loglens",
      "last_accessed": "2025-10-06T12:00:00Z"
    },
    "uuid-2": {
      "name": "another-project",
      "root_path": "/different/path",
      "loglens_config": "/different/path/.loglens",
      "last_accessed": "2025-10-05T08:30:00Z"
    }
  }
}
```

### CLI Commands

```bash
# Create hard link for existing project
loglens link --project <path>

# Remove hard link (preserves .loglens/ directory)
loglens unlink --project <path>

# List all linked projects
loglens list-projects

# Validate all links and repair if needed
loglens validate-links
```

### Link Validation Logic

1. **Startup Validation**
   - Verify all registered projects exist on filesystem
   - Check `.loglens/metadata.json` matches registry
   - Remove stale entries for deleted projects

2. **Auto-Repair**
   - If project moved but project_id matches, update paths
   - Prompt user for manual intervention on conflicts
   - Log all validation actions

3. **Consistency Checks**
   - Bidirectional reference integrity
   - No orphaned database entries
   - Timestamp consistency

---

## Component 3: MCP Server Implementation

### Purpose
Expose LogLens capabilities to LLMs via standardized Model Context Protocol.

### Protocol Architecture

```
LLM Client
    │
    │ JSON-RPC 2.0
    ▼
┌─────────────────────────────────┐
│ MCP Server (LogLens)            │
├─────────────────────────────────┤
│ Tool 1: add_log_file            │
│ Tool 2: get_analysis            │
│ Tool 3: query_analyses          │
└─────────┬───────────────────────┘
          │
          │ async processing
          ▼
┌─────────────────────────────────┐
│ Analysis Engine                 │
│ • AI Provider Integration       │
│ • Pattern Detection             │
│ • Result Storage                │
└─────────────────────────────────┘
```

### Tool 1: add_log_file

**Purpose:** Add log file to project and trigger analysis

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "project_path": {
      "type": "string",
      "description": "Path to software project root (must contain .loglens/)"
    },
    "log_file_path": {
      "type": "string",
      "description": "Absolute or relative path to log file"
    },
    "level": {
      "type": "string",
      "enum": ["ERROR", "WARN", "INFO", "DEBUG"],
      "description": "Minimum log level to analyze"
    },
    "provider": {
      "type": "string",
      "enum": ["openrouter", "openai", "claude", "gemini"],
      "description": "AI provider for analysis"
    },
    "auto_analyze": {
      "type": "boolean",
      "default": true,
      "description": "Automatically trigger analysis"
    }
  },
  "required": ["project_path", "log_file_path"]
}
```

**Output Schema:**
```json
{
  "success": true,
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "status": "pending",
  "message": "Log file added and analysis started"
}
```

**Processing Flow:**
```
1. Validate project_path contains .loglens/
2. Resolve log_file_path (absolute or relative)
3. Load configuration from .loglens/config.toml
4. Generate unique analysis_id (UUID v4)
5. Insert into analyses table (status: pending)
6. If auto_analyze == true:
   ├─> Spawn async analysis task
   ├─> Run AI analysis pipeline
   ├─> Store results in analysis_results table
   └─> Update status to completed
7. Return analysis_id immediately (non-blocking)
```

### Tool 2: get_analysis

**Purpose:** Retrieve analysis results by ID

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "analysis_id": {
      "type": "string",
      "description": "UUID of the analysis to retrieve"
    },
    "project_path": {
      "type": "string",
      "description": "Optional: path to project for validation"
    },
    "format": {
      "type": "string",
      "enum": ["summary", "full", "structured"],
      "default": "summary",
      "description": "Level of detail to return"
    }
  },
  "required": ["analysis_id"]
}
```

**Output Schema (summary format):**
```json
{
  "success": true,
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "status": "completed",
  "log_file": "/path/to/app.log",
  "summary": "Analysis found 12 ERROR entries, 5 recurring patterns detected",
  "issues_found": 12,
  "patterns": [
    {"pattern": "NullPointerException", "count": 5},
    {"pattern": "Connection timeout", "count": 3},
    {"pattern": "Database deadlock", "count": 2}
  ],
  "created_at": "2025-10-06T12:00:00Z",
  "completed_at": "2025-10-06T12:01:30Z"
}
```

**Output Format Options:**
- **summary:** High-level overview with key metrics (shown above)
- **full:** Complete markdown/HTML report with all details
- **structured:** Comprehensive JSON with all data points

**Processing Flow:**
```
1. Query analysis_results table by analysis_id
2. If project_path provided, validate it matches
3. Check analysis status (pending/completed/failed)
4. Format response based on requested format:
   ├─> summary: High-level overview with key metrics
   ├─> full: Complete markdown/HTML report
   └─> structured: JSON with all data points
5. Handle errors gracefully (not found, pending, etc.)
```

### Tool 3: query_analyses

**Purpose:** Discover analyses without knowing specific IDs

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "project_path": {
      "type": "string",
      "description": "Filter by project"
    },
    "status": {
      "type": "string",
      "enum": ["pending", "completed", "failed"],
      "description": "Filter by status"
    },
    "limit": {
      "type": "integer",
      "default": 10,
      "description": "Maximum results to return"
    },
    "since": {
      "type": "string",
      "description": "ISO timestamp - analyses after this time"
    }
  }
}
```

**Use Cases:**
- "Show me all failed analyses for this project"
- "What are the 5 most recent completed analyses?"
- "Find all analyses from the last 24 hours"

### Server Implementation Details

**File Location:** `src/mcp_server/mod.rs` (extend existing)

**Key Functions:**
```rust
async fn handle_add_log_file(params: Value) -> Result<Value>
async fn handle_get_analysis(params: Value) -> Result<Value>
async fn handle_query_analyses(params: Value) -> Result<Value>
```

**Async Analysis Processing:**
- Use tokio task spawning for non-blocking analysis
- Database connection pooling with sqlx
- Proper error handling and recovery
- Status updates throughout analysis lifecycle

---

## Component 4: Indexation & Query System

### Purpose
Efficient tracking, storage, and retrieval of project-linked analyses.

### Database Operations

**Create Analysis Record:**
```rust
async fn create_analysis(
    db: &SqlitePool,
    project_id: &str,
    log_file_path: &str,
    provider: &str,
    level: &str,
) -> Result<String> {
    let analysis_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO analyses
         (id, project_id, log_file_path, status, created_at, provider, level)
         VALUES (?, ?, ?, 'pending', ?, ?, ?)",
        analysis_id, project_id, log_file_path,
        Utc::now(), provider, level
    )
    .execute(db)
    .await?;
    Ok(analysis_id)
}
```

**Store Analysis Results:**
```rust
async fn store_analysis_results(
    db: &SqlitePool,
    analysis_id: &str,
    summary: &str,
    full_report: &str,
    patterns: &[Pattern],
) -> Result<()> {
    let patterns_json = serde_json::to_string(patterns)?;

    // Store results
    sqlx::query!(
        "INSERT INTO analysis_results
         (analysis_id, summary, full_report, patterns_detected)
         VALUES (?, ?, ?, ?)",
        analysis_id, summary, full_report, patterns_json
    )
    .execute(db)
    .await?;

    // Update status
    sqlx::query!(
        "UPDATE analyses
         SET status = 'completed', completed_at = ?
         WHERE id = ?",
        Utc::now(), analysis_id
    )
    .execute(db)
    .await?;

    Ok(())
}
```

### Query Patterns

**Get All Analyses for Project:**
```sql
SELECT a.*, ar.summary, ar.issues_found, ar.patterns_detected
FROM analyses a
LEFT JOIN analysis_results ar ON a.id = ar.analysis_id
WHERE a.project_id = ?
ORDER BY a.created_at DESC
```

**Get Recent Analyses:**
```sql
SELECT * FROM analyses
ORDER BY created_at DESC
LIMIT ?
```

**Search by Log File Pattern:**
```sql
SELECT * FROM analyses
WHERE log_file_path LIKE ?
```

**Filter by Status and Time:**
```sql
SELECT * FROM analyses
WHERE status = ?
  AND created_at > ?
ORDER BY created_at DESC
```

---

## Implementation Phases

### Phase 1: Foundation
**Goal:** Basic project initialization and configuration

```
Tasks:
├── 1.1 Create .loglens/ directory structure builder
├── 1.2 Implement loglens init CLI command
├── 1.3 Add project type auto-detection logic
├── 1.4 Generate configuration file templates
├── 1.5 Set up SQLite database with schema
└── 1.6 Write unit tests for initialization
```

**Deliverables:**
- Working `loglens init` command
- All configuration files generated
- Database schema created and tested

**Key Files to Create/Modify:**
- `src/cli.rs` - Add init subcommand
- `src/project/mod.rs` - New module for project management
- `src/project/init.rs` - Initialization logic
- `src/project/config.rs` - Configuration handling
- `src/project/database.rs` - Database schema and setup

---

### Phase 2: Hard Link System
**Goal:** Persistent project-LogLens associations

```
Tasks:
├── 2.1 Implement global project registry
├── 2.2 Add loglens link command
├── 2.3 Add loglens unlink command
├── 2.4 Add loglens list-projects command
├── 2.5 Add loglens validate-links command
├── 2.6 Build validation and auto-repair logic
└── 2.7 Create migration path for existing projects
```

**Deliverables:**
- All link management CLI commands
- Bidirectional reference validation
- Auto-repair functionality

**Key Files to Create/Modify:**
- `src/project/registry.rs` - Global registry management
- `src/project/link.rs` - Link/unlink operations
- `src/project/validate.rs` - Validation and repair logic
- `src/cli.rs` - Add link-related subcommands

---

### Phase 3: MCP Server Extension
**Goal:** Expose analysis capabilities via MCP

```
Tasks:
├── 3.1 Extend existing MCP server structure
├── 3.2 Implement add_log_file tool handler
├── 3.3 Implement get_analysis tool handler
├── 3.4 Implement query_analyses tool handler
├── 3.5 Add async analysis task spawning
├── 3.6 Integrate database connection pooling
├── 3.7 Add comprehensive error handling
└── 3.8 Write MCP integration tests
```

**Deliverables:**
- Three functional MCP tools
- Async analysis processing
- Full JSON-RPC compliance

**Key Files to Create/Modify:**
- `src/mcp_server/mod.rs` - Extend with new tools
- `src/mcp_server/tools/add_log_file.rs` - New tool
- `src/mcp_server/tools/get_analysis.rs` - New tool
- `src/mcp_server/tools/query_analyses.rs` - New tool
- `src/mcp_server/async_analysis.rs` - Async processing
- `src/project/database.rs` - Add analysis operations

---

### Phase 4: Indexation & Query
**Goal:** Efficient analysis tracking and retrieval

```
Tasks:
├── 4.1 Implement create_analysis database function
├── 4.2 Implement store_analysis_results function
├── 4.3 Add query_by_project function
├── 4.4 Add query_by_status function
├── 4.5 Add query_by_timerange function
├── 4.6 Optimize database with proper indexing
└── 4.7 Add search and filter capabilities
```

**Deliverables:**
- Complete database operation layer
- Optimized query performance
- Full search functionality

**Key Files to Create/Modify:**
- `src/project/database.rs` - Add all query operations
- `src/project/models.rs` - Database models
- `src/project/queries.rs` - Complex query logic

---

### Phase 5: Integration & Testing
**Goal:** Production-ready system validation

```
Tasks:
├── 5.1 End-to-end workflow testing
├── 5.2 Performance testing with large projects
├── 5.3 Multi-project concurrency testing
├── 5.4 MCP client integration testing
├── 5.5 Documentation and usage examples
├── 5.6 Error recovery and edge case handling
└── 5.7 Integration with Claude Desktop / other MCP clients
```

**Deliverables:**
- Comprehensive test suite
- Performance benchmarks
- User documentation
- Working MCP client integration

**Key Files to Create/Modify:**
- `tests/integration/mcp_workflow.rs` - E2E tests
- `tests/integration/multi_project.rs` - Concurrency tests
- `tests/performance/analysis_bench.rs` - Performance tests
- `docs/MCP_USAGE.md` - User guide
- `docs/MCP_CLIENT_INTEGRATION.md` - Integration guide

---

## Usage Example Workflow

### Developer Workflow

```bash
# Step 1: Initialize LogLens in your project
cd /path/to/my-rust-project
loglens init

# Output:
# ✓ Detected Rust project (Cargo.toml found)
# ✓ Created .loglens/ directory
# ✓ Generated configuration files
# ✓ Initialized database
# ✓ Registered project in global registry
# Project 'my-rust-project' initialized successfully

# Step 2: Verify link was created
loglens list-projects

# Output:
# Linked LogLens Projects:
# ┌────────────────────┬─────────────────────────────┬──────────────┐
# │ Name               │ Path                        │ Last Access  │
# ├────────────────────┼─────────────────────────────┼──────────────┤
# │ my-rust-project    │ /path/to/my-rust-project    │ 2 mins ago   │
# └────────────────────┴─────────────────────────────┴──────────────┘

# Step 3: Start MCP server
loglens --mcp-server

# Output:
# LogLens MCP Server started
# Listening on stdio for JSON-RPC requests
# Tools available: analyze_logs, parse_logs, filter_logs,
#                  add_log_file, get_analysis, query_analyses
```

### LLM Interaction via MCP

**LLM Tool Call 1: Add Log File**
```json
{
  "tool": "add_log_file",
  "parameters": {
    "project_path": "/path/to/my-rust-project",
    "log_file_path": "target/debug/app.log",
    "level": "ERROR",
    "auto_analyze": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "status": "pending",
  "message": "Log file added and analysis started"
}
```

**LLM Tool Call 2: Get Analysis (after brief delay)**
```json
{
  "tool": "get_analysis",
  "parameters": {
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "format": "summary"
  }
}
```

**Response:**
```json
{
  "success": true,
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "status": "completed",
  "log_file": "target/debug/app.log",
  "summary": "Analysis found 12 ERROR entries across 3 recurring patterns. Primary issues: NullPointerException (5 occurrences), Connection timeout (3 occurrences), and Database deadlock (2 occurrences). The errors suggest a resource contention issue in the database layer and missing null checks in the user service module.",
  "issues_found": 12,
  "patterns": [
    {"pattern": "NullPointerException in UserService::get_profile", "count": 5},
    {"pattern": "Connection timeout to postgres://localhost:5432", "count": 3},
    {"pattern": "Database deadlock in transaction handler", "count": 2}
  ],
  "created_at": "2025-10-06T12:00:00Z",
  "completed_at": "2025-10-06T12:01:30Z"
}
```

**LLM Tool Call 3: Query Related Analyses**
```json
{
  "tool": "query_analyses",
  "parameters": {
    "project_path": "/path/to/my-rust-project",
    "status": "completed",
    "limit": 5
  }
}
```

**Response:**
```json
{
  "success": true,
  "analyses": [
    {
      "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
      "log_file": "target/debug/app.log",
      "status": "completed",
      "issues_found": 12,
      "created_at": "2025-10-06T12:00:00Z"
    },
    {
      "analysis_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "log_file": "logs/production.log",
      "status": "completed",
      "issues_found": 8,
      "created_at": "2025-10-05T14:23:00Z"
    }
  ],
  "total_count": 2
}
```

### LLM Context Enhancement

The LLM can now provide context-aware responses:

**User:** "Why is my application crashing?"

**LLM:** "Based on the log analysis I just performed (analysis ID: 550e8400...), I found 12 ERROR entries. The primary issue appears to be a NullPointerException in UserService::get_profile (5 occurrences). This suggests missing null checks when retrieving user profiles. Additionally, there are 3 connection timeout errors to your PostgreSQL database, which might indicate either network issues or the connection pool being exhausted under load."

---

## Technical Considerations

### Concurrency & Performance
- **Async/Await:** Use async/await throughout for non-blocking operations
- **Connection Pooling:** Database connection pooling with sqlx to handle multiple concurrent requests
- **Task Queue:** Analysis task queue to prevent resource exhaustion
- **File System Locking:** Proper locking for file system operations to prevent race conditions

### Error Handling
- **Graceful Degradation:** When projects moved/deleted, provide clear error messages
- **Retry Logic:** Transient failures (network, filesystem) get automatic retry with exponential backoff
- **Comprehensive Logging:** All errors logged with context for debugging
- **User-Friendly Messages:** Convert technical errors to actionable user guidance

### Security
- **Path Validation:** Validate all file paths to prevent directory traversal attacks
- **SQL Injection Prevention:** Use sqlx parameterized queries exclusively
- **MCP Server Access:** Restrict MCP server access with proper authentication if needed
- **Data Privacy:** No sensitive data (credentials, PII) in analysis responses
- **Sandboxing:** Run analysis tasks in isolated context

### Extensibility
- **Plugin Architecture:** Support for additional MCP tools via plugin system
- **Custom Analysis Types:** Allow users to register custom analysis patterns
- **Storage Backends:** Abstract storage layer to support backends beyond SQLite
- **API Versioning:** Version MCP API for backward compatibility as features evolve
- **Configuration Hooks:** Allow users to customize behavior via config hooks

### Database Considerations
- **SQLite Limitations:** Single-writer limitation acceptable for initial implementation
- **Migration Path:** Design schema to allow migration to PostgreSQL if needed
- **Backup Strategy:** Automatic backup of .loglens/index.db before schema changes
- **Vacuum Strategy:** Periodic VACUUM to reclaim space from deleted analyses
- **WAL Mode:** Enable Write-Ahead Logging for better concurrency

### MCP Protocol Compliance
- **Handshake:** Proper MCP handshake implementation with capability negotiation
- **Tool Discovery:** Support for tool listing and schema introspection
- **Error Codes:** Use standard MCP error codes for protocol violations
- **Streaming:** Support for streaming results if analysis takes long time
- **Cancellation:** Allow LLM to cancel in-progress analysis requests

---

## File Structure After Implementation

```
loglens/
├── src/
│   ├── project/                    # NEW: Project management module
│   │   ├── mod.rs                  # Module exports
│   │   ├── init.rs                 # Project initialization
│   │   ├── config.rs               # Configuration handling
│   │   ├── metadata.rs             # Metadata operations
│   │   ├── database.rs             # Database schema and operations
│   │   ├── registry.rs             # Global project registry
│   │   ├── link.rs                 # Link/unlink operations
│   │   ├── validate.rs             # Validation and repair
│   │   ├── models.rs               # Data models
│   │   └── queries.rs              # Complex database queries
│   │
│   ├── mcp_server/                 # EXTENDED: MCP server
│   │   ├── mod.rs                  # Extended with new tools
│   │   ├── tools/
│   │   │   ├── analyze_logs.rs     # Existing tool
│   │   │   ├── parse_logs.rs       # Existing tool
│   │   │   ├── filter_logs.rs      # Existing tool
│   │   │   ├── add_log_file.rs     # NEW: Add log file tool
│   │   │   ├── get_analysis.rs     # NEW: Get analysis tool
│   │   │   └── query_analyses.rs   # NEW: Query analyses tool
│   │   └── async_analysis.rs       # NEW: Async analysis processing
│   │
│   ├── cli.rs                      # EXTENDED: Add new subcommands
│   └── ... (existing files)
│
├── tests/
│   ├── integration/
│   │   ├── mcp_workflow.rs         # NEW: E2E MCP workflow tests
│   │   ├── multi_project.rs        # NEW: Multi-project tests
│   │   └── project_init.rs         # NEW: Initialization tests
│   │
│   └── performance/
│       └── analysis_bench.rs       # NEW: Performance benchmarks
│
├── docs/
│   ├── MCP_INTEGRATION_PLAN.md     # THIS DOCUMENT
│   ├── MCP_USAGE.md                # NEW: User guide
│   └── MCP_CLIENT_INTEGRATION.md   # NEW: Integration guide
│
└── migrations/                      # NEW: Database migrations
    └── 001_initial_schema.sql
```

---

## Dependencies to Add

**Cargo.toml additions:**
```toml
[dependencies]
# Existing dependencies...
uuid = { version = "1.6", features = ["v4", "serde"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
# Existing dev dependencies...
tempfile = "3.8"      # For testing with temporary directories
```

---

## Success Metrics

### Phase Completion Criteria

**Phase 1 Complete When:**
- `loglens init` creates valid .loglens/ directory
- All configuration files generated correctly
- Database schema created and queryable
- Unit tests pass with 90%+ coverage

**Phase 2 Complete When:**
- All link commands functional
- Global registry maintains consistency
- Validation repairs broken links automatically
- Integration tests pass

**Phase 3 Complete When:**
- All three MCP tools respond correctly
- Async analysis completes without blocking
- MCP protocol compliance verified
- Tool schemas validate correctly

**Phase 4 Complete When:**
- Database operations handle concurrent requests
- Query performance meets targets (<100ms for typical queries)
- Indexing provides expected speedup
- Data integrity maintained under load

**Phase 5 Complete When:**
- E2E tests pass with multiple projects
- Performance benchmarks meet targets
- Documentation complete and validated
- Integration with at least one MCP client verified

### Performance Targets

- **Project Initialization:** <500ms
- **Link Operations:** <100ms
- **Analysis Creation:** <50ms (excluding actual analysis)
- **Query Operations:** <100ms for typical queries
- **Analysis Retrieval:** <200ms for summary format
- **Concurrent Projects:** Support 10+ projects simultaneously

---

## Risk Mitigation

### Identified Risks

1. **Database Lock Contention**
   - Risk: SQLite single-writer limitation causes bottlenecks
   - Mitigation: Use WAL mode, implement retry logic, design for eventual PostgreSQL migration

2. **File System Race Conditions**
   - Risk: Concurrent access to .loglens/ directory
   - Mitigation: Proper file locking, atomic operations where possible

3. **Analysis Task Exhaustion**
   - Risk: Too many concurrent analyses overwhelm system
   - Mitigation: Implement task queue with configurable concurrency limit

4. **Backward Compatibility**
   - Risk: Schema changes break existing projects
   - Mitigation: Implement migration system, version schema, maintain compatibility layer

5. **MCP Protocol Changes**
   - Risk: MCP specification evolves, breaking compatibility
   - Mitigation: Version MCP tools, maintain protocol version negotiation

### Recovery Strategies

- **Database Corruption:** Automatic backup before schema changes, recovery from backup
- **Registry Corruption:** Rebuild from .loglens/ directories found on filesystem
- **Analysis Failures:** Mark as failed with detailed error, allow retry
- **Partial Initialization:** Cleanup on failure, rollback partial changes

---

## Future Enhancements

### Post-MVP Features

1. **Multi-Database Support**
   - PostgreSQL backend for high-concurrency scenarios
   - Distributed analysis across multiple machines

2. **Analysis Caching**
   - Cache identical log analysis results
   - Incremental analysis for updated log files

3. **Real-Time Analysis**
   - Watch log files for changes
   - Stream analysis updates to LLM

4. **Enhanced Query Capabilities**
   - Full-text search across analysis results
   - Pattern-based filtering
   - Time-series analysis

5. **Collaboration Features**
   - Share analysis results across team
   - Central registry for organization-wide projects

6. **Advanced MCP Tools**
   - `compare_analyses`: Compare two analysis results
   - `trend_analysis`: Identify trends across multiple analyses
   - `suggest_fixes`: AI-powered fix suggestions

---

## Conclusion

This comprehensive plan provides a complete roadmap for integrating LogLens with the Model Context Protocol, enabling LLMs to intelligently interact with log analysis capabilities. The phased approach ensures each component is fully validated before moving to the next, reducing risk and maintaining code quality.

The architecture supports future extensibility while delivering immediate value through the core MCP tools. By following this plan, LogLens will become a powerful tool in AI-assisted development workflows, providing context-aware log analysis directly within LLM conversations.
