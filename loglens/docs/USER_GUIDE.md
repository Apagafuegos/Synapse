# LogLens User Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Command Line Usage](#command-line-usage)
5. [Configuration](#configuration)
6. [AI Providers](#ai-providers)
7. [Output Formats](#output-formats)
8. [MCP Server Mode](#mcp-server-mode)
9. [Advanced Usage](#advanced-usage)
10. [Troubleshooting](#troubleshooting)

## Introduction

LogLens is an AI-powered log analysis tool that helps you identify issues, patterns, and anomalies in your application logs. It supports multiple AI providers and can output results in various formats for different use cases.

### Key Features

- **Multi-AI Provider Support**: OpenRouter, OpenAI, Claude (Anthropic), and Google Gemini
- **Advanced Analysis**: Pattern detection, performance metrics, anomaly detection, and correlation analysis
- **Multiple Output Formats**: Console, HTML, JSON, and Markdown
- **MCP Integration**: Model Context Protocol server for AI application integration
- **Smart Log Processing**: Automatic parsing, filtering, and optimization for AI analysis

## Installation

### From Source

```bash
git clone <repository-url>
cd loglens
cargo build --release
```

The binary will be available at `./target/release/loglens`.

### Using Cargo

```bash
cargo install loglens
```

## Quick Start

### 1. Set up API Key

Choose your preferred AI provider and set the corresponding environment variable:

```bash
# For OpenRouter (recommended)
export OPENROUTER_API_KEY="your_api_key_here"

# For OpenAI
export OPENAI_API_KEY="your_api_key_here"

# For Claude/Anthropic
export ANTHROPIC_API_KEY="your_api_key_here"

# For Google Gemini
export GEMINI_API_KEY="your_api_key_here"
```

### 2. Analyze Your First Log File

```bash
# Analyze errors in a log file
loglens --file /var/log/app.log --level ERROR

# Analyze with specific provider
loglens --file /var/log/app.log --level ERROR --provider claude

# Generate HTML report
loglens --file /var/log/app.log --level ERROR --output-html report.html
```

### 3. Analyze Command Output

```bash
# Analyze errors from a running command
loglens --exec "journalctl -u nginx --since today" --level ERROR

# Analyze docker container logs
loglens --exec "docker logs myapp" --level WARN
```

## Command Line Usage

### Basic Syntax

```bash
loglens [OPTIONS]
```

### Required Options (choose one)

- `--file <PATH>` - Analyze log file
- `--exec <COMMAND>` - Execute command and analyze output
- `--mcp-server` - Start MCP server mode

### Common Options

| Option | Description | Default |
|--------|-------------|---------|
| `--level <LEVEL>` | Minimum log level (ERROR, WARN, INFO, DEBUG) | ERROR |
| `--provider <PROVIDER>` | AI provider (openrouter, openai, claude, gemini) | openrouter |
| `--api-key <KEY>` | API key for provider | From environment |
| `--output-format <FORMAT>` | Output format (console, html, json, markdown) | console |
| `--output-file <PATH>` | Save report to file | Print to console |
| `--output-html <PATH>` | Generate HTML report | None |

### Examples

#### Basic Log Analysis

```bash
# Analyze errors in application log
loglens --file /var/log/myapp.log --level ERROR

# Analyze with warnings included
loglens --file /var/log/myapp.log --level WARN

# Use specific AI provider
loglens --file /var/log/myapp.log --level ERROR --provider claude
```

#### Command Analysis

```bash
# Analyze systemd service logs
loglens --exec "journalctl -u myservice --since '1 hour ago'" --level ERROR

# Analyze Docker container logs
loglens --exec "docker logs --since 1h mycontainer" --level WARN

# Analyze kubectl logs
loglens --exec "kubectl logs deployment/myapp" --level ERROR
```

#### Output Options

```bash
# Generate HTML report
loglens --file app.log --level ERROR --output-html analysis.html

# Save as JSON for further processing
loglens --file app.log --level ERROR --output-format json --output-file analysis.json

# Generate Markdown for documentation
loglens --file app.log --level ERROR --output-format markdown --output-file analysis.md
```

#### Advanced Usage

```bash
# Use custom API key
loglens --file app.log --level ERROR --api-key "your-custom-key"

# Analyze with multiple providers (run separately)
loglens --file app.log --level ERROR --provider openai --output-file openai-analysis.txt
loglens --file app.log --level ERROR --provider claude --output-file claude-analysis.txt
```

## Configuration

LogLens supports hierarchical configuration through TOML files and environment variables.

### Configuration File Locations

1. **Project-level**: `./.loglens.toml` (current directory)
2. **User-level**: `~/.config/loglens/config.toml`

### Configuration Format

Create a `.loglens.toml` file:

```toml
[defaults]
provider = "openrouter"
log_level = "ERROR"

[providers.openrouter]
model = "openai/gpt-4"
timeout = 30
max_tokens = 2000
temperature = 0.1
# api_key = "optional-key-here"

[providers.openai]
model = "gpt-4"
timeout = 30
max_tokens = 2000
temperature = 0.1

[providers.claude]
model = "claude-3-opus-20240229"
timeout = 45
max_tokens = 4000
temperature = 0.0

[providers.gemini]
model = "gemini-pro"
timeout = 30
max_tokens = 2048
temperature = 0.1
```

### Environment Variables

API keys are typically set via environment variables:

```bash
export OPENROUTER_API_KEY="your_openrouter_key"
export OPENAI_API_KEY="your_openai_key"
export ANTHROPIC_API_KEY="your_anthropic_key"
export GEMINI_API_KEY="your_gemini_key"
```

### API Key Precedence

1. Command line `--api-key` parameter
2. Environment variable (`{PROVIDER}_API_KEY`)
3. Configuration file setting
4. Error if none found

## AI Providers

### OpenRouter (Recommended)

- **Best for**: Cost-effective access to multiple models
- **Models**: Access to GPT-4, Claude, Gemini, and others
- **Setup**: Get API key from [openrouter.ai](https://openrouter.ai)

```bash
export OPENROUTER_API_KEY="your_key"
loglens --file app.log --provider openrouter
```

### OpenAI

- **Best for**: Latest GPT models, reliable performance
- **Models**: GPT-4, GPT-3.5-turbo
- **Setup**: Get API key from [OpenAI](https://platform.openai.com)

```bash
export OPENAI_API_KEY="your_key"
loglens --file app.log --provider openai
```

### Claude (Anthropic)

- **Best for**: Detailed analysis, long context windows
- **Models**: Claude-3 (Opus, Sonnet, Haiku)
- **Setup**: Get API key from [Anthropic](https://console.anthropic.com)

```bash
export ANTHROPIC_API_KEY="your_key"
loglens --file app.log --provider claude
```

### Google Gemini

- **Best for**: Google ecosystem integration
- **Models**: Gemini Pro, Gemini Pro Vision
- **Setup**: Get API key from [Google AI Studio](https://makersuite.google.com)

```bash
export GEMINI_API_KEY="your_key"
loglens --file app.log --provider gemini
```

## Output Formats

### Console (Default)

Human-readable terminal output with sections and highlighting.

```bash
loglens --file app.log --output-format console
```

**Features:**
- Color-coded sections
- Word-wrapped text
- Progress indicators
- Suitable for terminal viewing

### HTML

Rich, interactive HTML reports with styling and navigation.

```bash
loglens --file app.log --output-format html --output-file report.html
# or
loglens --file app.log --output-html report.html
```

**Features:**
- Professional styling
- Collapsible sections
- Syntax highlighting for logs
- Suitable for sharing and archiving

### JSON

Machine-readable structured data for programmatic processing.

```bash
loglens --file app.log --output-format json --output-file analysis.json
```

**Structure:**
```json
{
  "metadata": {
    "timestamp": "2024-01-20T10:30:00Z",
    "provider": "openrouter",
    "total_logs": 1500,
    "filtered_logs": 45,
    "log_level": "ERROR",
    "input_source": "app.log"
  },
  "analysis": {
    "sequence_of_events": "...",
    "error_details": "...",
    "failed_component": "..."
  },
  "logs": [...],
  "stats": {
    "error_count": 45,
    "warning_count": 0,
    "info_count": 0,
    "debug_count": 0,
    "unique_messages": 12,
    "consolidated_duplicates": 0
  }
}
```

### Markdown

Documentation-friendly format for wikis and documentation systems.

```bash
loglens --file app.log --output-format markdown --output-file analysis.md
```

**Features:**
- Headers and structure
- Code blocks for logs
- Tables for statistics
- Compatible with GitHub, GitLab, etc.

## MCP Server Mode

LogLens can run as a Model Context Protocol (MCP) server, allowing integration with AI applications like Claude for Desktop.

### Starting the Server

```bash
# Start MCP server with stdio transport
loglens --mcp-server

# Alternative transport options (future)
loglens --mcp-server --mcp-transport stdio --mcp-port 8080
```

### Available MCP Tools

1. **analyze_logs**
   - Full AI analysis of log entries
   - Parameters: `logs`, `level`, `provider`, `api_key`

2. **parse_logs**
   - Parse raw log text into structured entries
   - Parameters: `logs`

3. **filter_logs**
   - Filter log entries by level and patterns
   - Parameters: `logs`, `level`, `pattern`

### MCP Client Integration

Example usage from an MCP client:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "analyze_logs",
    "arguments": {
      "logs": ["[ERROR] Database connection failed", "[WARN] Retrying connection"],
      "level": "ERROR",
      "provider": "openrouter"
    }
  }
}
```

### MCP JSON I/O Mode

For direct JSON processing:

```bash
echo '{
  "logs": ["[ERROR] Database failed", "[ERROR] Connection timeout"],
  "level": "ERROR",
  "provider": "openrouter",
  "output_format": "json"
}' | loglens --mcp-mode
```

## Advanced Usage

### Log Processing Pipeline

LogLens processes logs through several stages:

1. **Parsing**: Extract timestamps, levels, and messages
2. **Filtering**: Apply minimum log level filter
3. **Slimming**: Reduce volume for AI analysis
4. **AI Analysis**: Generate insights using selected provider
5. **Enhancement**: Add pattern analysis, performance metrics, etc.
6. **Report Generation**: Format output in requested format

### Pattern Analysis

LogLens automatically detects:

- **Recurring Patterns**: Repeated error messages and their frequency
- **Error Chains**: Sequences of related errors
- **Error Groups**: Errors clustered by component or pattern
- **Anomalies**: Unusual patterns and deviations

### Performance Analysis

Automatic performance metrics include:

- **Performance Score**: Overall system health (0-100)
- **Timing Statistics**: Response times and intervals
- **Bottleneck Detection**: Components causing delays
- **Trend Analysis**: Time-series performance data

### Anomaly Detection

Identifies unusual patterns:

- **High Error Frequency**: Unusual error rates
- **Security Alerts**: Potential security issues
- **Unusual Patterns**: Deviations from baseline
- **System Anomalies**: Infrastructure issues

### Correlation Analysis

Finds relationships between:

- **Error Correlations**: Related error types
- **Service Dependencies**: Component relationships
- **Cascading Failures**: Failure propagation patterns
- **Root Causes**: Primary failure sources

### Scripting and Automation

#### Bash Integration

```bash
#!/bin/bash
# Daily log analysis script

LOG_DIR="/var/log"
REPORT_DIR="/reports"
DATE=$(date +%Y-%m-%d)

for app in nginx mysql redis; do
    echo "Analyzing $app logs..."
    loglens --file "$LOG_DIR/$app.log" \
            --level ERROR \
            --provider openrouter \
            --output-html "$REPORT_DIR/$app-$DATE.html"
done
```

#### Python Integration

```python
import subprocess
import json

def analyze_logs(log_file, level="ERROR", provider="openrouter"):
    """Analyze logs and return structured results."""
    cmd = [
        "loglens",
        "--file", log_file,
        "--level", level,
        "--provider", provider,
        "--output-format", "json"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        return json.loads(result.stdout)
    else:
        raise Exception(f"Analysis failed: {result.stderr}")

# Usage
analysis = analyze_logs("/var/log/app.log")
print(f"Found {analysis['stats']['error_count']} errors")
```

### Performance Tuning

#### Large Log Files

For very large log files:

1. **Pre-filter logs**: Use `grep` or similar tools first
2. **Increase log level**: Use WARN or INFO instead of DEBUG
3. **Use slimming**: LogLens automatically reduces log volume
4. **Split analysis**: Analyze logs in time chunks

```bash
# Pre-filter large logs
grep "ERROR\|FATAL" /var/log/huge.log > errors.log
loglens --file errors.log --level ERROR
```

#### Batch Processing

```bash
# Process multiple files efficiently
find /var/log -name "*.log" -type f | while read logfile; do
    echo "Processing $logfile..."
    loglens --file "$logfile" --level ERROR --output-html "reports/$(basename $logfile .log).html"
done
```

## Troubleshooting

### Common Issues

#### 1. API Key Not Found

**Error**: `API key required for provider openrouter`

**Solutions:**
- Set environment variable: `export OPENROUTER_API_KEY="your_key"`
- Use command line: `--api-key "your_key"`
- Add to config file in `.loglens.toml`

#### 2. Authentication Failed

**Error**: `Authentication failed`

**Solutions:**
- Verify API key is correct
- Check provider account status and billing
- Ensure API key has proper permissions

#### 3. Rate Limited

**Error**: `Rate limited`

**Solutions:**
- Wait and retry
- Use different provider
- Reduce analysis frequency
- Check provider rate limits

#### 4. Invalid Response

**Error**: `Invalid response: ...`

**Solutions:**
- Try different AI provider
- Reduce log complexity/size
- Check provider service status
- Verify network connectivity

#### 5. File Not Found

**Error**: File access errors

**Solutions:**
- Verify file path is correct
- Check file permissions
- Ensure file exists and is readable
- Use absolute paths

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
RUST_LOG=debug loglens --file app.log --level ERROR
```

### Getting Help

1. **Check logs**: Enable debug mode for detailed error information
2. **Verify configuration**: Ensure API keys and settings are correct
3. **Test with simple logs**: Use small, well-formatted log files first
4. **Check provider status**: Verify AI provider service availability

### Best Practices

#### 1. API Key Security

- Use environment variables instead of command line parameters
- Don't commit API keys to version control
- Use separate keys for different environments
- Rotate keys periodically

#### 2. Cost Management

- Start with smaller, focused log analysis
- Use appropriate log levels (ERROR vs DEBUG)
- Monitor AI provider usage and costs
- Consider using OpenRouter for cost optimization

#### 3. Analysis Quality

- Use structured logs when possible
- Include sufficient context in log messages
- Filter relevant time ranges
- Combine with domain knowledge

#### 4. Integration

- Automate regular analysis for proactive monitoring
- Integrate with alerting systems
- Store analysis results for trend analysis
- Share reports with team members

## Next Steps

1. **Explore Advanced Features**: Try different AI providers and output formats
2. **Automate Analysis**: Set up regular log analysis scripts
3. **Integrate with Monitoring**: Combine with existing monitoring solutions
4. **Share Knowledge**: Use reports for incident post-mortems and team learning
5. **Contribute**: Report issues and contribute improvements

For more information, see:
- [API Documentation](API.md)
- [Configuration Reference](CONFIGURATION.md)
- [MCP Integration Guide](MCP_INTEGRATION.md)