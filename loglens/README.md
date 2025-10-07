# LogLens

**AI-Powered Log Analysis Platform**

LogLens is a comprehensive log analysis platform that combines AI-powered insights with modern web technology. It features a React-based frontend, Rust backend, and support for multiple AI providers (OpenAI, Claude, Gemini, OpenRouter) to deliver intelligent log analysis, pattern detection, anomaly identification, and actionable recommendations.

---

## ✨ Features

### 🎯 Core Capabilities

- **AI-Powered Analysis**: Leverage OpenAI, Claude, Gemini, or OpenRouter for intelligent log interpretation
- **Multi-Provider Support**: Switch between AI providers based on your needs and API availability
- **Advanced Log Processing**: Multi-format parsing (JSON, syslog, common formats) with intelligent filtering
- **Real-Time Analysis**: Stream analysis results with live progress tracking via WebSocket
- **Pattern Detection**: Automatically identify recurring patterns with frequency analysis and confidence scoring
- **Anomaly Detection**: Discover unusual patterns and outliers with statistical analysis
- **Performance Metrics**: Track timing statistics, bottlenecks, and performance scoring
- **Error Correlation**: Cross-error correlation analysis with strength scoring
- **Root Cause Analysis**: Intelligent root cause identification with confidence levels
- **Smart Recommendations**: Get actionable insights and prioritized remediation suggestions
- **Multi-Log Analysis**: Comparative analysis across multiple log files
- **Knowledge Base**: Built-in knowledge management for problem-solution pairs
- **Streaming Sources**: Real-time log streaming from files, commands, TCP, HTTP, and stdin

### 🌐 Web Interface Features

#### 📊 Dashboard
- Overview of all projects and recent activity
- Key metrics: total projects, analyses this week, average processing time, critical errors
- Quick access to recent projects

#### 📁 Project Management
- Create and organize log analysis projects
- Upload log files (.log, .txt formats)
- Track project history and analysis count
- Search and filter projects

#### 📈 Advanced Analysis View
- **Executive Summary**: High-level overview with confidence scoring and key insights
- **Error Analysis Dashboard**: Categorized error breakdown with severity levels and trends
- **Pattern Detection**: Visual representation of recurring patterns with frequency analysis
- **Performance Metrics**:
  - Timeline charts showing event distribution and trends
  - Performance scoring with bottleneck identification
  - Timing statistics and trend analysis
  - Threshold monitoring and alerts
- **Anomaly Detection**: Statistical anomaly detection with confidence levels and categorization
- **Error Correlation**: Cross-error correlation analysis with strength scoring
- **Multi-Log Analysis**: Comparative analysis across multiple files with trend identification
- **Recommendations**: Prioritized action items with confidence scoring
- **Knowledge Integration**: Related knowledge base entries and suggested solutions

#### ⚙️ Settings & Configuration
- **AI Provider Configuration**:
  - Select from OpenAI, Claude, Gemini, OpenRouter, or Mock (testing)
  - API key management with provider-specific setup instructions
  - Model selection with context limits and caching
  - Automatic model refresh and availability checking
- **Analysis Settings**:
  - Max log lines to analyze (100-10,000)
  - Default log level (ERROR, WARN, INFO, DEBUG)
  - Custom timeout configuration (60-1800 seconds)
  - Advanced analysis options (correlation, anomaly detection)
- **Streaming Configuration**:
  - Multiple streaming sources (file, command, TCP, HTTP, stdin)
  - Buffer sizes and timeout settings
  - Parser configuration for different log formats
  - Project isolation and source management
- **UI Settings**:
  - Dark/light theme with system preference detection
  - Toggle timestamps display
  - Toggle line numbers
  - Real-time update preferences
- **Knowledge Base**:
  - Create and manage problem-solution pairs
  - Public sharing across projects
  - Search and filtering options
  - Usage tracking and statistics

#### 🎨 User Experience
- **Dark Mode**: Full dark/light theme support with system preference detection
- **Responsive Design**: Mobile-friendly interface using Tailwind CSS
- **Error Boundaries**: Graceful error handling with recovery options
- **Loading States**: Smooth loading animations and skeleton screens
- **Real-time Updates**: Auto-refresh for running analyses
- **Export Options**: Export analysis results in multiple formats

### 🛠️ Technical Features

