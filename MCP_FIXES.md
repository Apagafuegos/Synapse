# MCP Server Fixes and Implementation Notes

This document details the critical fixes applied to the LogLens MCP server implementation and provides guidance for future development.

## Critical Fixes Applied

### 1. Stdio Logging Contamination (CRITICAL)

**Problem**: The MCP server with stdio transport had logging calls that corrupted the JSON-RPC protocol.

**Impact**: Claude Desktop and other MCP clients would fail to parse responses due to log lines mixed with JSON-RPC messages.

**Solution**:
- Removed all `tracing::info!()`, `tracing::error!()`, `tracing::warn!()` calls from stdio code paths
- Conditionally disabled logging initialization in CLI when using stdio transport
- Added comments warning about stdio contamination

**Files Modified**:
- `loglens-mcp/src/server.rs` - Removed tracing from `start_stdio()`
- `loglens-mcp/src/transport/stdio.rs` - Removed all tracing calls
- `loglens-mcp/src/tools/analyze.rs` - Removed tracing from background tasks
- `loglens-cli/src/main.rs` - Conditional logging init

**Important Rules**:
```rust
// ❌ NEVER do this in stdio mode:
tracing::info!("Starting server...");
println!("Server ready");
eprintln!("Debug info");

// ✅ Stdio mode must output ONLY JSON-RPC:
// - Pure stdin reading
// - Pure stdout JSON-RPC writing
// - No logs, no prints, no debug output
```

---

### 2. Database Path Resolution (CRITICAL)

**Problem**: Database was created in the source repository directory instead of proper user data directories.

**Impact**: When installed via `cargo install`, the database would be created in wrong locations, causing permission issues and data loss on updates.

**Solution**:
- Implemented XDG Base Directory specification using `directories` crate
- Database now stored in OS-appropriate locations:
  - **Linux**: `~/.local/share/loglens/loglens.db`
  - **macOS**: `~/Library/Application Support/loglens/loglens.db`
  - **Windows**: `%APPDATA%\loglens\loglens.db`
- Environment variable override still supported: `LOGLENS_DATABASE_PATH`

**Files Modified**:
- `Cargo.toml` - Added `directories = "5.0"` dependency
- `loglens-core/src/db_path.rs` - Complete rewrite with XDG support
- `loglens-core/Cargo.toml` - Added directories dependency

**Testing**:
```bash
# Verify database location
loglens --mcp-server &
pid=$!
sleep 2

# Check where it was created
ls ~/.local/share/loglens/loglens.db  # Linux
ls ~/Library/Application\ Support/loglens/loglens.db  # macOS

kill $pid
```

---

### 3. Broken Query Functions (CRITICAL)

**Problem**:
- `get_analysis_by_id()` always returned `None` (stub implementation)
- `query_analyses()` used incorrect schema column names

**Impact**: MCP tools `get_analysis` and `query_analyses` were completely non-functional.

**Solution**:
- Implemented proper `get_analysis_by_id()` with real database queries
- Fixed `query_analyses()` to use correct schema:
  - `log_file_id` → `log_file_path`
  - `level_filter` → `level`
  - `created_at` → `started_at` (for ordering)
  - Removed non-existent `metadata` column workaround

**Files Modified**:
- `loglens-core/src/project/queries.rs`

**Database Schema** (from `loglens-web/migrations/20240101000001_initial.sql`):
```sql
CREATE TABLE analyses (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    log_file_id TEXT,
    analysis_type TEXT NOT NULL,
    provider TEXT NOT NULL,
    level_filter TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 0,
    result TEXT,
    error_message TEXT,
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (log_file_id) REFERENCES log_files(id) ON DELETE SET NULL
);
```

---

### 4. Duplicate MCP Implementation (WARNING)

**Problem**: Two separate MCP server implementations existed, causing confusion.

**Solution**: Deleted old implementation in `loglens-core/src/mcp_server/`, kept only `loglens-mcp/` crate.

**Files Modified**:
- Deleted: `loglens-core/src/mcp_server/` (entire directory)
- Modified: `loglens-core/src/lib.rs` (removed module reference)

---

## MCP Implementation Architecture

### Current Structure

```
loglens-mcp/
├── src/
│   ├── lib.rs              # Public API, Database wrapper, Config
│   ├── server.rs           # MCP ServerHandler implementation
│   ├── schema.rs           # JSON schemas for tool parameters
│   ├── validation.rs       # Parameter validation
│   ├── tools/
│   │   ├── mod.rs          # Tool exports
│   │   ├── projects.rs     # Project management tools
│   │   ├── analyses.rs     # Analysis query tools
│   │   └── analyze.rs      # Analysis execution tool
│   └── transport/
│       ├── mod.rs          # Transport factory
│       ├── stdio.rs        # Stdio transport (NO LOGGING!)
│       └── http.rs         # HTTP transport (logging OK)
└── tests/
    ├── mcp_tests.rs        # Basic unit tests
    └── integration_tests.rs # Integration tests
```

### Available MCP Tools

1. **list_projects** - List all registered projects
2. **get_project** - Get detailed project information by ID
3. **list_analyses** - List analyses for a project with pagination
4. **get_analysis** - Get complete analysis results by ID
5. **get_analysis_status** - Get analysis status for polling
6. **analyze_file** - Trigger new analysis on existing log file

---

## Claude Desktop Integration

### Configuration

