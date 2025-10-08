# Install Script Updates for New MCP Version

## Summary

The install scripts (`install.sh` and `install.bat`) have been updated to reflect the new MCP capabilities with the updated implementation. The scripts were already well-structured but needed minor updates to showcase the new features.

## Changes Made

### 1. Updated Usage Examples

**install.sh:**
```bash
# OLD
echo "  loglens --mcp-server              # Start MCP server"
echo "  loglens init                      # Initialize project"

# NEW  
echo "  loglens --mcp-server              # Start MCP server (stdio mode)"
echo "  loglens --mcp-server --mcp-transport http  # Start MCP server (HTTP mode)"
echo "  loglens --mcp-server --mcp-port 8080       # Start MCP server on custom port"
echo "  loglens init                      # Initialize project"
```

**install.bat:**
```batch
# OLD
echo   loglens --mcp-server              # Start MCP server
echo   loglens init                      # Initialize project

# NEW
echo   loglens --mcp-server              # Start MCP server (stdio mode)
echo   loglens --mcp-server --mcp-transport http  # Start MCP server (HTTP mode)
echo   loglens --mcp-server --mcp-port 8080       # Start MCP server on custom port
echo   loglens init                      # Initialize project
```

### 2. Updated MCP Tool Descriptions

**install.sh:**
```bash
# OLD
echo "  - analyze_logs: AI-powered log analysis"
echo "  - parse_logs: Parse raw logs into structured format"
echo "  - filter_logs: Filter logs by level and patterns"

# NEW
echo "  - list_projects: List available LogLens projects"
echo "  - get_project: Get detailed project information"
echo "  - list_analyses: List analyses for a project"
echo "  - get_analysis: Get complete analysis results"
echo "  - get_analysis_status: Get analysis status for polling"
echo "  - analyze_file: Trigger new analysis on existing file"
```

**install.bat:**
```batch
# OLD
echo   - analyze_logs: AI-powered log analysis
echo   - parse_logs: Parse raw logs into structured format
echo   - filter_logs: Filter logs by level and patterns

# NEW
echo   - list_projects: List available LogLens projects
echo   - get_project: Get detailed project information
echo   - list_analyses: List analyses for a project
echo   - get_analysis: Get complete analysis results
echo   - get_analysis_status: Get analysis status for polling
echo   - analyze_file: Trigger new analysis on existing file
```

## Existing Features That Remain Unchanged

### ✅ Configuration Files
- MCP server configuration (`[mcp_server]` section with port and host)
- Data directory setup
- AI provider configuration

### ✅ Service Integration
- systemd service file for Linux (`loglens-mcp.service`)
- Windows service script (`start-loglens-mcp.bat`)
- Proper environment variable setup

### ✅ Installation Process
- Build system integration (still builds `loglens-cli` package)
- Frontend build and installation
- PATH setup and verification
- Comprehensive error handling

### ✅ Docker Support
- Existing Docker commands remain functional
- Volume mounting for data persistence

## New Features Highlighted

### 🚀 Transport Options
- **STDIO Transport**: Default mode for Claude Desktop integration
- **HTTP Transport**: Web-based access with SSE support
- **Custom Ports**: Flexible port configuration

### 🛠️ Enhanced MCP Tools
The updated tool list reflects the actual implementation:
- **Project Management**: `list_projects`, `get_project`
- **Analysis Management**: `list_analyses`, `get_analysis`, `get_analysis_status`
- **File Analysis**: `analyze_file` with async execution

## Compatibility

### ✅ Backward Compatibility
- All existing functionality remains supported
- Configuration files use same format
- Service files remain functional

### ✅ Cross-Platform Support
- **Linux**: Full systemd integration
- **Windows**: Background service scripts
- **macOS**: Standard Unix-like installation

## Testing the Updated Scripts

```bash
# Test Linux installation
./install.sh

# Test Windows installation (in Windows environment)
install.bat

# Verify new MCP functionality
loglens --mcp-server --mcp-transport http --mcp-port 8080
```

## Migration Notes

### For Existing Users
- No migration required - existing installations continue to work
- New features are available immediately after update
- Configuration files remain compatible

### For New Users
- Scripts provide comprehensive setup guidance
- All MCP transport options are demonstrated
- Service integration is automatically configured

## Future Considerations

### Potential Enhancements
- Add MCP client configuration examples
- Include Claude Desktop setup instructions
- Add troubleshooting guide for transport issues

### Maintenance
- Scripts are now synchronized with actual MCP implementation
- Tool descriptions match server capabilities exactly
- Transport options reflect supported implementations

The install scripts are now fully aligned with the updated MCP implementation and provide users with comprehensive guidance for leveraging all new features.