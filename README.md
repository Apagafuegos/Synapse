# LogLens

**Intelligent Log Analysis Made Simple**

LogLens is a powerful log analysis tool that leverages artificial intelligence to help developers and system administrators understand complex log files quickly and efficiently. Instead of manually sifting through thousands of log entries, LogLens uses advanced AI to identify patterns, detect anomalies, and provide actionable insights.

## 🤖 AI-Driven Development

LogLens is collaboration between artificial intelligence and human expertise. This project has been **fully planned, developed, and structured by AI systems**, with significant contributions from:

- **Claude Sonnet**: Responsible for core architecture design, Rust implementation, and major feature development
- **GLM-4.5**: Contributed to advanced AI integration, optimization strategies, and technical documentation

### Human Involvement

While the AI systems handled the technical implementation, human guidance was essential throughout the process:

- **Original Concept**: The idea for an intelligent log analysis tool was conceived by human developers
- **Requirements Definition**: Human input shaped the feature set and user experience goals
- **Testing & Validation**: Human testers provided feedback and validated the AI-generated code
- **Debugging & Refinement**: Human developers identified and resolved issues that AI systems couldn't detect
- **Deployment Strategy**: Human oversight ensured proper deployment and distribution planning

This collaborative approach demonstrates how AI can handle complex software development tasks while human creativity and oversight guide the project toward practical, user-focused solutions.

## 🌟 Key Features

### Smart Log Analysis
- **AI-Powered Insights**: Uses state-of-the-art AI models (OpenAI, Claude, Gemini, OpenRouter) to analyze log patterns and identify issues
- **Pattern Detection**: Automatically identifies recurring patterns and frequency trends in your logs
- **Anomaly Detection**: Flags unusual log entries that might indicate problems or security issues
- **Performance Analysis**: Identifies performance bottlenecks and timing-related issues
- **Root Cause Analysis**: Correlates different errors to help identify underlying causes

### Multiple Input Sources
- **File Analysis**: Analyze log files directly from your system
- **Command Execution**: Run commands and analyze their output in real-time
- **Streaming Support**: Monitor live log streams as they're generated

### Flexible Output Formats
- **Console Reports**: Clean, readable terminal output with organized sections
- **HTML Reports**: Rich, interactive web-based reports with charts and visualizations
- **JSON Export**: Structured data for integration with other tools
- **Markdown Documentation**: Easy-to-share analysis reports

### Web Interface
- **Interactive Dashboard**: Modern web interface for log analysis and visualization
- **Project Management**: Organize and manage multiple log analysis projects
- **Real-time Analysis**: Live log monitoring with WebSocket support
- **Export Options**: Export analysis results in various formats

## 🚀 Getting Started

### Installation

**Quick Install (Recommended):**
```bash
# Download and run the installer
curl -fsSL https://raw.githubusercontent.com/your-username/loglens/main/install.sh | bash
```

**Manual Installation:**
```bash
# Clone the repository
git clone https://github.com/your-username/loglens.git
cd loglens

# Build from source (requires Rust)
cargo build --release

# Or install via package manager (when available)
# npm install -g loglens
# brew install loglens
```

### Basic Usage

#### Analyzing Log Files

```bash
# Analyze a log file with default settings
loglens --file application.log

# Filter by log level (ERROR, WARN, INFO, DEBUG)
loglens --file server.log --level ERROR

# Use a specific AI provider
loglens --file app.log --provider openai

# Generate HTML report
loglens --file system.log --output html --report analysis.html
```

#### Analyzing Command Output

```bash
# Run a command and analyze its output
loglens --exec "docker logs my-container" --level WARN

# Analyze system logs
loglens --exec "journalctl -u nginx" --provider claude
```

#### Web Interface

```bash
# Start the web server
loglens-web

# Access the interface at http://localhost:8080
```

## 📊 How It Works

### 1. Log Collection
LogLens can read logs from:
- Local log files
- Command execution output
- Live log streams

### 2. Intelligent Processing
The tool processes logs through several stages:
- **Parsing**: Extracts structured information (timestamp, level, message)
- **Filtering**: Removes noise based on log level
- **Optimization**: Reduces log volume while preserving important information
- **AI Analysis**: Uses AI models to identify patterns and issues

### 3. Advanced Analysis
Beyond basic AI analysis, LogLens provides:
- **Pattern Recognition**: Identifies recurring issues and their frequency
- **Performance Metrics**: Analyzes timing data and identifies bottlenecks
- **Anomaly Detection**: Flags unusual events that might indicate problems
- **Correlation Analysis**: Links related errors to find root causes

### 4. Report Generation
Results are presented in your preferred format:
- **Console**: Quick overview with color-coded sections
- **HTML**: Interactive reports with charts and detailed analysis
- **JSON**: Machine-readable format for automation
- **Markdown**: Easy-to-share documentation format

## 🔧 Configuration

### Setting Up AI Providers

LogLens supports multiple AI providers. Configure your API keys:

**Environment Variables:**
```bash
# OpenAI
export OPENAI_API_KEY="your-api-key-here"

# Claude
export ANTHROPIC_API_KEY="your-api-key-here"

# Gemini
export GEMINI_API_KEY="your-api-key-here"

# OpenRouter
export OPENROUTER_API_KEY="your-api-key-here"
```

