# Comprehensive Plan: Fully Implement HTTP and STDIO MCP Transports

**Created**: 2025-10-10
**Status**: PENDING IMPLEMENTATION
**Objective**: Make both STDIO and HTTP MCP transports fully functional and production-ready

---

## Current State Analysis

### STDIO Transport Status: ⚠️ BROKEN
- **Error**: "connection closed: initialized request"
- **Root Cause**: `Arc::try_unwrap()` in `synapse-mcp/src/transport/stdio.rs:12` fails when Arc has multiple references
- **Issue**: Server doesn't properly maintain connection lifecycle
- **Location**: `synapse-mcp/src/transport/stdio.rs`

### HTTP Transport Status: ❌ PLACEHOLDER ONLY
- **Current**: Dummy SSE handler that sends "MCP server ready" message
- **Missing**: Complete SSE server implementation using rmcp's SSE transport
- **Location**: `synapse-mcp/src/transport/http.rs`

### RMCP Version
- Currently using: `rmcp = "0.2.1"` with features `["server", "transport-io"]`
- Need to add: `"transport-sse-server"` feature for HTTP/SSE support

---

## Implementation Plan

### PHASE 1: Update Dependencies ⚙️

**File**: `synapse-mcp/Cargo.toml`

**Changes Required**:

1. Update rmcp dependency to include SSE server transport:
```toml
rmcp = { workspace = true, features = ["server", "transport-io", "transport-sse-server"] }
```

2. Add tokio cancellation token support:
```toml
tokio-util = "0.7"
```

3. Verify axum and tower dependencies are sufficient for SSE routing

**Estimated Time**: 5 minutes

---

### PHASE 2: Completely Rewrite HTTP Transport 🔨

**File**: `synapse-mcp/src/transport/http.rs`

**ACTION**: **REPLACE ENTIRE FILE** with production implementation

#### Required Imports
```rust
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use std::time::Duration;
use crate::server::SynapseMcpHandler;
```

#### Implementation Structure

1. **SSE Server Configuration**:
   - Bind address: `0.0.0.0:{port}` for network access
   - SSE endpoint path: `/mcp/sse`
   - POST message endpoint: `/mcp/message`
   - Keep-alive interval: 15 seconds
   - Cancellation token for graceful shutdown

2. **Server Creation**:
```rust
pub async fn run_http_server(handler: Arc<SynapseMcpHandler>, port: u16) -> anyhow::Result<()> {
    // Create cancellation token
    let ct = CancellationToken::new();

    // Configure SSE server
    let sse_config = SseServerConfig {
        bind: format!("0.0.0.0:{}", port).parse()?,
        sse_path: "/mcp/sse".to_string(),
        post_path: "/mcp/message".to_string(),
        ct: ct.clone(),
        sse_keep_alive: Some(Duration::from_secs(15)),
    };

    // Create SSE server and get router
    let (sse_server, sse_router) = SseServer::new(sse_config);

    // Add MCP handler as service
    sse_server.with_service(move || {
        // Clone handler for each connection
        Arc::clone(&handler)
    });

    // Create final router
    let app = Router::new().merge(sse_router);

    // Start server
    tracing::info!("HTTP MCP server listening on port {}", port);

    // Listen for shutdown signal
    tokio::select! {
        result = sse_server.serve() => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down HTTP MCP server...");
            ct.cancel();
        }
    }

    Ok(())
}
```

3. **Connection Management**:
   - Automatic session management via SSE server
   - Keep-alive messages prevent connection timeouts
   - Graceful shutdown cancels all connections
   - Proper error handling and logging

4. **Remove Placeholder Code**:
   - Delete `HttpServerState` struct
   - Delete `sse_handler` function
   - Delete all placeholder comments

**Estimated Time**: 30 minutes

---

### PHASE 3: Fix STDIO Transport 🔧

**File**: `synapse-mcp/src/transport/stdio.rs`

**Current Code (BROKEN)**:
```rust
pub async fn run_stdio_server(handler: Arc<SynapseMcpHandler>) -> anyhow::Result<()> {
    let (stdin, stdout) = stdio();

    // THIS FAILS - Arc has multiple references
    let handler = Arc::try_unwrap(handler).map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?;

    let _service = serve_server(handler, (stdin, stdout)).await?;

    // THIS DOESN'T WORK - server never actually runs
    std::future::pending::<()>().await;

    Ok(())
}
```