- **WASM Integration**: WebAssembly for high-performance client-side processing
- **Type Safety**: Full TypeScript implementation in frontend
- **React Query**: Efficient data fetching and caching with optimistic updates
- **WebSocket Support**: Real-time streaming of analysis results and progress
- **SQLite Database**: Persistent storage with SQLx for type-safe queries and WAL mode
- **Circuit Breakers**: Resilient API calls with automatic retry logic and rate limiting
- **Caching Layer**: In-memory caching for improved performance with TTL management
- **Structured Logging**: Comprehensive tracing with configurable log levels
- **MCP Integration**: Full Model Context Protocol support for AI assistant integration
- **Streaming Architecture**: Real-time log processing with multiple source support
- **Advanced Analytics**: Statistical analysis, correlation detection, and trend identification
- **Knowledge Management**: Built-in knowledge base with sharing and search capabilities
- **Export System**: Multi-format export (HTML, PDF, JSON, CSV, Markdown) with templates
- **Performance Optimization**: Chunking, batching, and asynchronous processing

---

## 🚀 Installation

### Prerequisites

- **Rust** (≥1.70): Install from [rustup.rs](https://rustup.rs/)
- **Node.js** (≥18): For frontend development
- **Docker** (optional): For containerized deployment

### Option 1: Quick Install (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/loglens.git
cd loglens

# Run the installation script
chmod +x install.sh
./install.sh
```

This will:
- Build the release binary
- Install to `~/.local/bin/loglens`
- Make the binary executable
- Verify installation

**Add to PATH** (if needed):
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"
```

### Option 2: Docker Deployment

```bash
# Using docker-compose
docker-compose up -d

# Or build and run manually
docker build -t loglens .
docker run -d -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/uploads:/app/uploads \
  -e OPENROUTER_API_KEY=your_key_here \
  loglens
```

Access the web interface at: `http://localhost:3000`

### Option 3: Manual Build

```bash
# Build the web server (includes frontend)
cd loglens-web
cargo build --release

# The binary will be at target/release/loglens-web
```

### Option 4: Development Setup

```bash
# Install frontend dependencies
cd loglens-web/frontend-react
npm install

# Build WASM module
npm run build:wasm

# Start development server
npm run dev

# In another terminal, start the backend
cd ../..
cargo run --bin loglens-web
```

---

## 📖 Usage

### Web Application

```bash
# Start the web server
loglens-web

# Or with custom configuration
LOGLENS_DATABASE_URL=sqlite:data/loglens.db \
PORT=3000 \
RUST_LOG=info \
loglens-web
```

Then navigate to `http://localhost:3000` in your browser.

**Workflow**:
1. Navigate to **Settings** and configure your AI provider and API key
2. Go to **Projects** and create a new project
3. Upload a log file to the project
4. Click **Analyze** to start AI-powered analysis
5. View detailed results including patterns, anomalies, and recommendations
6. Export results in your preferred format

### CLI (Legacy - Deprecated)

While the web interface is the primary way to use LogLens, the CLI is still available:

```bash
# Analyze a log file
loglens --file /path/to/app.log --provider openrouter --level ERROR

# Execute a command and analyze output
loglens --exec "journalctl -u myservice" --provider claude --level WARN

# Start MCP server mode
loglens --mcp-server
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Database
LOGLENS_DATABASE_URL=sqlite:data/loglens.db

# Server
PORT=3000                    # Web server port
WORKERS=4                    # Number of worker threads

# AI Provider API Keys
OPENROUTER_API_KEY=sk-or-...
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...

# Performance
CACHE_ENABLED=true           # Enable in-memory caching
CACHE_TTL=3600              # Cache TTL in seconds
CACHE_MAX_ENTRIES=1000      # Maximum cache entries
MAX_FILE_SIZE=52428800      # Max upload size (50MB)
MAX_ANALYSIS_LINES=50000    # Max lines per analysis

# Logging
RUST_LOG=info               # Log level: error, warn, info, debug, trace

# Paths
LOGLENS_FRONTEND_DIR=/app/frontend-react/dist
LOGLENS_UPLOAD_DIR=/app/uploads
```

### Database Migrations

```bash
# Run migrations
cd loglens-web
sqlx migrate run

# Create migration
sqlx migrate add migration_name
```

### Docker Environment

Create a `.env` file:

```env
OPENROUTER_API_KEY=your_key_here
OPENAI_API_KEY=your_key_here
PORT=3000
RUST_LOG=info
```

Then use with docker-compose:

```bash
docker-compose --env-file .env up -d
```

---

## 🧪 Development

### Frontend Development

```bash
cd loglens-web/frontend-react

# Install dependencies
npm install

# Start dev server with hot reload
npm run dev

# Type checking
npm run type-check

# Linting
npm run lint
npm run lint:fix

# Testing
npm test
npm run test:coverage

# Build for production
npm run build
```

### Backend Development

```bash
# Run in development mode with auto-reload
cargo watch -x 'run --bin loglens-web'

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

### WASM Development

```bash
cd loglens-wasm

# Build WASM module
wasm-pack build --target web --out-dir pkg

# The generated files will be in pkg/
```

---

## 🏗️ Architecture

### Project Structure

```
loglens/
├── loglens-core/           # Core analysis engine (Rust)
│   ├── src/
│   │   ├── ai_provider/    # AI provider integrations
│   │   ├── analyzer.rs     # Main analysis orchestrator
│   │   ├── parser.rs       # Log parsing logic
│   │   ├── filter.rs       # Log filtering
│   │   └── output/         # Output formatters
│   └── Cargo.toml
├── loglens-web/            # Web server (Rust + Axum)
│   ├── frontend-react/     # React frontend
│   │   ├── src/
│   │   │   ├── components/ # React components
│   │   │   ├── pages/      # Page components
│   │   │   ├── hooks/      # Custom hooks
│   │   │   ├── services/   # API services
│   │   │   └── types/      # TypeScript types
│   │   ├── dist/           # Production build
│   │   └── package.json
│   ├── src/
│   │   ├── handlers/       # API route handlers
│   │   ├── models/         # Data models
│   │   ├── middleware/     # Middleware
│   │   └── main.rs         # Server entry point
│   ├── migrations/         # SQLx migrations
│   └── Cargo.toml
├── loglens-wasm/           # WASM module
│   ├── src/lib.rs
│   └── Cargo.toml
├── docker-compose.yml      # Docker orchestration
├── Dockerfile              # Container image
└── install.sh              # Installation script
```

### Technology Stack

**Backend**:
- Rust with Axum web framework
- SQLx for type-safe database queries
- SQLite for data persistence
- Tokio for async runtime
- Tower/Tower-HTTP for middleware
- Tracing for structured logging

**Frontend**:
- React 18 with TypeScript
- Vite for build tooling
- React Query for data fetching
- React Router for navigation
- Tailwind CSS for styling
- Recharts for data visualization
- Heroicons for icons
- WebAssembly for performance-critical operations

**AI Integration**:
- OpenRouter (multi-model gateway)
- OpenAI API
- Anthropic Claude API
- Google Gemini API

---

## 📊 API Endpoints

### Projects
- `GET /api/projects` - List all projects
- `POST /api/projects` - Create project
- `GET /api/projects/:id` - Get project details
- `DELETE /api/projects/:id` - Delete project

### Files
- `GET /api/projects/:id/files` - List project files
- `POST /api/projects/:id/files` - Upload file
- `DELETE /api/projects/:id/files/:file_id` - Delete file

### Analysis
- `POST /api/projects/:id/files/:file_id/analyze` - Start analysis
- `GET /api/projects/:id/analyses` - List analyses
- `GET /api/analyses/:id` - Get analysis details
- `GET /api/analyses/:id/stream` - Stream analysis results (SSE)
- `DELETE /api/analyses/:id` - Delete analysis
- `POST /api/analyses/:id/cancel` - Cancel running analysis

### Streaming
- `GET /api/streaming/sources` - List streaming sources
- `POST /api/streaming/sources` - Create streaming source
- `GET /api/streaming/sources/:id` - Get source details
- `DELETE /api/streaming/sources/:id` - Delete streaming source
- `GET /api/streaming/stats` - Get streaming statistics
- `POST /api/streaming/sources/:id/restart` - Restart streaming source

### Knowledge Base
- `GET /api/knowledge` - List knowledge entries
- `POST /api/knowledge` - Create knowledge entry
- `GET /api/knowledge/:id` - Get knowledge entry
- `PUT /api/knowledge/:id` - Update knowledge entry
- `DELETE /api/knowledge/:id` - Delete knowledge entry
- `GET /api/knowledge/search` - Search knowledge base
- `POST /api/knowledge/:id/share` - Share knowledge entry

### Export
- `POST /api/analyses/:id/export` - Export analysis
- `GET /api/exports/:id/download` - Download export
- `GET /api/exports/:id` - Get export status
- `POST /api/exports/:id/share` - Create shareable link

### MCP Integration
- `POST /api/mcp/analyze` - MCP analysis endpoint
- `GET /api/mcp/tools` - List available MCP tools
- `POST /api/mcp/tools/:tool` - Execute MCP tool

### System
- `GET /api/health` - Health check
- `GET /api/dashboard/stats` - Dashboard statistics

---

## 📡 Streaming Features

### Supported Sources

**File Streaming**: Real-time log file tailing with automatic restart
```bash
# Stream from a log file
curl -X POST http://localhost:3000/api/streaming/sources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "app-logs",
    "source_type": "file", 
    "config": {
      "file_path": "/var/log/app.log",
      "buffer_size": 1000,
      "timeout": 30
    },
    "project_id": "project-uuid"
  }'
