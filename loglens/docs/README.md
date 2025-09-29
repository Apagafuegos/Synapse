# LogLens Documentation

Welcome to the LogLens documentation! This directory contains comprehensive guides and references for using LogLens, an AI-powered log analysis tool.

## Documentation Overview

### 📖 [User Guide](USER_GUIDE.md)
Complete guide for end users covering installation, basic usage, advanced features, and troubleshooting.

**What you'll learn:**
- How to install and set up LogLens
- Basic command-line usage and examples
- Working with different AI providers
- Output formats and report generation
- MCP server integration
- Advanced analysis features
- Scripting and automation

### 🔧 [API Documentation](API.md)
Comprehensive API reference for developers integrating LogLens as a library.

**What you'll find:**
- Core API interfaces and methods
- AI provider system documentation
- Configuration management APIs
- Log processing pipeline details
- Advanced analysis modules
- Data structures and types
- Code examples and usage patterns

### ⚙️ [Configuration Guide](CONFIGURATION.md)
Detailed configuration reference covering all settings and customization options.

**Topics covered:**
- Configuration file format and locations
- Environment variable usage
- Provider-specific settings
- Security best practices
- Environment-specific configurations
- Troubleshooting configuration issues

## Quick Start

### Installation
```bash
# From source
git clone <repository-url>
cd loglens
cargo build --release

# The binary will be at ./target/release/loglens
```

### Basic Usage
```bash
# Set API key
export OPENROUTER_API_KEY="your_api_key"

# Analyze a log file
loglens --file /var/log/app.log --level ERROR

# Generate HTML report
loglens --file /var/log/app.log --level ERROR --output-html report.html
```

## Key Features

- **Multi-AI Provider Support**: OpenRouter, OpenAI, Claude, Gemini
- **Smart Log Processing**: Automatic parsing, filtering, and optimization
- **Advanced Analysis**: Pattern detection, performance metrics, anomaly detection
- **Multiple Output Formats**: Console, HTML, JSON, Markdown
- **MCP Integration**: Model Context Protocol server for AI applications
- **Flexible Configuration**: TOML files and environment variables

## Architecture Overview

LogLens is built with a modular architecture:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Input Layer   │───▶│ Processing Core  │───▶│  Output Layer   │
│                 │    │                  │    │                 │
│ • File Reader   │    │ • Parser         │    │ • Console       │
│ • Command Exec  │    │ • Filter         │    │ • HTML          │
│ • MCP Server    │    │ • Slimmer        │    │ • JSON          │
└─────────────────┘    │ • AI Analysis    │    │ • Markdown      │
                       │ • Enhancers      │    └─────────────────┘
                       └──────────────────┘
                               │
                    ┌──────────────────┐
                    │   AI Providers   │
                    │                  │
                    │ • OpenRouter     │
                    │ • OpenAI         │
                    │ • Claude         │
                    │ • Gemini         │
                    └──────────────────┘
```

## Documentation Structure

### For Users
1. Start with [User Guide](USER_GUIDE.md) for basic usage
2. Refer to [Configuration Guide](CONFIGURATION.md) for customization
3. Check troubleshooting sections for common issues

### For Developers
1. Review [API Documentation](API.md) for integration
2. Understand the architecture and data flow
3. Explore code examples and patterns

### For Contributors
1. Read all documentation to understand the system
2. Check inline code documentation
3. Follow established patterns and conventions

## Examples and Use Cases

### System Administration
```bash
# Monitor system logs for errors
loglens --exec "journalctl --since '1 hour ago'" --level ERROR

# Analyze web server logs
loglens --file /var/log/nginx/error.log --level ERROR --output-html nginx-report.html
```

### Development and Debugging
```bash
# Analyze application logs during development
loglens --file logs/development.log --level WARN --provider claude

# Check for patterns in test failures
loglens --exec "npm test 2>&1" --level ERROR --output-format json
```

### DevOps and Monitoring
```bash
# Automated log analysis in CI/CD
loglens --file build.log --level ERROR --output-format json > analysis.json

# Kubernetes pod log analysis
loglens --exec "kubectl logs deployment/myapp" --level ERROR
```

### MCP Integration
```bash
# Start MCP server for AI applications
loglens --mcp-server

# Use with Claude for Desktop or other MCP clients
# Provides analyze_logs, parse_logs, and filter_logs tools
```

## Getting Help

### Documentation
- **User Guide**: Comprehensive usage documentation
- **API Reference**: Complete API documentation
- **Configuration**: Detailed configuration options

### Support Resources
- Check troubleshooting sections in guides
- Enable debug mode: `RUST_LOG=debug loglens ...`
- Verify configuration and API keys
- Test with simple examples first

### Common Issues
1. **API Key Problems**: Ensure environment variables are set correctly
2. **Rate Limiting**: Use different providers or reduce frequency
3. **Large Files**: Pre-filter logs or use higher log levels
4. **Permission Issues**: Check file access and permissions

## Contributing

We welcome contributions to LogLens! Areas where you can help:

### Documentation
- Improve existing guides and examples
- Add use case documentation
- Create integration tutorials
- Fix typos and clarifications

### Code
- Bug fixes and improvements
- New AI provider integrations
- Additional output formats
- Performance optimizations

### Testing
- Test with different log formats
- Verify provider integrations
- Performance and load testing
- Documentation accuracy

## Version Information

This documentation is for LogLens v0.1.0. Features and APIs may change in future versions.

## License

LogLens is released under [appropriate license]. See the main repository for license details.

---

**Need more help?** Check the specific guides above or enable debug logging for detailed troubleshooting information.