**Fixed Implementation**:
```rust
use rmcp::{transport::io::stdio, service::serve_server};
use std::sync::Arc;
use crate::server::SynapseMcpHandler;

pub async fn run_stdio_server(handler: Arc<SynapseMcpHandler>) -> anyhow::Result<()> {
    // NO LOGGING - stdio transport requires pure JSON-RPC on stdout

    // Get stdio transport
    let (stdin, stdout) = stdio();

    // DON'T unwrap Arc - serve_server can work with Arc
    // Clone the handler for the service
    let handler_clone = (*handler).clone();

    // Create and run the service
    let service = serve_server(handler_clone, (stdin, stdout)).await?;

    // Actually run the service until completion or error
    service.run().await?;

    Ok(())
}
```

**Key Fixes**:
1. **Remove `Arc::try_unwrap()`**: Keep handler as Arc and clone the inner value
2. **Fix Service Lifecycle**: Call `service.run().await` instead of pending forever
3. **Proper Error Handling**: Let errors propagate naturally
4. **No Logging**: Maintain strict no-logging policy for stdio

**Estimated Time**: 15 minutes

---

### PHASE 4: Update Server Structure 🏗️

**File**: `synapse-mcp/src/server.rs`

**Verify Handler Trait Implementations**:

1. Ensure `SynapseMcpHandler` implements `Clone`:
```rust
#[derive(Clone)]
pub struct SynapseMcpHandler {
    server: Arc<McpServer>,
}
```

2. Update start methods if needed:
```rust
impl McpServer {
    pub async fn start_stdio(&self) -> anyhow::Result<()> {
        let handler = Arc::new(SynapseMcpHandler::new(Arc::new(self.clone())));
        create_and_run_transport(TransportType::Stdio, handler).await
    }

    pub async fn start_http(&self, port: u16) -> anyhow::Result<()> {
        let handler = Arc::new(SynapseMcpHandler::new(Arc::new(self.clone())));

        tracing::info!("Starting Synapse MCP server with HTTP transport on port {}", port);
        tracing::info!("Server name: {}", self.config.server_name);
        tracing::info!("Server version: {}", self.config.server_version);

        create_and_run_transport(TransportType::Http { port }, handler).await
    }
}
```

**Changes**:
- Verify Arc nesting is correct
- Add proper error context
- Ensure logging only happens in HTTP mode

**Estimated Time**: 10 minutes

---

### PHASE 5: Add Comprehensive Tests 🧪

**File**: `synapse-mcp/tests/transport_tests.rs` (NEW FILE)

**Test Structure**:

```rust
use synapse_mcp::{create_server, Config, Database};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

/// Test STDIO transport connection
#[tokio::test]
async fn test_stdio_transport_connection() {
    // Create test database
    let (db, _temp) = setup_test_db().await;
    let config = Config::default();
    let server = create_server(db, config).await.unwrap();

    // Test stdio connection lifecycle
    // Create pipe for stdio simulation
    // Send initialize request
    // Verify response
    // Send tool call
    // Verify response
}

/// Test HTTP/SSE transport connection
#[tokio::test]
async fn test_http_sse_transport() {
    // Start server on random port
    // Connect SSE client
    // Verify keep-alive messages
    // Send POST message
    // Verify response
}

/// Test concurrent HTTP connections
#[tokio::test]
async fn test_concurrent_http_clients() {
    // Start server
    // Connect multiple SSE clients
    // Verify session isolation
    // Send messages from different clients
    // Verify correct routing
}

/// Test tool invocation via STDIO
#[tokio::test]
async fn test_stdio_tool_invocation() {
    // Setup test database with projects
    // Create stdio transport
    // Send list_projects tool call
    // Verify JSON-RPC response format
    // Verify tool result content
}

/// Test graceful shutdown
#[tokio::test]
async fn test_graceful_shutdown() {
    // Start both transport servers
    // Connect clients
    // Send shutdown signal
    // Verify clean closure
    // Verify no errors
}
```

**Coverage Areas**:
1. STDIO transport lifecycle
2. HTTP/SSE transport lifecycle
3. Tool invocation through both transports
4. Error handling
5. Concurrent connections (HTTP only)
6. Graceful shutdown
7. JSON-RPC protocol compliance

**Estimated Time**: 30 minutes

---

### PHASE 6: Manual Testing & Validation ✅

**STDIO Transport Testing**:

```bash
# Test 1: Start STDIO server
synapse --mcp-server --mcp-transport stdio

# Test 2: Send initialize request (in separate terminal)
echo '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | synapse --mcp-server --mcp-transport stdio

# Test 3: Send tools/list request
echo '{"jsonrpc":"2.0","method":"tools/list","id":2}' | synapse --mcp-server --mcp-transport stdio
```

**HTTP Transport Testing**:

```bash
# Test 1: Start HTTP server
synapse --mcp-server --mcp-transport http --mcp-port 3001

# Test 2: Connect SSE client (separate terminal)
curl -N http://localhost:3001/mcp/sse

# Test 3: Send message via POST
curl -X POST http://localhost:3001/mcp/message \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'

# Test 4: Verify keep-alive messages (should see events every 15 seconds)
```

