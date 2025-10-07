# LogLens MCP Server - Quick Start

Get LogLens working with Claude Desktop in 5 minutes.

## 1. Build the MCP Server

```bash
cd /path/to/loglens
cargo build --release -p loglens-core --features mcp-server,project-management --bin mcp_server
```

Binary location: `./target/release/mcp_server`

## 2. Configure Claude Desktop

Edit your Claude Desktop config:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Linux**: `~/.config/claude-desktop/config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

Add this (replace `/path/to/loglens` with your actual path):

```json
{
  "mcpServers": {
    "loglens": {
      "command": "/path/to/loglens/target/release/mcp_server",
      "env": {
        "OPENROUTER_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

## 3. Restart Claude Desktop

Close and reopen Claude Desktop completely.

## 4. Test It

In Claude Desktop, try:

```
"Analyze these logs and find errors:

[2024-10-06 10:30:45] ERROR Database connection timeout
[2024-10-06 10:30:46] WARN Retrying connection attempt 1
[2024-10-06 10:30:47] ERROR Max retries exceeded
[2024-10-06 10:30:48] INFO Application shutting down
"
```

Claude should use the `analyze_logs` tool to process your request.

## 5. Project-Based Workflow (Optional)

For persistent analysis tracking:

```bash
# Initialize a project
cd /your/project
loglens init

# In Claude Desktop:
"Add the log file at logs/app.log to my project at /your/project"
```

## Available Tools

- **analyze_logs** - AI-powered log analysis
- **parse_logs** - Structure log entries
- **filter_logs** - Filter by log level
- **add_log_file** - Add logs to project
- **get_analysis** - Retrieve results by ID
- **query_analyses** - Search analysis history

## Troubleshooting

**"Server not found"**
- Check the `command` path is absolute and correct
- Verify the binary exists: `ls -la /path/to/loglens/target/release/mcp_server`
- Make it executable: `chmod +x /path/to/loglens/target/release/mcp_server`

**"API key invalid"**
- Set your API key in the config under `env`
- Or pass it in each request via the `api_key` parameter

**"No tools available"**
- Restart Claude Desktop after config changes
- Check for JSON syntax errors in config file
- Review Claude Desktop logs for errors

## Next Steps

- Read [full MCP Server documentation](MCP_SERVER.md)
- Learn about [project management](../README.md#-usage)
- Configure [advanced options](MCP_SERVER.md#advanced-configuration)

---

**Quick Reference**

Build: `cargo build --release -p loglens-core --features mcp-server,project-management --bin mcp_server`
Binary: `./target/release/mcp_server`
Config: `~/.config/claude-desktop/config.json` (Linux)
Init Project: `loglens init`
