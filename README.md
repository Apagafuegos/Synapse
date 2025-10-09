# LogLens

**AI-Powered Log Analysis Platform**

LogLens is an intelligent log analysis platform that leverages multiple AI providers to deliver automated insights, pattern detection, anomaly identification, and actionable recommendations from your log files. With a modern web interface and powerful CLI, LogLens transforms complex log data into understandable analysis.

---

## ✨ Features

### 🎯 Core Capabilities

- **AI-Powered Analysis**: Choose from OpenAI, Claude, Gemini, or OpenRouter for intelligent log interpretation
- **Smart Pattern Detection**: Automatically identifies recurring patterns with frequency analysis and confidence scoring
- **Anomaly Detection**: Statistical analysis to discover unusual patterns and outliers
- **Performance Metrics**: Track timing statistics, bottlenecks, and performance trends
- **Error Correlation**: Find relationships between different errors with strength scoring
- **Knowledge Base**: Built-in problem-solution knowledge management with sharing capabilities
- **Real-time Streaming**: Live log streaming from files, commands, TCP, HTTP, and stdin
- **Multi-format Support**: Parse JSON, syslog, common formats, and custom log patterns
- **Project Management**: Organize log files and analyses by project

### 🌐 Web Interface

- **Dashboard**: Overview of all projects, recent activity, and key metrics
- **Project Management**: Create, organize, and manage multiple projects
- **Analysis View**: Comprehensive results with interactive charts and visualizations
- **Settings**: Configure AI providers, API keys, and analysis preferences
- **Knowledge Base**: Create and manage problem-solution pairs
- **Streaming Manager**: Real-time log streaming interface
- **Export Options**: Download reports in HTML, PDF, JSON, CSV, or Markdown

### 🤖 MCP Integration

Full Model Context Protocol support for AI assistant integration with tools like:
- `analyze_logs`: Direct log content analysis
- `parse_logs`: Structured log parsing with metadata
- `filter_logs`: Log filtering by level and patterns
- `add_log_file`: File-based analysis
- `get_analysis`: Retrieve detailed results
- `query_analyses`: Search and filter analyses

---

## 🚀 Installation

### Quick Install (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/loglens.git
cd loglens

# Run the installation script
chmod +x install.sh
./install.sh
```

This builds the release binary, builds the frontend, and installs to `~/.local/bin/loglens`.

**Add to PATH** (if needed):
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
```

### Docker Deployment

```bash
# Using docker-compose
docker-compose up -d

# Or build and run manually
docker build -t loglens .
docker run -d -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/uploads:/app/uploads \
  -e OPENROUTER_API_KEY=your_key_here \
  loglens --dashboard
```

---

## 📖 Usage

### Web Dashboard (Primary Interface)

```bash
# Start the web dashboard
loglens --dashboard

# On a custom port
loglens --dashboard --port 8080
```

Navigate to `http://localhost:3000` in your browser.

**Basic Workflow**:
1. Configure your AI provider in Settings (add your API key)
2. Create a new project
3. Upload log files (.log, .txt formats supported)
4. Click "Analyze" to start AI-powered analysis
5. View results with patterns, anomalies, and recommendations
6. Export results in your preferred format

### MCP Server Mode

```bash
# Start MCP server (stdio mode - for AI assistants)
loglens --mcp-server

# Start MCP server (HTTP mode - for web clients)
loglens --mcp-server --mcp-transport http --mcp-port 3001
```

### Project Management

```bash
# Initialize LogLens in current directory
loglens init

# Initialize in specific directory
loglens init --path /path/to/project

# Link project to global registry
loglens link

# List all linked projects
loglens list-projects

# Validate project links
loglens validate-links

# Validate and auto-repair issues
loglens validate-links --repair

# Unlink a project
loglens unlink
```

---

## ⚙️ Configuration

### AI Provider Setup

The web interface provides easy configuration for all AI providers:

**Supported Providers**:
- **OpenRouter**: Multi-model gateway (recommended)
- **OpenAI**: GPT models
- **Claude**: Anthropic models  
- **Gemini**: Google models
- **Mock**: Testing without API calls