**Integration Testing**:
1. Start web dashboard
2. Start MCP server (both transports)
3. Verify database connections
4. Test all 6 tools
5. Monitor logs for errors

**Estimated Time**: 20 minutes

---

## Success Criteria

### STDIO Transport Must ✅

- [ ] Server starts without errors
- [ ] Accepts stdin/stdout connections immediately
- [ ] Processes JSON-RPC 2.0 messages correctly
- [ ] All 6 MCP tools work: `list_projects`, `get_project`, `list_analyses`, `get_analysis`, `get_analysis_status`, `analyze_file`
- [ ] Connection stays alive during operation
- [ ] No stdout contamination (pure JSON-RPC only)
- [ ] Graceful shutdown on Ctrl+C
- [ ] Proper error messages to stderr only

### HTTP Transport Must ✅

- [ ] Server starts on specified port without errors
- [ ] SSE endpoint `/mcp/sse` is accessible and returns event stream
- [ ] POST endpoint `/mcp/message` handles JSON-RPC messages
- [ ] Keep-alive messages sent every 15 seconds
- [ ] Multiple clients can connect simultaneously
- [ ] Session management works correctly (messages routed to correct client)
- [ ] All 6 MCP tools work correctly
- [ ] Proper logging to stderr/tracing (not stdout)
- [ ] Graceful shutdown cancels all connections cleanly

### Both Transports Must ✅

- [ ] JSON-RPC 2.0 protocol compliance
- [ ] Database connections work properly
- [ ] No memory leaks during extended operation
- [ ] Production-ready error handling
- [ ] Clear error messages for debugging
- [ ] No placeholder code or TODOs
- [ ] All functions fully implemented
- [ ] Comprehensive test coverage

---

## Implementation Order

1. **Phase 1**: Update Cargo.toml → 5 minutes
2. **Phase 2**: Rewrite HTTP transport → 30 minutes
3. **Phase 3**: Fix STDIO transport → 15 minutes
4. **Phase 4**: Update server.rs → 10 minutes
5. **Phase 5**: Write tests → 30 minutes
6. **Phase 6**: Manual testing → 20 minutes

**Total Estimated Time**: ~2 hours

---

## Technical References

### RMCP Documentation
- GitHub: https://github.com/4t145/rmcp
- Docs.rs: https://docs.rs/rmcp/latest/
- SSE Server Example: https://github.com/4t145/rmcp/tree/dev/examples/servers/src/axum.rs

### SSE Server Configuration
```rust
pub struct SseServerConfig {
    pub bind: SocketAddr,          // Bind address
    pub sse_path: String,           // SSE endpoint path
    pub post_path: String,          // POST message path
    pub ct: CancellationToken,      // Shutdown token
    pub sse_keep_alive: Option<Duration>,  // Keep-alive interval
}
```

### MCP Protocol Specification
- JSON-RPC 2.0 for all messages
- Initialize handshake required
- Tools list discovery
- Tool invocation with parameters
- Error responses with codes

---

## Key Requirements

### Code Quality Standards

- ✅ **NO PLACEHOLDERS**: Every function must be fully implemented
- ✅ **NO TODOs**: Complete, working code only
- ✅ **NO STUBS**: All endpoints must work correctly
- ✅ **NO MOCK RESPONSES**: Real implementations only
- ✅ **PRODUCTION READY**: Proper error handling, logging, graceful shutdown
- ✅ **TESTED**: Both manual and automated tests must pass
- ✅ **DOCUMENTED**: Clear code comments and error messages

### Transport-Specific Requirements

**STDIO**:
- Absolutely no logging to stdout (breaks protocol)
- Pure JSON-RPC messages only
- stderr for errors is acceptable
- Must work with Claude Desktop, Cursor, etc.

**HTTP**:
- Full SSE implementation with keep-alive
- Concurrent client support
- Session management
- Proper HTTP status codes
- CORS if needed for web clients

---

## Rollback Plan

If implementation fails:
1. Keep current code in `git stash`
2. Implement minimal fixes first (STDIO Arc issue)
3. Test incrementally
4. Roll back to last working state if needed

---

## Post-Implementation

### Documentation Updates
- [ ] Update `CLAUDE.md` with working MCP examples
- [ ] Update README with transport options
- [ ] Add troubleshooting guide
- [ ] Document Claude Desktop integration

### Future Enhancements
- [ ] WebSocket transport option
- [ ] TLS support for HTTP transport
- [ ] Authentication/authorization for HTTP
- [ ] Monitoring and metrics
- [ ] Rate limiting

---

**END OF PLAN**
