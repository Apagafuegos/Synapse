# Phase 3: MCP Server Extension - Complete Implementation Plan

## Executive Summary

Implementing 3 new MCP tools for project-linked log analysis: `add_log_file`, `get_analysis`, and `query_analyses`. This extends the existing MCP server to support persistent analysis tracking with async background processing.

## Architecture Overview

```
                 MCP Client (Claude Code)
                          |
                          | JSON-RPC 2.0
                          v
            +---------------------------+
            |   MCP Server (LogLens)    |
            +---------------------------+
            | - add_log_file            |
            | - get_analysis            |
            | - query_analyses          |
            +---------------------------+
                     |         |
           +---------+         +----------+
           |                              |
           v                              v
   +----------------+          +--------------------+
   | Async Analysis |          | Database Queries   |
   | Task Spawning  |          | (SQLite + sqlx)    |
   +----------------+          +--------------------+
           |                              |
           v                              v
   [Background Task]              [.loglens/index.db]
   - Read log file                - projects
   - Run AI analysis              - analyses
   - Store results                - analysis_results
   - Update status
```

## Prerequisites Verified

**Phase 1 & 2 Complete:**
- Database schema exists (projects, analyses, analysis_results tables)
- Models defined (Project, Analysis, AnalysisResult, Pattern)
- Project initialization system operational
- Global registry and link management functional

**Critical Issue Identified:**
- `project-management` feature NOT enabled by default
- Must be activated for new MCP tools to compile

---

## Implementation Sequence

### STEP 1: Foundation - Database Operations
**Dependencies:** None
**Validation:** `cargo test --features project-management`

#### 1.1 Modify Cargo.toml
```toml
[features]
default = ["full", "project-management"]  # Add project-management
mcp-server = ["rmcp", "schemars", "project-management"]  # Link dependency
```

#### 1.2 Create project/queries.rs (~400 lines)

**Database CRUD Functions:**

```rust
// Create new analysis record
pub async fn create_analysis(
    pool: &SqlitePool,
    project_id: String,
    log_file_path: String,
    provider: String,
    level: String,
) -> Result<String>

// Retrieve analysis with optional results
pub async fn get_analysis_by_id(
    pool: &SqlitePool,
    analysis_id: &str,
) -> Result<Option<(Analysis, Option<AnalysisResult>)>>

// Query analyses with filters
pub async fn query_analyses(
    pool: &SqlitePool,
    project_id: Option<&str>,
    status: Option<AnalysisStatus>,
    limit: Option<i64>,
    since: Option<DateTime<Utc>>,
) -> Result<Vec<Analysis>>

// Store analysis results
pub async fn store_analysis_results(
    pool: &SqlitePool,
    analysis_id: &str,
    summary: Option<String>,
    full_report: Option<String>,
    patterns: Vec<Pattern>,
    issues_found: Option<i64>,
) -> Result<()>

// Update analysis status
pub async fn update_analysis_status(
    pool: &SqlitePool,
    analysis_id: &str,
    status: AnalysisStatus,
    completed_at: Option<DateTime<Utc>>,
) -> Result<()>

// Resolve project by path
pub async fn get_project_by_path(
    pool: &SqlitePool,
    root_path: &str,
) -> Result<Option<Project>>
```

**Unit Tests:**
- `test_create_analysis()` - Verify analysis record creation
- `test_get_analysis_by_id()` - Retrieve with JOIN on results
- `test_query_analyses_with_filters()` - Test all filter combinations
- `test_store_analysis_results()` - Verify result storage and patterns JSON
- `test_update_analysis_status()` - Status transitions and timestamps

#### 1.3 Update project/mod.rs
```rust
#[cfg(feature = "project-management")]
pub mod queries;
```

---

### STEP 2: Error Handling
**Dependencies:** Step 1
**Validation:** `cargo check`

#### 2.1 Extend mcp_server/error.rs

**New Error Variants:**
```rust
#[derive(Error, Debug)]
pub enum McpError {
    // ... existing variants ...

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid project path: {0}")]
    InvalidProjectPath(String),

    #[error("Analysis not found: {0}")]
    AnalysisNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}
```

**Error Conversions:**
```rust
impl From<sqlx::Error> for McpError {
    fn from(err: sqlx::Error) -> Self {
        McpError::DatabaseError(err.to_string())
    }
}
```

