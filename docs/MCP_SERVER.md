# Synapse MCP Server

Synapse provides a Model Context Protocol (MCP) server that enables AI assistants like Claude Desktop to analyze logs directly.

## Building the MCP Server

```bash
# From the repository root
cargo build --release -p synapse-core --features mcp-server,project-management --bin mcp_server

# Binary location
./target/release/mcp_server
```

## Running the MCP Server

The MCP server uses stdio (standard input/output) for communication:

```bash
# Run directly
./target/release/mcp_server

# The server will listen on stdin/stdout for MCP protocol messages
# Logs are written to stderr
```

## Claude Desktop Integration

Add to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Linux**: `~/.config/claude-desktop/config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "synapse": {
      "command": "/absolute/path/to/synapse/target/release/mcp_server",
      "env": {
        "RUST_LOG": "info",
        "OPENROUTER_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

## Available MCP Tools

### 1. `analyze_logs`
Analyze log lines using AI to identify patterns, issues, and insights.

**Parameters**:
- `logs` (required): Array of log lines to analyze
- `level` (optional): Minimum log level ("ERROR", "WARN", "INFO", "DEBUG")
- `provider` (optional): AI provider ("openrouter", "openai", "claude", "gemini")
- `api_key` (optional): API key for the provider (if not set in env)

**Example**:
```json
{
  "logs": [
    "[ERROR] Database connection failed",
    "[WARN] Retry attempt 1",
    "[ERROR] Max retries exceeded"
  ],
  "level": "ERROR",
  "provider": "openrouter"
}
```

### 2. `parse_logs`
Parse log lines into structured format with timestamps, levels, and messages.

**Parameters**:
- `logs` (required): Array of log lines to parse

**Example**:
```json
{
  "logs": [
    "2024-01-15 10:30:45 ERROR Database connection timeout",
    "2024-01-15 10:30:46 WARN Retrying connection"
  ]
}
```

### 3. `filter_logs`
Filter log lines by minimum log level.

**Parameters**:
- `logs` (required): Array of log lines to filter
- `level` (required): Minimum log level to include

**Example**:
```json
{
  "logs": ["[ERROR] Critical", "[INFO] Normal", "[WARN] Warning"],
  "level": "WARN"
}
```

### 4. `add_log_file` (Project Management)
Add a log file to a Synapse project and trigger analysis.

**Parameters**:
- `project_path` (required): Path to project root (must contain `.synapse/`)
- `log_file_path` (required): Absolute or relative path to log file
- `level` (optional): Minimum log level ("ERROR", "WARN", "INFO", "DEBUG")
- `provider` (optional): AI provider ("openrouter", "openai", "claude", "gemini")
- `auto_analyze` (optional): Automatically trigger analysis (default: true)
- `api_key` (optional): API key for the provider

**Example**:
```json
{
  "project_path": "/home/user/my-project",
  "log_file_path": "logs/app.log",
  "level": "ERROR",
  "provider": "openrouter",
  "auto_analyze": true
}
```

**Prerequisites**: Run `synapse init` in the project directory first.

### 5. `get_analysis`
Retrieve analysis results by ID.

**Parameters**:
- `analysis_id` (required): UUID of the analysis
- `project_path` (required): Path to project for validation
- `format` (optional): Detail level ("summary", "full", "structured")

**Example**:
```json
{
  "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
  "project_path": "/home/user/my-project",
  "format": "summary"
}
```

### 6. `query_analyses`
Query analyses with filters.

**Parameters**:
- `project_path` (required): Filter by project
- `status` (optional): Filter by status ("pending", "completed", "failed")
- `limit` (optional): Maximum results to return (default: 10)
- `since` (optional): ISO timestamp - analyses after this time

**Example**:
```json
{
  "project_path": "/home/user/my-project",
  "status": "completed",
  "limit": 5
}
```

## Environment Variables

Configure the MCP server behavior via environment variables:

```bash
# Logging level
RUST_LOG=info                    # Options: error, warn, info, debug, trace

# AI Provider API Keys
OPENROUTER_API_KEY=sk-or-...
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
```

## Usage in Claude Desktop

Once configured, you can use natural language commands in Claude Desktop:

```
"Analyze these error logs from my application"
[paste logs]

"Parse this log file and show me the structure"
[provide log content]

"Add this log file to my project and analyze errors"
project: /home/user/my-app
file: logs/production.log
```

Claude will automatically invoke the appropriate MCP tools to process your request.

## Workflow: Project-Based Analysis

1. **Initialize a Synapse project**:
   ```bash
   cd /path/to/your/project
   synapse init
   ```

2. **Configure Claude Desktop** with the MCP server (see above)

3. **In Claude Desktop**, analyze logs:
   ```
   "Add the log file at logs/app.log to my project at /path/to/your/project
    and analyze all ERROR level entries"
   ```

4. **Retrieve results**:
   ```
   "Show me the analysis summary for ID 550e8400-..."
   ```

5. **Query history**:
   ```
   "Show me the last 5 completed analyses for /path/to/your/project"
   ```

## Troubleshooting

### Server Not Starting
- Check that the binary path in `claude_desktop_config.json` is absolute
- Verify the binary is executable: `chmod +x target/release/mcp_server`
- Check stderr logs for initialization errors

### API Key Issues
- Ensure API keys are set in environment variables or passed in tool parameters
- Keys can be set in `claude_desktop_config.json` under `env`
- Verify key format matches provider requirements

### Analysis Failures
- Check that the log format is recognized (JSON, syslog, common formats)
- Ensure the AI provider is available and responding
- Review analysis results for specific error messages

### Project Not Found
- Verify `.synapse/` directory exists in project root
- Run `synapse init` if not already initialized
- Check that `project_path` is absolute, not relative

## Advanced Configuration

### Custom Models

Configure specific AI models via the Synapse project config:

```bash
# Edit .synapse/config.toml in your project
[ai]
provider = "openrouter"
model = "anthropic/claude-3-opus"
```

### Database Location

Synapse stores analysis results in `.synapse/index.db` within each project.

### Log Caching

Analyzed logs are cached in `.synapse/logs/` to avoid re-reading large files.

## Development

### Testing the MCP Server

Test with raw JSON-RPC messages:

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/mcp_server

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/mcp_server

# Call analyze_logs
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"analyze_logs","arguments":{"logs":["[ERROR] Test"],"level":"ERROR"}}}' | ./target/release/mcp_server
```

### Debugging

Enable verbose logging:

```bash
RUST_LOG=debug ./target/release/mcp_server
```

Logs are written to stderr, so they won't interfere with the MCP protocol on stdout.

## See Also

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [Claude Desktop Documentation](https://claude.ai/desktop)
- [Synapse README](../README.md)
- [Project Management Guide](../README.md#-usage)
