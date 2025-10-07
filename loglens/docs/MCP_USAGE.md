# LogLens MCP Usage Guide

**Complete guide to using LogLens Model Context Protocol integration for AI-powered log analysis**

## Table of Contents
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Basic Workflow](#basic-workflow)
- [MCP Tools Reference](#mcp-tools-reference)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [Performance Tips](#performance-tips)

---

## Quick Start

### 5-Minute Setup

```bash
# 1. Initialize LogLens in your project
cd /path/to/your/project
loglens init

# 2. Start MCP server
loglens --mcp-server

# 3. Connect your MCP client (Claude Desktop, etc.)
# See MCP_CLIENT_INTEGRATION.md for client setup
```

### First Analysis

```json
{
  "tool": "add_log_file",
  "parameters": {
    "project_path": "/path/to/your/project",
    "log_file_path": "logs/application.log",
    "level": "ERROR",
    "auto_analyze": true
  }
}
```

---

## Installation

### Prerequisites

- **Rust**: 1.70 or later
- **Operating System**: Linux, macOS, or Windows
- **Disk Space**: 50MB for LogLens + variable for analysis storage

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/loglens.git
cd loglens

# Build with MCP features
cargo build --release --features "project-management,mcp-server"

# Install binary
cargo install --path . --features "project-management,mcp-server"
```

### Verify Installation

```bash
loglens --version
loglens --help
```

---

## Basic Workflow

### Step 1: Project Initialization

Initialize LogLens in your software project:

```bash
cd /path/to/your/project
loglens init
```

**What happens:**
- Creates `.loglens/` directory with configuration
- Generates unique project ID
- Initializes SQLite database for analysis tracking
- Registers project in global registry (`~/.config/loglens/projects.json`)

**Output:**
```
✓ Detected Rust project (Cargo.toml found)
✓ Created .loglens/ directory
✓ Generated configuration files
✓ Initialized database
✓ Registered project in global registry
Project 'your-project' initialized successfully
Project ID: 550e8400-e29b-41d4-a716-446655440000
```

### Step 2: Verify Setup

```bash
# List all linked projects
loglens list-projects

# Validate project configuration
loglens validate-links
```

### Step 3: Start MCP Server

```bash
# Start server (listens on stdio)
loglens --mcp-server

# Or with logging enabled
RUST_LOG=info loglens --mcp-server
```

**Expected output:**
```
LogLens MCP Server started
Protocol Version: 2024-11-05
Listening on stdio for JSON-RPC requests
Tools available: 6 (analyze_logs, parse_logs, filter_logs, add_log_file, get_analysis, query_analyses)
```

### Step 4: Connect MCP Client

Connect your MCP-compatible client (Claude Desktop, custom client, etc.) to the running server.

See **[MCP_CLIENT_INTEGRATION.md](./MCP_CLIENT_INTEGRATION.md)** for detailed client setup instructions.

### Step 5: Analyze Logs

Once connected, use MCP tools through your client to analyze logs.

---

## MCP Tools Reference

LogLens provides **6 MCP tools** for log analysis and management:

### 1. analyze_logs

**Purpose:** Analyze log content directly (Phase 1-2 tool)

**Parameters:**
```json
{
  "logs": ["[ERROR] Message 1", "[WARN] Message 2"],
  "level": "ERROR",
  "provider": "openrouter",
  "api_key": "your-api-key"
}
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| logs | array[string] | Yes | Log entries to analyze |
| level | string | No | Min level (ERROR/WARN/INFO/DEBUG) |
| provider | string | No | AI provider (default: openrouter) |
| api_key | string | No | API key for provider |

**Response:**
```json
{
  "summary": "Analysis found 12 ERROR entries...",
  "patterns": [
    {"pattern": "NullPointerException", "count": 5}
  ],
  "recommendations": ["Add null checks in UserService"]
}
```

### 2. parse_logs

**Purpose:** Parse log entries into structured format

**Parameters:**
```json
{
  "logs": ["[ERROR] 2025-01-06 Message"],
  "format": "standard"
}
```

**Response:**
```json
{
  "entries": [
    {
      "timestamp": "2025-01-06T12:00:00Z",
      "level": "ERROR",
      "message": "Message"
    }
  ]
}
```

### 3. filter_logs

**Purpose:** Filter log entries by level or pattern

**Parameters:**
```json
{
  "logs": ["[ERROR] Msg", "[INFO] Msg"],
  "level": "ERROR",
  "pattern": "database"
}
```

### 4. add_log_file ⭐ (Phase 3)

**Purpose:** Add log file to project and trigger analysis

**Parameters:**
```json
{
  "project_path": "/absolute/path/to/project",
  "log_file_path": "logs/app.log",
  "level": "ERROR",
  "provider": "openrouter",
  "auto_analyze": true,
  "api_key": "your-key"
}
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| project_path | string | Yes | - | Path to project root (must contain .loglens/) |
| log_file_path | string | Yes | - | Path to log file (absolute or relative to project) |
| level | string | No | ERROR | Minimum log level to analyze |
| provider | string | No | openrouter | AI provider for analysis |
| auto_analyze | boolean | No | true | Auto-trigger analysis |
| api_key | string | No | (from env) | API key for provider |

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

**Workflow:**
1. Validates project has `.loglens/` directory
2. Resolves log file path (absolute or relative)
3. Creates analysis record in database
4. Spawns async analysis task (if auto_analyze=true)
5. Returns immediately with analysis_id

**Example Usage:**

```javascript
// From LLM/MCP client
const result = await mcp.callTool("add_log_file", {
  project_path: "/home/user/myapp",
  log_file_path: "target/debug/app.log",
  level: "ERROR"
});

console.log(`Analysis ID: ${result.analysis_id}`);
// Analysis ID: 550e8400-e29b-41d4-a716-446655440000
```

### 5. get_analysis ⭐ (Phase 3)

**Purpose:** Retrieve analysis results by ID

**Parameters:**
```json
{
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_path": "/absolute/path/to/project",
  "format": "summary"
}
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| analysis_id | string (UUID) | Yes | - | Analysis ID from add_log_file |
| project_path | string | Yes | - | Project path for database lookup |
| format | string | No | summary | Output format (summary/full/structured) |

**Response Formats:**

**Summary** (default - concise overview):
```json
{
  "success": true,
  "analysis_id": "550e8400-...",
  "project_id": "6ba7b810-...",
  "status": "completed",
  "log_file": "/path/to/app.log",
  "summary": "Analysis found 12 ERROR entries, 5 recurring patterns detected",
  "issues_found": 12,
  "patterns": [
    {"pattern": "NullPointerException", "count": 5},
    {"pattern": "Connection timeout", "count": 3}
  ],
  "created_at": "2025-10-06T12:00:00Z",
  "completed_at": "2025-10-06T12:01:30Z"
}
```

**Full** (complete analysis report):
```json
{
  "success": true,
  "analysis_id": "550e8400-...",
  "status": "completed",
  "analysis": {
    "id": "550e8400-...",
    "project_id": "6ba7b810-...",
    "log_file": "/path/to/app.log",
    "provider": "openrouter",
    "level": "ERROR",
    "created_at": "2025-10-06T12:00:00Z",
    "completed_at": "2025-10-06T12:01:30Z"
  },
  "result": {
    "summary": "...",
    "full_report": "## Analysis Report\n\n### Key Issues\n...",
    "patterns": [...],
    "issues_found": 12
  }
}
```

**Structured** (all data in structured format):
```json
{
  "success": true,
  "analysis_id": "550e8400-...",
  "project_id": "6ba7b810-...",
  "log_file": "/path/to/app.log",
  "provider": "openrouter",
  "level": "ERROR",
  "status": "completed",
  "summary": "...",
  "full_report": "...",
  "patterns": [...],
  "issues_found": 12,
  "created_at": "2025-10-06T12:00:00Z",
  "completed_at": "2025-10-06T12:01:30Z"
}
```

**Status Values:**
- `pending`: Analysis in progress
- `completed`: Analysis finished successfully
- `failed`: Analysis encountered an error

**Example Usage:**

```javascript
// Wait for analysis to complete
await sleep(2000); // Give it time to analyze

// Get results
const results = await mcp.callTool("get_analysis", {
  analysis_id: "550e8400-e29b-41d4-a716-446655440000",
  project_path: "/home/user/myapp",
  format: "summary"
});

if (results.status === "completed") {
  console.log(`Found ${results.issues_found} issues`);
  results.patterns.forEach(p => {
    console.log(`- ${p.pattern}: ${p.count} occurrences`);
  });
}
```

### 6. query_analyses ⭐ (Phase 3)

**Purpose:** Query analyses with filters (discovery without specific IDs)

**Parameters:**
```json
{
  "project_path": "/absolute/path/to/project",
  "status": "completed",
  "limit": 10,
  "since": "2025-10-06T00:00:00Z"
}
```

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| project_path | string | Yes | - | Filter by project |
| status | string | No | (all) | Filter by status (pending/completed/failed) |
| limit | integer | No | 10 | Maximum results to return |
| since | string (ISO date) | No | (all time) | Analyses after this timestamp |

**Response:**
```json
{
  "success": true,
  "analyses": [
    {
      "analysis_id": "550e8400-...",
      "log_file": "/path/to/app.log",
      "status": "completed",
      "provider": "openrouter",
      "level": "ERROR",
      "created_at": "2025-10-06T12:00:00Z",
      "completed_at": "2025-10-06T12:01:30Z"
    }
  ],
  "total_count": 5
}
```

**Use Cases:**
- "Show me all failed analyses for this project"
- "What are the 5 most recent completed analyses?"
- "Find all analyses from the last 24 hours"
- "List pending analyses awaiting completion"

**Example Usage:**

```javascript
// Find recent failures
const failures = await mcp.callTool("query_analyses", {
  project_path: "/home/user/myapp",
  status: "failed",
  limit: 5
});

console.log(`Found ${failures.total_count} failed analyses`);

// Find analyses from last hour
const since = new Date(Date.now() - 3600000).toISOString();
const recent = await mcp.callTool("query_analyses", {
  project_path: "/home/user/myapp",
  since: since
});
```

---

## Advanced Usage

### Multiple Projects

LogLens supports managing multiple projects simultaneously:

```bash
# Initialize multiple projects
cd /path/to/project1 && loglens init
cd /path/to/project2 && loglens init
cd /path/to/project3 && loglens init

# List all projects
loglens list-projects

# Output:
# Linked LogLens Projects:
# ┌────────────────┬─────────────────────────┬──────────────┐
# │ Name           │ Path                    │ Last Access  │
# ├────────────────┼─────────────────────────┼──────────────┤
# │ project1       │ /path/to/project1       │ 2 mins ago   │
# │ project2       │ /path/to/project2       │ 5 mins ago   │
# │ project3       │ /path/to/project3       │ 1 hour ago   │
# └────────────────┴─────────────────────────┴──────────────┘
```

**MCP Usage:**

```javascript
// Analyze logs across multiple projects
const projects = [
  "/path/to/project1",
  "/path/to/project2",
  "/path/to/project3"
];

for (const project of projects) {
  const result = await mcp.callTool("add_log_file", {
    project_path: project,
    log_file_path: "logs/error.log",
    level: "ERROR"
  });

  console.log(`${project}: Analysis ${result.analysis_id}`);
}
```

### Concurrent Analysis

LogLens handles concurrent analyses efficiently:

```javascript
// Launch multiple analyses in parallel
const analyses = await Promise.all([
  mcp.callTool("add_log_file", {
    project_path: "/path/to/project",
    log_file_path: "logs/app1.log"
  }),
  mcp.callTool("add_log_file", {
    project_path: "/path/to/project",
    log_file_path: "logs/app2.log"
  }),
  mcp.callTool("add_log_file", {
    project_path: "/path/to/project",
    log_file_path: "logs/app3.log"
  })
]);

// Wait for all to complete
await sleep(5000);

// Check status of all
for (const analysis of analyses) {
  const result = await mcp.callTool("get_analysis", {
    analysis_id: analysis.analysis_id,
    project_path: "/path/to/project"
  });

  console.log(`${result.log_file}: ${result.status}`);
}
```

### Periodic Analysis

Set up periodic log analysis:

```javascript
// Analyze logs every 5 minutes
setInterval(async () => {
  const result = await mcp.callTool("add_log_file", {
    project_path: "/path/to/project",
    log_file_path: "logs/production.log",
    level: "ERROR"
  });

  // Wait for completion
  await sleep(3000);

  const analysis = await mcp.callTool("get_analysis", {
    analysis_id: result.analysis_id,
    project_path: "/path/to/project",
    format: "summary"
  });

  if (analysis.issues_found > 0) {
    console.log(`⚠️  Alert: ${analysis.issues_found} new errors detected`);
    // Send notification, create ticket, etc.
  }
}, 300000); // 5 minutes
```

### Custom Analysis Workflows

```javascript
async function analyzeAndReport(projectPath, logFile) {
  // Step 1: Add log file
  const addResult = await mcp.callTool("add_log_file", {
    project_path: projectPath,
    log_file_path: logFile,
    level: "ERROR"
  });

  console.log(`Started analysis: ${addResult.analysis_id}`);

  // Step 2: Poll for completion
  let status = "pending";
  while (status === "pending") {
    await sleep(1000);

    const result = await mcp.callTool("get_analysis", {
      analysis_id: addResult.analysis_id,
      project_path: projectPath,
      format: "summary"
    });

    status = result.status;
  }

  // Step 3: Get full results
  const fullResults = await mcp.callTool("get_analysis", {
    analysis_id: addResult.analysis_id,
    project_path: projectPath,
    format: "full"
  });

  // Step 4: Generate report
  console.log("\n=== Analysis Report ===");
  console.log(`Status: ${fullResults.status}`);
  console.log(`Issues Found: ${fullResults.result.issues_found}`);
  console.log(`\nTop Patterns:`);
  fullResults.result.patterns.forEach((p, i) => {
    console.log(`${i+1}. ${p.pattern} (${p.count} occurrences)`);
  });
  console.log(`\n${fullResults.result.summary}`);

  return fullResults;
}

// Usage
await analyzeAndReport("/path/to/project", "logs/app.log");
```

### Analysis History

```javascript
// Get analysis history for a project
async function getAnalysisHistory(projectPath, days = 7) {
  const since = new Date(Date.now() - days * 86400000).toISOString();

  const history = await mcp.callTool("query_analyses", {
    project_path: projectPath,
    since: since,
    limit: 100
  });

  // Group by status
  const byStatus = {
    completed: [],
    failed: [],
    pending: []
  };

  history.analyses.forEach(a => {
    byStatus[a.status].push(a);
  });

  console.log(`\n=== Analysis History (last ${days} days) ===`);
  console.log(`Total: ${history.total_count}`);
  console.log(`Completed: ${byStatus.completed.length}`);
  console.log(`Failed: ${byStatus.failed.length}`);
  console.log(`Pending: ${byStatus.pending.length}`);

  return byStatus;
}

// Usage
await getAnalysisHistory("/path/to/project", 7);
```

---

## Troubleshooting

### Common Issues

#### 1. "Project not initialized"

**Error:**
```
Error: Project path /path/to/project does not contain .loglens/ directory
```

**Solution:**
```bash
cd /path/to/project
loglens init
```

#### 2. "Analysis stuck in pending"

**Symptoms:** Analysis never completes, status remains "pending"

**Causes:**
- AI provider API key missing or invalid
- Network connectivity issues
- Large log file taking longer than expected

**Solutions:**
```bash
# Check API key is set
echo $OPENROUTER_API_KEY

# Enable debug logging
RUST_LOG=debug loglens --mcp-server

# Query analysis status
# Use get_analysis tool to check for error details
```

#### 3. "Database locked"

**Error:**
```
Error: database is locked
```

**Cause:** Multiple processes accessing SQLite database simultaneously

**Solution:**
- LogLens uses WAL mode to minimize locking
- Wait a moment and retry
- Check no other LogLens instances are running

#### 4. "Invalid analysis_id format"

**Error:**
```
Error: Invalid UUID format for analysis_id
```

**Solution:**
- Ensure analysis_id is a valid UUID v4
- Check for typos or truncation
- Use the exact ID returned by add_log_file

#### 5. "Log file not found"

**Error:**
```
Error: Log file not found: logs/app.log
```

**Solutions:**
- Use absolute path: `/full/path/to/logs/app.log`
- Or relative to project root: `logs/app.log` (from project directory)
- Verify file exists: `ls -la /path/to/logs/app.log`

### Debug Mode

Enable comprehensive logging:

```bash
# All modules
RUST_LOG=debug loglens --mcp-server

# Specific modules
RUST_LOG=loglens_core::mcp_server=debug loglens --mcp-server

# Multiple modules
RUST_LOG=loglens_core=debug,sqlx=info loglens --mcp-server
```

### Validation Commands

```bash
# Validate all project links
loglens validate-links

# Check database integrity
sqlite3 /path/to/project/.loglens/index.db "PRAGMA integrity_check;"

# List all analyses for a project
sqlite3 /path/to/project/.loglens/index.db "SELECT id, status, created_at FROM analyses;"
```

---

## Performance Tips

### 1. Database Optimization

**WAL Mode** (enabled by default):
- Allows concurrent readers during writes
- Improves performance under load

**Periodic Vacuum:**
```bash
# Clean up deleted records and reclaim space
sqlite3 /path/to/project/.loglens/index.db "VACUUM;"
```

### 2. Log File Size

**Best Practices:**
- Files < 10MB: Excellent performance
- Files 10-100MB: Good performance, may take longer
- Files > 100MB: Consider splitting or filtering

**Splitting Large Files:**
```bash
# Split into 10MB chunks
split -b 10M large.log split_log_

# Analyze each chunk
for file in split_log_*; do
  # Use add_log_file for each chunk
done
```

### 3. Concurrent Analyses

**Recommended Limits:**
- **Per Project:** 10-20 concurrent analyses
- **System-Wide:** 50-100 concurrent analyses

**Connection Pool** (configured automatically):
- Max 5 database connections per pool
- Connection timeout: 30 seconds

### 4. Analysis Caching

LogLens stores all analysis results permanently:
- Retrieve completed analyses instantly
- No re-analysis for same log file

### 5. API Provider Selection

**Performance Comparison:**

| Provider | Speed | Cost | Quality |
|----------|-------|------|---------|
| OpenRouter | Fast | Low | Good |
| Claude | Fast | Medium | Excellent |
| OpenAI | Medium | Medium | Excellent |
| Gemini | Fast | Low | Good |

**Tip:** Use `provider` parameter in `add_log_file` to select provider per analysis.

### 6. Level Filtering

**Impact:**
- `ERROR`: Fastest (fewest entries)
- `WARN`: Fast
- `INFO`: Medium
- `DEBUG`: Slowest (most entries)

**Recommendation:** Start with `ERROR`, expand to `WARN` if needed.

---

## Best Practices

### 1. Project Organization

```
your-project/
├── .loglens/           # LogLens directory
│   ├── config.toml
│   ├── metadata.json
│   ├── index.db
│   └── analyses/
├── logs/               # Your log files
│   ├── app.log
│   ├── error.log
│   └── access.log
└── [your project files]
```

### 2. Analysis Naming

Use descriptive log file names for easy identification:

```bash
# Good
logs/production-error-2025-10-06.log
logs/staging-debug-auth-module.log

# Less descriptive
logs/log.txt
logs/output.log
```

### 3. Retention Policy

Clean up old analyses periodically:

```sql
-- Delete analyses older than 30 days
sqlite3 /path/to/project/.loglens/index.db \
  "DELETE FROM analyses WHERE created_at < datetime('now', '-30 days');"

-- Vacuum to reclaim space
sqlite3 /path/to/project/.loglens/index.db "VACUUM;"
```

### 4. Error Handling

Always handle analysis errors:

```javascript
try {
  const result = await mcp.callTool("add_log_file", {
    project_path: "/path/to/project",
    log_file_path: "logs/app.log"
  });

  // Poll for completion with timeout
  const maxWait = 60000; // 60 seconds
  const startTime = Date.now();

  while (Date.now() - startTime < maxWait) {
    const analysis = await mcp.callTool("get_analysis", {
      analysis_id: result.analysis_id,
      project_path: "/path/to/project"
    });

    if (analysis.status !== "pending") {
      if (analysis.status === "failed") {
        console.error("Analysis failed");
        break;
      }

      // Success - process results
      console.log(`Found ${analysis.issues_found} issues`);
      break;
    }

    await sleep(1000);
  }
} catch (error) {
  console.error(`Analysis error: ${error.message}`);
}
```

---

## Further Reading

- **[MCP_CLIENT_INTEGRATION.md](./MCP_CLIENT_INTEGRATION.md)** - Claude Desktop and MCP client setup
- **[MCP_INTEGRATION_PLAN.md](./MCP_INTEGRATION_PLAN.md)** - Complete architecture and implementation details
- **[PHASE_3_COMPLETION_REPORT.md](./PHASE_3_COMPLETION_REPORT.md)** - Phase 3 & 4 technical details

---

## Support

**Issues:** https://github.com/yourusername/loglens/issues
**Documentation:** https://github.com/yourusername/loglens/docs
**Examples:** https://github.com/yourusername/loglens/examples