**API Key Priority**:
1. Web interface settings (encrypted storage)
2. Environment variables (`OPENROUTER_API_KEY`, `OPENAI_API_KEY`, etc.)
3. Configuration file

### Environment Variables

```bash
# Server Configuration
PORT=3000                    # Web server port
RUST_LOG=info               # Log level

# Database (auto-configured, override if needed)
LOGLENS_DATABASE_PATH=/custom/path/loglens.db

# AI Provider API Keys (if not using web interface)
OPENROUTER_API_KEY=sk-or-...
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...

# Performance
MAX_FILE_SIZE=52428800      # Max upload size (50MB)
```

---

## 📊 Key Features in Detail

### Analysis Types

**Executive Summary**: High-level overview with key insights and confidence scoring

**Error Analysis**: Categorized breakdown with:
- Severity levels and trends
- Timeline charts
- Frequency analysis
- Root cause suggestions

**Pattern Detection**: 
- Recurring pattern identification
- Frequency analysis with confidence scores
- Pattern evolution tracking

**Performance Analysis**:
- Timing statistics and trends
- Bottleneck identification
- Performance scoring
- Threshold monitoring

**Anomaly Detection**:
- Statistical anomaly identification
- Confidence levels and categorization
- Alert integration

**Correlation Analysis**:
- Cross-error relationship mapping
- Timeline correlation
- Impact assessment

### Knowledge Base

Create and manage problem-solution pairs:
- Public sharing across projects
- Full-text search with filtering
- Tag-based categorization
- Usage tracking and statistics
- AI-generated solution suggestions

### Streaming Sources

Real-time log streaming from multiple sources:
- **File Streaming**: Tail log files with automatic restart
- **Command Streaming**: Stream output from system commands
- **TCP Listener**: Accept logs via TCP connections
- **HTTP Endpoint**: Receive logs via HTTP POST
- **Stdin**: Stream from standard input

### Export Formats

- **HTML**: Interactive reports with charts
- **PDF**: Professional formatted reports
- **JSON**: Structured data for programmatic use
- **CSV**: Tabular data for spreadsheets
- **Markdown**: Documentation-friendly format

---

## 🔒 Security & Reliability

### Security Features

- **API Key Encryption**: Secure storage with encryption at rest
- **Input Validation**: Comprehensive sanitization and validation
- **CORS Protection**: Configurable cross-origin request security
- **SQL Injection Prevention**: Parameterized queries
- **XSS Protection**: Built-in React XSS protection
- **File Validation**: Size limits and type checking

### Reliability Features

- **Error Recovery**: Graceful handling with automatic retry
- **Health Monitoring**: Comprehensive system health checks
- **Circuit Breakers**: Resilient external API calls
- **Connection Pooling**: Efficient database management
- **Data Persistence**: Reliable SQLite storage with WAL mode

---

## 🐛 Troubleshooting

### Common Issues

**"loglens command not found"**:
```bash
# Add to PATH
export PATH="$HOME/.local/bin:$PATH"

# Or reinstall
./install.sh
```

**"Frontend not loading"**:
```bash
# Check if frontend was built
ls ~/.local/bin/frontend/index.html

# Rebuild if needed
cd loglens-web/frontend-react
npm install && npm run build
```

**"API connection refused"**:
- Ensure firewall allows port 3000
- Check if port is already in use
- Verify installation completed successfully

**Database issues**:
```bash
# Database is auto-created in ~/.loglens/data/
# Check permissions
ls -la ~/.loglens/data/loglens.db

# Reset if needed (note: loses all data)
rm ~/.loglens/data/loglens.db
loglens --dashboard  # Will recreate automatically
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug loglens --dashboard

# For MCP server (stdio mode - use only for debugging)
RUST_LOG=debug loglens --mcp-server --mcp-transport http
```

---

## 📞 Support

For issues, questions, or feature requests:
- Open an issue on GitHub
- Check the troubleshooting guide above
- Enable debug logging for detailed error information

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Happy log analyzing! 🔍**

Transform your logs from noise into actionable insights with LogLens.