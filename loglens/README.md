# LogLens

**AI-Powered Log Analysis Platform**

LogLens is a comprehensive log analysis platform that combines AI-powered insights with modern web technology. It features a React-based frontend, Rust backend, and support for multiple AI providers (OpenAI, Claude, Gemini, OpenRouter) to deliver intelligent log analysis, pattern detection, anomaly identification, and actionable recommendations.

---

## ✨ Features

### 🎯 Core Capabilities

- **AI-Powered Analysis**: Leverage OpenAI, Claude, Gemini, or OpenRouter for intelligent log interpretation
- **Multi-Provider Support**: Switch between AI providers based on your needs and API availability
- **Real-Time Analysis**: Stream analysis results with live progress tracking
- **Pattern Detection**: Automatically identify recurring patterns and trends in logs
- **Anomaly Detection**: Discover unusual patterns and outliers with confidence scoring
- **Performance Metrics**: Track timing statistics, bottlenecks, and performance scoring
- **Error Correlation**: Find relationships between errors and identify root causes
- **Smart Recommendations**: Get actionable insights and remediation suggestions

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
- **Executive Summary**: High-level overview with confidence scoring
- **Error Analysis Dashboard**: Categorized error breakdown with severity levels
- **Pattern Detection**: Visual representation of recurring patterns and frequencies
- **Performance Metrics**:
  - Timeline charts showing event distribution
  - Performance scoring with bottleneck identification
  - Timing statistics and trends
- **Anomaly Detection**: Unusual patterns with confidence levels
- **Recommendations**: Prioritized action items for issue resolution

#### ⚙️ Settings & Configuration
- **AI Provider Configuration**:
  - Select from OpenAI, Claude, Gemini, OpenRouter, or Mock (testing)
  - API key management with provider-specific setup instructions
  - Model selection with context limits
  - Model caching and refresh
- **Analysis Settings**:
  - Max log lines to analyze (100-10,000)
  - Default log level (ERROR, WARN, INFO, DEBUG)
  - Custom timeout configuration (60-1800 seconds)
- **UI Settings**:
  - Toggle timestamps display
  - Toggle line numbers
  - Dark mode support

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
- **React Query**: Efficient data fetching and caching
- **WebSocket Support**: Real-time streaming of analysis results
- **SQLite Database**: Persistent storage with SQLx for type-safe queries
- **Circuit Breakers**: Resilient API calls with automatic retry logic
- **Caching Layer**: In-memory caching for improved performance
- **Structured Logging**: Comprehensive tracing with configurable log levels

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

### Settings
- `GET /api/settings` - Get settings
- `PUT /api/settings` - Update settings
- `POST /api/settings/models/fetch` - Fetch available models

### System
- `GET /api/health` - Health check
- `GET /api/dashboard/stats` - Dashboard statistics

---

## 🔒 Security

- Input validation and sanitization on all user inputs
- UUID validation for resource identifiers
- File size limits (default 50MB)
- API key encryption at rest
- CORS configuration for web requests
- SQL injection protection via parameterized queries
- XSS prevention through React's built-in escaping

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
