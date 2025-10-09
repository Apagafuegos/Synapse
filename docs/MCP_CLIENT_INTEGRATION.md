# LogLens MCP Client Integration Guide

**Complete guide for integrating LogLens MCP server with Claude Desktop and other MCP clients**

## Table of Contents
- [Overview](#overview)
- [Claude Desktop Integration](#claude-desktop-integration)
- [Generic MCP Client Integration](#generic-mcp-client-integration)
- [Authentication Setup](#authentication-setup)
- [Testing the Integration](#testing-the-integration)
- [Common Issues](#common-issues)
- [Security Considerations](#security-considerations)

---

## Overview

### What is MCP?

Model Context Protocol (MCP) is a standardized protocol for connecting AI assistants to external tools and data sources. LogLens implements MCP to expose log analysis capabilities to LLMs like Claude.

### Architecture

```
┌─────────────────┐
│ Claude Desktop  │ (or other MCP client)
│   (LLM Client)  │
└────────┬────────┘
         │ JSON-RPC 2.0
         │ over stdio
         ▼
┌─────────────────┐
│ LogLens MCP     │
│ Server          │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Your Project    │
│ .loglens/       │
│ ├─ index.db     │
│ └─ analyses/    │
└─────────────────┘
```

### Supported Clients

- ✅ **Claude Desktop** (recommended)
- ✅ **Custom MCP clients** (via stdio)
- ✅ **Web-based MCP clients** (with adapter)
- ⚠️ **Network-based clients** (requires wrapper)

---

## Claude Desktop Integration

### Prerequisites

- **Claude Desktop**: Latest version (download from claude.ai/desktop)
- **LogLens**: Installed and in PATH
- **Operating System**: macOS, Windows, or Linux

### Step 1: Install LogLens

```bash
# Build with MCP features
cd /path/to/loglens
cargo build --release --features "project-management,mcp-server"
cargo install --path . --features "project-management,mcp-server"

# Verify installation
loglens --version
which loglens  # Note this path for configuration
```

### Step 2: Configure Claude Desktop

Claude Desktop uses a configuration file to define MCP servers.

**Configuration File Location:**

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |

**Create or edit the configuration file:**

```json
{
  "mcpServers": {
    "loglens": {
      "command": "/full/path/to/loglens",
      "args": ["--mcp-server"],
      "env": {
        "OPENROUTER_API_KEY": "your-openrouter-api-key-here",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Important:**
- Use **absolute path** to loglens binary (from `which loglens`)
- Set API keys as environment variables
- Optional: Add `RUST_LOG` for debugging

### Step 3: Restart Claude Desktop

Close and reopen Claude Desktop to load the new configuration.

### Step 4: Verify Connection

Open Claude Desktop and ask:

```
Can you list the available MCP tools?
```

**Expected response:**
```
I can see the following LogLens tools:
- analyze_logs
- parse_logs
- filter_logs
- add_log_file
- get_analysis
- query_analyses
```

### Step 5: Test Log Analysis

Initialize a project and test analysis:

```
1. Initialize LogLens in /path/to/my-project
2. Analyze the log file at /path/to/my-project/logs/app.log
3. Show me the results
```

**Claude will:**
1. Call `add_log_file` tool
2. Wait for analysis to complete
3. Call `get_analysis` tool
4. Present results in natural language

---

## Generic MCP Client Integration

### Protocol Specification

LogLens implements **MCP Protocol Version 2024-11-05**.

**Communication:**
- **Transport:** stdio (standard input/output)
- **Format:** JSON-RPC 2.0
- **Encoding:** UTF-8

### Handshake Sequence

```json
// 1. Client → Server: Initialize
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "clientInfo": {
      "name": "your-client",
      "version": "1.0.0"
    }
  }
}

// 2. Server → Client: Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "loglens",
      "version": "0.1.0"
    }
  }
}

// 3. Client → Server: Initialized notification
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### Tool Discovery

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}

// Response
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "add_log_file",
        "description": "Add log file to project and trigger analysis",
        "inputSchema": {
          "type": "object",
          "properties": {
            "project_path": {"type": "string"},
            "log_file_path": {"type": "string"},
            "level": {"type": "string"},
            "provider": {"type": "string"},
            "auto_analyze": {"type": "boolean"},
            "api_key": {"type": "string"}
          },
          "required": ["project_path", "log_file_path"]
        }
      }
      // ... other tools
    ]
  }
}
```

### Tool Invocation

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "add_log_file",
    "arguments": {
      "project_path": "/path/to/project",
      "log_file_path": "logs/app.log",
      "level": "ERROR"
    }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"success\":true,\"analysis_id\":\"550e8400-...\",\"status\":\"pending\"}"
      }
    ]
  }
}
```