---

### STEP 3: Async Analysis Engine
**Dependencies:** Steps 1, 2
**Validation:** `cargo test --features project-management,mcp-server`

#### 3.1 Create mcp_server/async_analysis.rs (~200 lines)

**Core Function:**
```rust
pub async fn spawn_analysis_task(
    pool: SqlitePool,
    analysis_id: String,
    log_file_path: PathBuf,
    provider: String,
    level: String,
    api_key: Option<String>,
) -> Result<()> {
    tokio::spawn(async move {
        // Workflow implementation
    });
    Ok(())
}
```

**Background Workflow:**
1. Read log file from disk (tokio::fs::read_to_string)
2. Create LogLens instance
3. Run analyze_lines() with specified provider and level
4. Parse JSON report to extract metadata
5. Call store_analysis_results() with extracted data
6. Update status to Completed or Failed
7. Log all errors with context

**Error Handling:**
- Catch all errors in spawned task
- Mark analysis as Failed on any error
- Store error message in metadata
- Log detailed error information

---

### STEP 4: MCP Tool Handlers
**Dependencies:** Steps 1-3
**Validation:** `cargo test --features mcp-server`

#### 4.1 Tool Schemas (in list_tools())

**add_log_file Schema:**
```json
{
  "type": "object",
  "properties": {
    "project_path": {"type": "string", "description": "..."},
    "log_file_path": {"type": "string", "description": "..."},
    "level": {"type": "string", "enum": ["ERROR", "WARN", "INFO", "DEBUG"], "default": "ERROR"},
    "provider": {"type": "string", "enum": ["openrouter", "openai", "claude", "gemini"], "default": "openrouter"},
    "auto_analyze": {"type": "boolean", "default": true},
    "api_key": {"type": "string"}
  },
  "required": ["project_path", "log_file_path"]
}
```

**get_analysis Schema:**
```json
{
  "type": "object",
  "properties": {
    "analysis_id": {"type": "string", "description": "UUID of analysis"},
    "project_path": {"type": "string", "description": "Optional validation"},
    "format": {"type": "string", "enum": ["summary", "full", "structured"], "default": "summary"}
  },
  "required": ["analysis_id"]
}
```

**query_analyses Schema:**
```json
{
  "type": "object",
  "properties": {
    "project_path": {"type": "string"},
    "status": {"type": "string", "enum": ["pending", "completed", "failed"]},
    "limit": {"type": "integer", "default": 10},
    "since": {"type": "string", "description": "ISO timestamp"}
  }
}
```

#### 4.2 Helper Functions

**validate_project_path():**
- Resolve to absolute path
- Verify .loglens/ directory exists
- Verify metadata.json exists
- Return validated PathBuf or McpError

**format_analysis_response():**
- summary: High-level overview (id, status, summary, patterns, counts)
- full: Complete data (analysis + result objects)
- structured: Comprehensive JSON with all fields

#### 4.3 Tool Handlers

**handle_add_log_file:**
1. Parse arguments (project_path, log_file_path, level, provider, auto_analyze, api_key)
2. Validate project path
3. Resolve log file path (absolute or relative to project)
4. Open database pool
5. Get or create project record
6. Create analysis record with status=pending
7. If auto_analyze: spawn_analysis_task()
8. Return analysis_id and status

**handle_get_analysis:**
1. Parse arguments (analysis_id, project_path, format)
2. Open database pool
3. Get analysis + result by ID
4. If project_path provided: validate it matches
5. Format response based on format parameter
6. Return formatted JSON

**handle_query_analyses:**
1. Parse arguments (project_path, status, limit, since)
2. Open database pool
3. If project_path: resolve to project_id
4. Build query with filters
5. Execute with pagination
6. Return array of analysis summaries

#### 4.4 Update list_tools() and call_tool()

**list_tools():**
- Add 3 new Tool definitions with schemas
- Total: 6 tools (3 existing + 3 new)

**call_tool():**
```rust
match request.name.as_ref() {
    "analyze_logs" => self.handle_analyze_logs(request.arguments).await,
    "parse_logs" => self.handle_parse_logs(request.arguments).await,
    "filter_logs" => self.handle_filter_logs(request.arguments).await,
    "add_log_file" => self.handle_add_log_file(request.arguments).await,
    "get_analysis" => self.handle_get_analysis(request.arguments).await,
    "query_analyses" => self.handle_query_analyses(request.arguments).await,
    _ => Err(McpError::method_not_found())
}
```

