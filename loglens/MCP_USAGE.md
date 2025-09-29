# LogLens MCP Server Usage

## Overview
LogLens provides an MCP (Model Context Protocol) server for integration with AI applications and tools.

## Starting the Server
```bash
./loglens --mcp-server
```

## MCP Protocol Requirements
The server strictly follows MCP protocol specifications and requires a complete handshake:

1. **Initialize Request**: Client sends `initialize` with capabilities
2. **Initialize Response**: Server responds with its capabilities
3. **Initialized Notification**: Client MUST send `notifications/initialized`
4. **Ready**: Server accepts tool requests

## Available Tools

### analyze_logs
Analyze log entries using AI to identify issues and patterns.

### parse_logs
Parse raw log text into structured log entries.

### filter_logs
Filter log entries by level and patterns.

## Testing
Use the provided test script to verify proper handshake:
```bash
# Example proper handshake sequence
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0"}}}' | ./loglens --mcp-server
```

## Troubleshooting
If you see "connection closed: initialized request", ensure your MCP client sends the required `notifications/initialized` message after the initialize response.