### Error Handling

```json
// Error Response
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Invalid params: project_path not found",
    "data": {
      "param": "project_path",
      "reason": ".loglens directory not found"
    }
  }
}
```

**Standard Error Codes:**
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

### Example: Python MCP Client

```python
import json
import subprocess
import sys

class LogLensMCPClient:
    def __init__(self, loglens_path):
        self.process = subprocess.Popen(
            [loglens_path, "--mcp-server"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        self.id = 0

    def send_request(self, method, params=None):
        self.id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self.id,
            "method": method,
            "params": params or {}
        }

        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()

        response = json.loads(self.process.stdout.readline())
        return response

    def initialize(self):
        return self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "python-client", "version": "1.0.0"}
        })

    def list_tools(self):
        return self.send_request("tools/list")

    def call_tool(self, tool_name, arguments):
        return self.send_request("tools/call", {
            "name": tool_name,
            "arguments": arguments
        })

    def close(self):
        self.process.stdin.close()
        self.process.wait()


# Usage
client = LogLensMCPClient("/usr/local/bin/loglens")

# Initialize
init_response = client.initialize()
print("Initialized:", init_response)

# List tools
tools = client.list_tools()
print("Available tools:", [t["name"] for t in tools["result"]["tools"]])

# Add log file
result = client.call_tool("add_log_file", {
    "project_path": "/path/to/project",
    "log_file_path": "logs/app.log",
    "level": "ERROR"
})
print("Analysis started:", result)

client.close()
```

### Example: Node.js MCP Client

```javascript
const { spawn } = require('child_process');
const readline = require('readline');

class LogLensMCPClient {
  constructor(loglensPath) {
    this.process = spawn(loglensPath, ['--mcp-server']);
    this.id = 0;
    this.pendingRequests = new Map();

    const rl = readline.createInterface({
      input: this.process.stdout,
      crlfDelay: Infinity
    });

    rl.on('line', (line) => {
      const response = JSON.parse(line);
      const callback = this.pendingRequests.get(response.id);
      if (callback) {
        callback(response);
        this.pendingRequests.delete(response.id);
      }
    });
  }

  sendRequest(method, params) {
    return new Promise((resolve, reject) => {
      this.id++;
      const request = {
        jsonrpc: "2.0",
        id: this.id,
        method,
        params: params || {}
      };

      this.pendingRequests.set(this.id, (response) => {
        if (response.error) {
          reject(new Error(response.error.message));
        } else {
          resolve(response.result);
        }
      });

      this.process.stdin.write(JSON.stringify(request) + '\n');
    });
  }

  async initialize() {
    return this.sendRequest('initialize', {
      protocolVersion: "2024-11-05",
      capabilities: { tools: {} },
      clientInfo: { name: "node-client", version: "1.0.0" }
    });
  }

  async listTools() {
    return this.sendRequest('tools/list');
  }

  async callTool(toolName, arguments) {
    return this.sendRequest('tools/call', {
      name: toolName,
      arguments
    });
  }

  close() {
    this.process.stdin.end();
  }
}

// Usage
(async () => {
  const client = new LogLensMCPClient('/usr/local/bin/loglens');

  // Initialize
  await client.initialize();
  console.log('Initialized');

  // List tools
  const tools = await client.listTools();
  console.log('Available tools:', tools.tools.map(t => t.name));

  // Add log file
  const result = await client.callTool('add_log_file', {
    project_path: '/path/to/project',
    log_file_path: 'logs/app.log',
    level: 'ERROR'
  });
  console.log('Analysis started:', result);

  client.close();
})();
```

---

## Authentication Setup

### API Keys Configuration

LogLens supports multiple AI providers. Configure API keys as environment variables.

**Method 1: Environment Variables**

```bash
# OpenRouter (recommended)
export OPENROUTER_API_KEY="sk-or-v1-..."

# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic Claude
export ANTHROPIC_API_KEY="sk-ant-..."

# Google Gemini
export GEMINI_API_KEY="..."
```

**Method 2: Configuration File**

Edit `~/.config/loglens/config.toml`:

```toml
[api_keys]
openrouter = "sk-or-v1-..."
openai = "sk-..."
anthropic = "sk-ant-..."
gemini = "..."
```

**Method 3: Per-Request**

Pass API key in tool call:

```json
{
  "tool": "add_log_file",
  "parameters": {
    "project_path": "/path/to/project",
    "log_file_path": "logs/app.log",
    "api_key": "sk-or-v1-..."
  }
}
```

**Precedence:**
1. Per-request `api_key` parameter (highest)
2. Environment variables
3. Configuration file
4. Error if none found

### Getting API Keys