---

### STEP 5: Integration Testing
**Dependencies:** All previous steps
**Validation:** `cargo test --all-features`

#### 5.1 Create tests/mcp_integration.rs (~300 lines)

**Test Scenarios:**

```rust
#[tokio::test]
async fn test_add_log_file_workflow() {
    // Setup: Create temp project with .loglens/, initialize DB
    // Execute: Call add_log_file via MCP
    // Verify: Analysis created with pending status
    // Wait: For async analysis to complete
    // Verify: Results stored, status=completed
}

#[tokio::test]
async fn test_get_analysis_formats() {
    // Setup: Create completed analysis with results
    // Test: Request with format=summary
    // Test: Request with format=full
    // Test: Request with format=structured
    // Verify: Each format returns appropriate data
}

#[tokio::test]
async fn test_query_analyses_filters() {
    // Setup: Create multiple analyses (different status, timestamps)
    // Test: Query by project_path
    // Test: Query by status filter
    // Test: Query with limit pagination
    // Test: Query with since timestamp
    // Verify: Correct filtering and ordering
}

#[tokio::test]
async fn test_error_handling() {
    // Test: Invalid project path
    // Test: Missing .loglens/ directory
    // Test: Nonexistent log file
    // Test: Invalid analysis_id
    // Verify: Appropriate errors returned
}
```

---

### STEP 6: Manual Validation
**Dependencies:** All implementation steps
**Validation:** Manual testing with MCP client

#### 6.1 Build and Test
```bash
# Build release binary
cargo build --release --all-features

# Start MCP server
./target/release/loglens --mcp-server
```

#### 6.2 Manual Test Cases

**Test 1: Project-Linked Analysis**
1. Initialize project: `loglens init --path /path/to/project`
2. Call add_log_file via MCP client
3. Verify analysis_id returned
4. Poll get_analysis until status=completed
5. Verify results contain patterns and summary

**Test 2: Query Workflow**
1. Create multiple analyses
2. Query by status=pending
3. Query by status=completed
4. Verify correct filtering

**Test 3: Output Formats**
1. Get analysis with format=summary
2. Get analysis with format=full
3. Get analysis with format=structured
4. Verify response sizes and content

---

## Success Criteria

### Functional Requirements
- [x] add_log_file creates analysis and spawns background task
- [x] get_analysis retrieves with 3 format options
- [x] query_analyses supports all filters (project, status, limit, since)
- [x] Async analysis completes and stores results
- [x] Status transitions: pending -> completed/failed
- [x] Project validation prevents invalid requests

### Performance Targets
- Analysis creation: < 50ms (excluding actual analysis)
- Query operations: < 100ms for typical queries
- Analysis retrieval: < 200ms for summary format

### Quality Standards
- Unit tests for all database operations
- Integration tests for complete MCP workflow
- Error cases properly handled
- Comprehensive logging at appropriate levels
- Feature flags correctly applied

---

## File Manifest

**New Files:**
- `loglens-core/src/project/queries.rs` (~400 lines)
- `loglens-core/src/mcp_server/async_analysis.rs` (~200 lines)
- `loglens-core/tests/mcp_integration.rs` (~300 lines)

**Modified Files:**
- `loglens-core/Cargo.toml` (+2 lines)
- `loglens-core/src/project/mod.rs` (+3 lines)
- `loglens-core/src/mcp_server/error.rs` (+25 lines)
- `loglens-core/src/mcp_server/mod.rs` (+600 lines)

**Total:** ~1,530 lines of new/modified code

---

## Implementation Details

### Project Path Validation Logic

```rust
fn validate_project_path(path: &str) -> Result<PathBuf, McpError> {
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
```

### Log File Path Resolution

```rust
fn resolve_log_file_path(project_path: &Path, log_file: &str) -> Result<PathBuf, McpError> {
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
```

### Analysis Response Formatting