```

**Command Streaming**: Stream output from system commands
```bash
curl -X POST http://localhost:3000/api/streaming/sources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "system-logs",
    "source_type": "command",
    "config": {
      "command": "journalctl -f -u nginx",
      "buffer_size": 500,
      "timeout": 60
    },
    "project_id": "project-uuid"
  }'
```

**TCP Listener**: Accept logs via TCP connections
```bash
curl -X POST http://localhost:3000/api/streaming/sources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "tcp-logs",
    "source_type": "tcp",
    "config": {
      "bind_address": "0.0.0.0:5140",
      "buffer_size": 2000,
      "timeout": 10
    },
    "project_id": "project-uuid"
  }'
```

### Streaming Management

- **Buffer Management**: Configurable buffer sizes for performance optimization
- **Parser Configuration**: Support for multiple log formats (JSON, syslog, custom)
- **Statistics Tracking**: Real-time metrics on connection counts, processed logs
- **Error Recovery**: Automatic restart on failures with exponential backoff
- **Project Isolation**: Per-project streaming with resource isolation

---

## 🧠 Knowledge Base

### Creating Knowledge Entries

```typescript
// Via Web Interface
const knowledgeEntry = {
  title: "Database Connection Timeout",
  problem: "Application experiences intermittent database connection timeouts",
  solution: "Increase connection pool size and implement connection retry logic",
  category: "infrastructure",
  tags: ["database", "timeout", "connection-pool"],
  public: true,
  related_patterns: ["connection-refused", "timeout-error"]
};
```

### Knowledge Features

- **Problem-Solution Pairs**: Structured knowledge with problem description and solutions
- **Public Sharing**: Share knowledge across projects for team collaboration
- **Full-Text Search**: Advanced search with filtering and relevance ranking
- **Usage Tracking**: Track which knowledge entries are most useful
- **Tag System**: Categorize knowledge with flexible tagging
- **Pattern Integration**: Link knowledge to detected log patterns
- **AI Suggestions**: Get AI-generated solution suggestions

### Knowledge API

```bash
# Create knowledge entry
curl -X POST http://localhost:3000/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{
    "title": "API Rate Limiting",
    "problem": "Exceeding API rate limits causing 429 errors",
    "solution": "Implement exponential backoff and request queuing",
    "category": "external",
    "tags": ["api", "rate-limit", "429"],
    "public": true
  }'