**OpenRouter** (recommended for flexibility):
1. Visit https://openrouter.ai
2. Sign up for an account
3. Go to Settings → API Keys
4. Generate a new API key
5. Copy and set as `OPENROUTER_API_KEY`

**OpenAI:**
1. Visit https://platform.openai.com
2. Sign in and go to API Keys
3. Create new secret key
4. Set as `OPENAI_API_KEY`

**Anthropic Claude:**
1. Visit https://console.anthropic.com
2. Get API access
3. Generate API key
4. Set as `ANTHROPIC_API_KEY`

**Google Gemini:**
1. Visit https://makersuite.google.com
2. Get API key
3. Set as `GEMINI_API_KEY`

---

## Testing the Integration

### Quick Test

```bash
# 1. Start MCP server manually
loglens --mcp-server

# 2. In another terminal, send test request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0.0"}}}' | loglens --mcp-server
```

### Comprehensive Test

```bash
# Run integration tests
cd /path/to/loglens
cargo test --features "project-management,mcp-server" --test mcp_integration

# Expected output:
# running 9 tests
# test test_list_tools_includes_new_tools ... ok
# test test_add_log_file_workflow ... ok
# test test_get_analysis_formats ... ok
# test test_query_analyses_filters ... ok
# test test_error_handling ... ok
# test test_project_path_validation ... ok
# test test_log_file_path_resolution ... ok
# ...
```

### Manual Test Workflow

```bash
# 1. Initialize test project
mkdir /tmp/loglens-test
cd /tmp/loglens-test
echo '[ERROR] Test error' > test.log
loglens init

# 2. Start server (leave running)
loglens --mcp-server &
SERVER_PID=$!

# 3. Test via Claude Desktop or custom client
# - Add log file: test.log
# - Get analysis results
# - Query analyses

# 4. Cleanup
kill $SERVER_PID
rm -rf /tmp/loglens-test
```

### Validation Checklist

- ✅ Server starts without errors
- ✅ Handshake completes successfully
- ✅ All 6 tools are discoverable
- ✅ add_log_file creates analysis
- ✅ get_analysis retrieves results
- ✅ query_analyses filters correctly
- ✅ Error handling is graceful
- ✅ Multiple analyses work concurrently

---

## Common Issues

### Issue 1: Server Won't Start

**Symptoms:**
```bash
$ loglens --mcp-server
Error: failed to start MCP server
```

**Causes & Solutions:**

**Missing features:**
```bash
# Rebuild with features
cargo build --release --features "project-management,mcp-server"
```