**Configuration File:**
Create `~/.config/loglens/config.toml`:
```toml
[providers.openai]
api_key = "your-api-key"
model = "gpt-4"

[providers.claude]
api_key = "your-api-key"
model = "claude-3-sonnet-20240229"
```

### Project-Level Configuration

Create `.loglens.toml` in your project directory:
```toml
default_level = "WARN"
default_provider = "openai"
output_format = "html"

[analysis]
max_log_entries = 1000
enable_patterns = true
enable_performance = true
enable_anomalies = true
```

## 🌐 Web Interface Features

### Dashboard
- **Overview**: Quick summary of recent analyses
- **Project Management**: Organize logs by project
- **Real-time Monitoring**: Live log analysis with automatic updates

### Analysis Views
- **Detailed Analysis**: Comprehensive breakdown of log patterns and issues
- **Pattern Detection**: Visual representation of recurring patterns
- **Performance Metrics**: Charts and graphs showing performance trends
- **Anomaly Detection**: Highlighted unusual log entries

### Export & Sharing
- **Multiple Formats**: Export as HTML, PDF, JSON, or CSV
- **Report Templates**: Customizable report layouts
- **Sharing Options**: Direct links to analysis results

## 💡 Use Cases

### Development & Debugging
```bash
# Analyze application logs during development
loglens --file dev.log --level DEBUG --provider claude

# Monitor test execution
loglens --exec "npm test" --output html --report test-results.html
```

### System Administration
```bash
# Monitor system logs
loglens --file /var/log/syslog --level ERROR

# Analyze service logs
loglens --exec "systemctl status nginx" --provider openai
```

### Production Monitoring
```bash
# Start web interface for production monitoring
loglens-web --port 8080 --production

# Analyze multiple log files
loglens --file /var/log/app/*.log --level WARN --output json
```

## 🛠️ Advanced Features

### MCP Integration
LogLens supports the Model Context Protocol (MCP) for integration with other AI tools:

```bash
# Start MCP server mode
loglens --mcp-server

# Use in MCP mode for JSON I/O
echo '{"logs": ["[ERROR] Connection failed"], "level": "ERROR"}' | loglens --mcp-mode
```

### Custom Analysis
Create custom analysis rules and filters:
```bash
# Custom pattern matching
loglens --file app.log --pattern "timeout|connection.*failed"

# Time-based filtering
loglens --file server.log --since "2024-01-01" --until "2024-01-31"
```

### Batch Processing
Analyze multiple files at once:
```bash
# Process all log files in a directory
loglens --file /path/to/logs/*.log --output-dir ./reports/

# Create summary report
loglens --file *.log --summary --report summary.html
```

## 📈 Performance & Scalability

- **Large File Support**: Handles multi-gigabyte log files efficiently
- **Memory Optimization**: Smart log reduction to manage memory usage
- **Parallel Processing**: Multi-threaded analysis for faster results
- **Streaming Analysis**: Real-time processing of live log streams

## 🔒 Security & Privacy

- **Local Processing**: Logs are processed locally, no data sent to external servers
- **API Key Security**: Secure storage of AI provider credentials
- **Log Anonymization**: Option to remove sensitive information from logs
- **Audit Trail**: Track analysis history and results

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines for more information.

### Development Setup
```bash
# Clone the repository
git clone https://github.com/your-username/loglens.git
cd loglens

# Install development dependencies
cargo install cargo-watch cargo-clippy

# Run tests
cargo test

# Start development server
cargo watch -x "run -- --help"
```

## 📚 Documentation

- [API Documentation](docs/API.md)
- [Configuration Guide](docs/CONFIGURATION.md)
- [User Guide](docs/USER_GUIDE.md)
- [Quality Engineering](docs/QUALITY_ENGINEERING.md)

## 🐛 Troubleshooting

### Common Issues

**API Key Not Found:**
```bash
# Check if API key is set
echo $OPENAI_API_KEY

# Set API key
export OPENAI_API_KEY="your-key"
```

**Permission Denied:**
```bash
# Check file permissions
ls -la your-log-file.log

# Fix permissions
chmod 644 your-log-file.log
```

**Memory Issues:**
```bash
# Reduce log entries for analysis
loglens --file large.log --max-entries 500

# Use streaming mode for large files
loglens --file large.log --streaming
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **AI Development Team**: Claude Sonnet and GLM-4.5 for their exceptional work in planning, developing, and structuring the entire LogLens project
- **Human Contributors**: For the original concept, requirements definition, testing, debugging, and deployment guidance
- **AI Providers**: Thanks to OpenAI, Anthropic, Google, and OpenRouter for their amazing APIs that power LogLens
- **Technologies**: Built with Rust for performance and reliability, with web interface powered by modern web technologies

This project showcases the potential of human-AI collaboration in software development, demonstrating how AI can handle complex technical implementation while human creativity and oversight ensure practical, user-focused solutions.

---

**LogLens** - Making log analysis intelligent and accessible to everyone.

For more information, visit [https://github.com/your-username/loglens](https://github.com/your-username/loglens) or contact us at [support@loglens.dev](mailto:support@loglens.dev).