# Search knowledge
curl "http://localhost:3000/api/knowledge/search?q=database&category=infrastructure"
```

---

## 🤖 MCP Integration

### Model Context Protocol Support

LogLens provides full MCP (Model Context Protocol) support for integration with AI assistants like Claude Desktop:

### Available MCP Tools

1. **analyze_logs**: Direct log content analysis
2. **parse_logs**: Structured log parsing with metadata
3. **filter_logs**: Log filtering by level and patterns
4. **add_log_file**: File-based analysis with automatic processing
5. **get_analysis**: Retrieve detailed analysis results
6. **query_analyses**: Search and filter analyses

### MCP Server Mode

```bash
# Start MCP server
loglens --mcp-server

# With custom configuration
LOGLENS_MCP_PORT=8080 \
LOGLENS_MCP_HOST=localhost \
loglens --mcp-server
```

### Claude Desktop Integration

Add to Claude Desktop configuration:

```json
{
  "mcpServers": {
    "loglens": {
      "command": "loglens",
      "args": ["--mcp-server"]
    }
  }
}
```

### MCP Usage Example

```typescript
// Via MCP-compatible AI assistant
const result = await mcp.call("analyze_logs", {
  logs: ["[ERROR] Database connection failed", "[WARN] Retry attempt 1"],
  level: "ERROR",
  provider: "openrouter",
  options: {
    include_patterns: true,
    include_correlations: true
  }
});
```

---

## 📊 Advanced Analytics

### Multi-Log Analysis

- **Comparative Analysis**: Compare logs across multiple files and time periods
- **Cross-File Patterns**: Identify patterns affecting multiple systems
- **Trend Analysis**: Track pattern evolution over time
- **Correlation Analysis**: Find relationships between different log sources

### Performance Analysis

- **Timing Metrics**: Detailed timing analysis with statistical summaries
- **Bottleneck Detection**: Automatic identification of performance bottlenecks
- **Threshold Monitoring**: Customizable performance thresholds with alerts
- **Resource Utilization**: Track system resource usage patterns

### Anomaly Detection

- **Statistical Analysis**: Advanced statistical methods for anomaly detection
- **Confidence Scoring**: Reliability assessment for detected anomalies
- **Type Classification**: Categorize anomalies (timing, frequency, pattern)
- **Alert Integration**: Real-time anomaly detection with alerting

### Correlation Analysis

- **Cross-Error Correlation**: Find relationships between different errors
- **Root Cause Analysis**: Advanced root cause identification
- **Impact Assessment**: Assess the impact of issues on system performance
- **Timeline Correlation**: Time-based correlation analysis

---

## 📄 Export & Reporting

### Supported Export Formats

- **HTML Reports**: Styled reports with charts and interactive elements
- **PDF Reports**: Professional PDF export via wkhtmltopdf
- **JSON Export**: Structured data for programmatic consumption
- **CSV Export**: Tabular data for spreadsheet analysis
- **Markdown Export**: Documentation-friendly format

### Shareable Reports

```bash
# Create shareable link
curl -X POST http://localhost:3000/api/exports/:id/share \
  -H "Content-Type: application/json" \
  -d '{
    "expires_in": "7d",
    "password": "optional-password",
    "allow_download": true
  }'