**Port already in use (shouldn't happen with stdio):**
```bash
# LogLens uses stdio, not network ports
# This usually indicates another issue
```

**Permissions:**
```bash
# Ensure loglens is executable
chmod +x /path/to/loglens
```

### Issue 2: Claude Desktop Can't Find LogLens

**Symptoms:**
Claude Desktop says "LogLens not available" or "No MCP servers connected"

**Solutions:**

**Check configuration path:**
```bash
# macOS
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Linux
cat ~/.config/Claude/claude_desktop_config.json
```

**Verify absolute path:**
```json
{
  "mcpServers": {
    "loglens": {
      "command": "/usr/local/bin/loglens",  // Must be absolute!
      "args": ["--mcp-server"]
    }
  }
}
```

**Check binary exists:**
```bash
which loglens
# Use this exact path in config
```

**Restart Claude Desktop:**
- Close completely (not just window)
- Reopen and verify connection

### Issue 3: API Key Errors

**Symptoms:**
```
Error: API key not found for provider 'openrouter'
```

**Solutions:**

**Set environment variable in config:**
```json
{
  "mcpServers": {
    "loglens": {
      "command": "/usr/local/bin/loglens",
      "args": ["--mcp-server"],
      "env": {
        "OPENROUTER_API_KEY": "your-key-here"
      }
    }
  }
}
```

**Or use system environment:**
```bash
# In ~/.bashrc or ~/.zshrc
export OPENROUTER_API_KEY="sk-or-v1-..."
```

**Test key validity:**
```bash
curl https://openrouter.ai/api/v1/models \
  -H "Authorization: Bearer $OPENROUTER_API_KEY"
```

### Issue 4: Analysis Timeout

**Symptoms:**
Analysis stays in "pending" status indefinitely

**Causes:**
- Large log file (>100MB)
- Slow AI provider response
- Network issues

**Solutions:**

**Check analysis status:**
```javascript
// Poll with timeout
const maxWait = 120000; // 2 minutes
const startTime = Date.now();

while (Date.now() - startTime < maxWait) {
  const result = await mcp.callTool("get_analysis", {
    analysis_id: "...",
    project_path: "..."
  });

  if (result.status !== "pending") break;
  await sleep(2000);
}
```

**Enable debug logging:**
```bash
# In Claude Desktop config
"env": {
  "RUST_LOG": "debug"
}
```

**Check server logs:**
```bash
# macOS
tail -f ~/Library/Logs/Claude/mcp-server-loglens.log

# Linux
tail -f ~/.local/share/Claude/logs/mcp-server-loglens.log
```

### Issue 5: Permission Denied

**Symptoms:**
```
Error: Permission denied: /path/to/project/.loglens
```

**Solutions:**

**Check directory permissions:**
```bash
ls -la /path/to/project/.loglens/
# Should show read/write for user
```

**Fix permissions:**
```bash
chmod -R u+rw /path/to/project/.loglens/
```

**Run as correct user:**
```bash
# Ensure Claude Desktop runs as your user
# (Should be default)
```

---

## Security Considerations

### 1. API Key Protection

**DO:**
- ✅ Store API keys in environment variables
- ✅ Use configuration files with restricted permissions (chmod 600)
- ✅ Rotate API keys regularly
- ✅ Use separate keys for development and production

**DON'T:**
- ❌ Commit API keys to version control
- ❌ Share API keys in chat logs
- ❌ Use production keys for testing
- ❌ Store keys in plaintext files with broad permissions

### 2. File System Access

LogLens MCP server has access to:
- ✅ Projects initialized with `loglens init`
- ✅ Log files within those projects
- ✅ `.loglens/` directories

LogLens **cannot** access:
- ❌ Files outside initialized projects
- ❌ System files
- ❌ Other users' files

**Recommendation:** Only initialize LogLens in projects you want to analyze.

### 3. Network Security

**stdio Communication:**
- MCP communication over stdio (not network)
- No exposed ports
- Local-only access

**AI Provider Communication:**
- HTTPS to AI provider APIs
- API keys transmitted securely
- No sensitive data stored remotely (only analysis results)

### 4. Database Security

**SQLite Permissions:**
```bash
# Restrict .loglens directory
chmod 700 /path/to/project/.loglens/

# Restrict database file
chmod 600 /path/to/project/.loglens/index.db
```

**Sensitive Data:**
- LogLens stores analysis results locally
- Log file contents may be sent to AI providers
- Review logs for sensitive data before analysis

### 5. Audit Trail

All analyses are tracked:
```sql
-- View all analyses
sqlite3 .loglens/index.db "SELECT * FROM analyses;"

-- View by user (if multi-user system)
-- Track who initiated which analyses
```

---

## Performance Tuning

### Client-Side

**Connection Pooling:**
```javascript
// Reuse MCP client connection
const client = new LogLensMCPClient();
await client.initialize();

// Perform multiple operations
for (const file of logFiles) {
  await client.callTool('add_log_file', {...});
}

// Close once when done
client.close();
```

**Batch Operations:**
```javascript
// Instead of sequential
for (const file of files) {
  await addLogFile(file);
}

// Use concurrent
await Promise.all(
  files.map(file => addLogFile(file))
);
```

### Server-Side

**Database Configuration:**

Already optimized:
- WAL mode enabled
- Connection pooling (max 5 connections)
- Indexes on critical columns

**Log File Optimization:**

```bash
# For very large files, pre-filter
grep ERROR large.log > errors-only.log
# Analyze filtered file
```

---

## Support Resources

- **Documentation:** [MCP_USAGE.md](./MCP_USAGE.md)
- **Architecture:** [MCP_INTEGRATION_PLAN.md](./MCP_INTEGRATION_PLAN.md)
- **Issues:** https://github.com/yourusername/loglens/issues
- **MCP Specification:** https://spec.modelcontextprotocol.io/

---

## Appendix: Configuration Examples

### Example 1: Claude Desktop with Multiple Projects

```json
{
  "mcpServers": {
    "loglens": {
      "command": "/usr/local/bin/loglens",
      "args": ["--mcp-server"],
      "env": {
        "OPENROUTER_API_KEY": "sk-or-v1-...",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Example 2: Development with Debug Logging

```json
{
  "mcpServers": {
    "loglens-dev": {
      "command": "/Users/dev/loglens/target/debug/loglens",
      "args": ["--mcp-server"],
      "env": {
        "OPENROUTER_API_KEY": "sk-or-v1-...",
        "RUST_LOG": "debug,sqlx=info"
      }
    }
  }
}
```

### Example 3: Multiple AI Providers

```json
{
  "mcpServers": {
    "loglens": {
      "command": "/usr/local/bin/loglens",
      "args": ["--mcp-server"],
      "env": {
        "OPENROUTER_API_KEY": "sk-or-v1-...",
        "OPENAI_API_KEY": "sk-...",
        "ANTHROPIC_API_KEY": "sk-ant-...",
        "GEMINI_API_KEY": "..."
      }
    }
  }
}
```