Edit `~/.config/Claude/claude_desktop_config.json` (Linux) or equivalent:

```json
{
  "mcpServers": {
    "loglens": {
      "command": "/path/to/loglens",
      "args": ["--mcp-server", "--mcp-transport", "stdio"]
    }
  }
}
```

**Important**:
- Use absolute path to `loglens` binary
- Must specify `--mcp-transport stdio` explicitly
- Restart Claude Desktop after configuration changes

### Testing

1. **Start MCP Server**:
```bash
loglens --mcp-server --mcp-transport stdio
```

2. **Verify stdio output** (should be ONLY JSON-RPC):
```bash
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | \
  loglens --mcp-server --mcp-transport stdio

# Expected: Pure JSON response like:
# {"jsonrpc":"2.0","id":1,"result":{"tools":[...]}}

# ❌ Bad: Any lines like "Starting server..." or "INFO:"
```

3. **Test in Claude Desktop**:
- Open conversation
- Type command using LogLens tools
- Verify tools appear in MCP menu
- Test each tool's functionality

---

## Development Guidelines

### Adding New MCP Tools

1. **Create tool function** in appropriate file under `loglens-mcp/src/tools/`:
```rust
pub async fn my_new_tool(db: &Database, params: Value) -> Result<Value> {
    // Parse parameters
    let param = params["my_param"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing my_param"))?;

    // Query database
    let result = sqlx::query("SELECT * FROM table WHERE col = ?")
        .bind(param)
        .fetch_all(&db.pool)
        .await?;

    // Return result
    Ok(serde_json::json!({
        "success": true,
        "data": result
    }))
}
```

2. **Add schema** in `loglens-mcp/src/schema.rs`:
```rust
pub fn my_new_tool_schema() -> serde_json::Map<String, serde_json::Value> {
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), json!("object"));

    let mut props = serde_json::Map::new();
    props.insert("my_param".to_string(), json!({
        "type": "string",
        "description": "Parameter description"
    }));

    schema.insert("properties".to_string(), json!(props));
    schema.insert("required".to_string(), json!(["my_param"]));
    schema
}
```

3. **Register tool** in `server.rs` `list_tools()` and `call_tool()`.

4. **Add validation** in `validation.rs`.

5. **Write tests** in `tests/integration_tests.rs`.

---

### Stdio Transport Rules (CRITICAL)

**When using stdio transport, you MUST:**

1. ❌ **NEVER** use any of these:
   - `println!()`
   - `eprintln!()`
   - `tracing::info!()`, `tracing::error!()`, etc.
   - `log::info!()`, `log::error!()`, etc.
   - `dbg!()`
   - Any output to stdout/stderr

2. ✅ **ONLY** use:
   - Pure JSON-RPC messages on stdout
   - Database for logging (if needed)
   - File-based logging (if absolutely necessary)

3. **Testing stdio compliance**:
```bash
# This should output ONLY JSON, nothing else:
echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | \
  your-mcp-server | jq .

# If jq fails to parse, you have logging contamination!
```

---

### Database Schema Conventions

**Always use these column names**:
- `id` - Primary key (TEXT, UUID)
- `project_id` - Foreign key to projects
- `log_file_path` - Path to log file (not `log_file_id`)
- `provider` - AI provider name
- `level` - Log level filter (not `level_filter`)
- `status` - Analysis status (INTEGER: 0=pending, 1=running, 2=completed, 3=failed)
- `created_at` - Record creation timestamp
- `started_at` - Analysis start timestamp
- `completed_at` - Analysis completion timestamp
- `metadata` - JSON metadata (TEXT)

**When adding queries**:
1. Check actual schema in `loglens-web/migrations/`
2. Use exact column names
3. Test with real database
4. Never assume schema from comments

---

## Testing Checklist

Before merging MCP changes:

- [ ] `cargo test --all-features` passes
- [ ] `cargo check --all-features` passes with no errors
- [ ] Integration tests pass: `cargo test -p loglens-mcp --test integration_tests`
- [ ] Stdio output is pure JSON (no logging)
- [ ] Database created in correct OS location
- [ ] All 6 MCP tools work in Claude Desktop
- [ ] Query functions return real data
- [ ] Documentation updated

---

## Troubleshooting

### "Cannot parse JSON-RPC response"
**Cause**: Logging contamination in stdio transport
**Fix**: Remove all `tracing::*` calls from stdio code path

### "Database not found"
**Cause**: Looking in wrong directory
**Fix**: Check `get_database_path()` implementation, verify XDG directories

### "Analysis not found" (but it exists)
**Cause**: Query using wrong schema
**Fix**: Check migrations, use correct column names in queries

### "MCP tools not appearing in Claude"
**Cause**: Invalid JSON schema or tool registration
**Fix**: Verify schema format in `list_tools()`, test with `mcp-test` tool

---

## Future Improvements

1. **Async analysis progress streaming** - Stream progress updates via MCP notifications
2. **File watching** - Monitor log files for changes and auto-analyze
3. **Multi-database support** - Support PostgreSQL, MySQL for enterprise
4. **Tool result caching** - Cache frequently accessed analysis results
5. **Bulk operations** - Batch analyze multiple files efficiently
6. **WebSocket transport** - Alternative to stdio for web interfaces

---

## References

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [XDG Base Directory Spec](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [rmcp Rust SDK](https://docs.rs/rmcp/)
- [SQLx Documentation](https://docs.rs/sqlx/)