```

### Export Features

- **Template Support**: Customizable report templates
- **Chart Inclusion**: Optional chart embedding in exports
- **Correlation Data**: Include/exclude correlation analysis
- **Metadata**: Comprehensive export metadata
- **Password Protection**: Optional password protection for shared links
- **Expiration Control**: Time-limited access to shared reports

---

## 🔒 Security & Reliability

### Security Features

- **Input Validation**: Comprehensive input sanitization and validation
- **API Key Encryption**: Secure storage of API keys with encryption
- **CORS Configuration**: Configurable cross-origin request security
- **SQL Injection Protection**: Parameterized queries prevent injection attacks
- **File Size Limits**: Configurable upload limits with validation
- **XSS Prevention**: Built-in React XSS protection
- **Authentication**: Settings-based API key management

### Reliability Features

- **Error Boundaries**: Graceful error handling in UI with recovery options
- **Retry Logic**: Automatic retry for failed operations with backoff
- **Graceful Degradation**: Fallback functionality during failures
- **Health Checks**: Comprehensive system health monitoring
- **Structured Logging**: Detailed operational logging with tracing
- **Circuit Breakers**: Resilient API call handling
- **Connection Pooling**: Efficient database connection management
- **Data Persistence**: Reliable data storage with WAL mode

---

## 🐛 Troubleshooting

### Frontend Issues

**Build fails with WASM errors**:
```bash
cd loglens-wasm
wasm-pack build --target web --out-dir pkg
```

**API connection refused**:
- Ensure backend is running on correct port
- Check CORS settings if accessing from different origin
- Verify `LOGLENS_FRONTEND_DIR` environment variable

### Database Issues

**Migration errors**:
```bash
# Reset database
rm data/loglens.db
sqlx migrate run
```

**SQLx offline mode**:
```bash
export SQLX_OFFLINE=true
cargo check
```

### Docker Issues

**Port already in use**:
```bash
# Change port in docker-compose.yml or use:
docker-compose down
```

**Volume permissions**:
```bash
chmod -R 755 data/ uploads/
```

---

## 📝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## 🙏 Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- UI powered by [React](https://react.dev/) and [Tailwind CSS](https://tailwindcss.com/)
- AI analysis via [OpenRouter](https://openrouter.ai/), [OpenAI](https://openai.com/), [Anthropic](https://anthropic.com/), and [Google](https://ai.google.dev/)
- Icons by [Heroicons](https://heroicons.com/)
- Charts by [Recharts](https://recharts.org/)

---

## 📞 Support

For issues, questions, or feature requests, please open an issue on GitHub.

**Happy log analyzing! 🔍**