```rust
fn format_analysis_response(
    analysis: Analysis,
    result: Option<AnalysisResult>,
    format: &str,
) -> Result<serde_json::Value, McpError> {
    match format {
        "summary" => {
            // Return high-level overview
            json!({
                "success": true,
                "analysis_id": analysis.id,
                "project_id": analysis.project_id,
                "status": analysis.status.to_string(),
                "log_file": analysis.log_file_path,
                "summary": result.as_ref().and_then(|r| r.summary.clone()),
                "issues_found": result.as_ref().and_then(|r| r.issues_found),
                "patterns": result.as_ref().map(|r| &r.patterns_detected).unwrap_or(&vec![]),
                "created_at": analysis.created_at,
                "completed_at": analysis.completed_at,
            })
        },
        "full" => {
            // Return complete report
            json!({
                "success": true,
                "analysis": analysis,
                "result": result,
            })
        },
        "structured" => {
            // Return comprehensive JSON
            json!({
                "success": true,
                "analysis_id": analysis.id,
                "project_id": analysis.project_id,
                "status": analysis.status.to_string(),
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
            })
        },
        _ => Err(McpError::InvalidInput(format!("Invalid format: {}", format)))
    }
}
```

### Async Analysis Task Error Handling

```rust
async fn spawn_analysis_task(
    pool: SqlitePool,
    analysis_id: String,
    log_file_path: PathBuf,
    provider: String,
    level: String,
    api_key: Option<String>,
) -> Result<()> {
    tokio::spawn(async move {
        let result = async {
            // Read log file
            let log_content = tokio::fs::read_to_string(&log_file_path).await?;
            let log_lines: Vec<String> = log_content.lines().map(|s| s.to_string()).collect();

            // Run analysis
            let loglens = LogLens::new()?;
            let report = loglens.generate_full_report(
                log_lines, &level, &provider, api_key.as_deref(),
                "mcp_async", OutputFormat::Json
            ).await?;

            // Parse report to extract metadata
            // ... extraction logic ...

            Ok::<_, anyhow::Error>((summary, full_report, patterns, issues))
        }.await;

        match result {
            Ok((summary, full_report, patterns, issues)) => {
                // Store results
                if let Err(e) = store_analysis_results(&pool, &analysis_id, summary, full_report, patterns, issues).await {
                    error!("Failed to store results: {}", e);
                    let _ = update_analysis_status(&pool, &analysis_id, AnalysisStatus::Failed, Some(Utc::now())).await;
                    return;
                }

                // Mark completed
                if let Err(e) = update_analysis_status(&pool, &analysis_id, AnalysisStatus::Completed, Some(Utc::now())).await {
                    error!("Failed to update status: {}", e);
                }
            },
            Err(e) => {
                error!("Analysis failed: {}", e);
                let _ = update_analysis_status(&pool, &analysis_id, AnalysisStatus::Failed, Some(Utc::now())).await;
            }
        }
    });

    Ok(())
}
```

---

## Risk Mitigation & Performance Considerations

### Database Connection Management
**Challenge**: Multiple MCP tools accessing database simultaneously
**Solution**: Use connection pooling (already configured with max 5 connections)
**Implementation**: Each tool handler opens pool, executes query, closes

### Concurrent Analysis Limit
**Challenge**: Too many concurrent analyses could overwhelm system
**Solution**: Implement task queue with configurable concurrency limit (future enhancement)

### Log File Size Handling
**Challenge**: Very large log files could cause OOM
**Solution**: Use existing slimmer module to reduce log volume
**Implementation**: In async_analysis.rs, call slim_logs() before analysis

### Database Lock Contention
**Challenge**: SQLite write locks during high concurrency
**Solution**: WAL mode enabled, retry logic for SQLITE_BUSY errors

### API Key Security
**Challenge**: API keys passed through MCP protocol
**Solution**: Support both parameter and environment variable, prefer env vars

### Analysis Result Size
**Challenge**: Full reports can be very large
**Solution**: Support format options (summary/full/structured) to control response size
**Implementation**: Default to summary format

---

## Notes

This implementation plan was generated on 2025-10-06 based on the MCP Integration Plan (docs/MCP_INTEGRATION_PLAN.md) Phase 3 requirements. All prerequisites from Phase 1 and Phase 2 have been verified as complete.

**Planning Session ID:** b9604cd2-8055-4616-9822-fdc24a478706
