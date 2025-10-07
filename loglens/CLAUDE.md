# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building
```bash
# Build entire workspace
cargo build

# Build specific package
cargo build -p loglens-core
cargo build -p loglens-web
cargo build -p loglens-cli
cargo build -p loglens-wasm

# Release build
cargo build --release

# Check compilation without building
cargo check
cargo check -p loglens-web
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p loglens-core
cargo test -p loglens-web

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests with features
cargo test -p loglens-core --features project-management
cargo test -p loglens-core --features mcp-server
```

### Running LogLens

**Web Server (Primary Interface)**:
```bash
# Start web server (auto-starts frontend)
cargo run -p loglens-web

# With custom port
PORT=8080 cargo run -p loglens-web

# With debug logging
RUST_LOG=debug cargo run -p loglens-web
```

**CLI (Legacy)**:
```bash
# Analyze a log file
cargo run -p loglens-cli -- --file test.log --level ERROR --provider openrouter

# Execute command and analyze output
cargo run -p loglens-cli -- --exec "journalctl -u myservice" --provider claude

# Start MCP server
cargo run -p loglens-cli -- --mcp-server
```

**Frontend Development**:
```bash
cd loglens-web/frontend-react

# Install dependencies
npm install

# Build WASM module
npm run build:wasm

# Start dev server with hot reload
npm run dev

# Type checking
npm run type-check

# Production build
npm run build
```

### Database Management

**Important**: LogLens uses a **single, unified database** at `<project-root>/data/loglens.db`. The database is automatically created on first launch. No `DATABASE_URL` environment variable is required.

```bash
# Database is auto-created at: data/loglens.db
# To use a custom location (optional):
export LOGLENS_DATABASE_PATH=/custom/path/to/loglens.db

# Run migrations (from loglens-web directory)
cd loglens-web
sqlx migrate run

# Create new migration
sqlx migrate add migration_name

# For offline compilation (when DATABASE_URL not available)
export SQLX_OFFLINE=true
cargo check
```

## Project Architecture

### Workspace Structure

LogLens is organized as a Cargo workspace with four crates:

- **loglens-core**: Core analysis engine (library)
- **loglens-web**: Web server and API (binary + library)
- **loglens-cli**: Command-line interface (binary)
- **loglens-wasm**: WebAssembly module for frontend

### Core Components (loglens-core)

**Main Library** (`src/lib.rs`):
- Unified API for log analysis across CLI, web, and MCP integrations
- Key functions: `analyze_lines()`, `generate_full_report()`, `process_mcp_request()`
- Configuration and API key management

**AI Provider System** (`src/ai_provider/`):
- Pluggable architecture supporting multiple AI backends
- `mod.rs`: Factory pattern with `AIProvider` trait
- Provider implementations: OpenRouter, OpenAI, Claude, Gemini
- Mock provider for testing without API calls
- Unified request/response types: `AnalysisRequest`, `AnalysisResponse`

**Analysis Engine** (`src/analyzer.rs`):
- Orchestrates AI analysis with specialized analyzers
- Coordinates: pattern detection, performance analysis, anomaly detection, correlation analysis
- Provides `AnalysisConfig` for customization and `AnalysisProgress` for tracking

**Advanced Analyzers** (`src/analyzer/`):
- `patterns.rs`: Frequency analysis and recurring pattern detection
- `performance.rs`: Timing statistics, bottleneck identification, scoring
- `anomaly.rs`: Statistical anomaly detection with confidence levels
- `correlation.rs`: Cross-error correlation and root cause analysis

**Log Processing Pipeline**:
1. `input.rs`: File reading with encoding detection, command execution
2. `parser.rs`: Regex-based structured log extraction → `LogEntry` structs
3. `filter.rs`: Level-based filtering (ERROR, WARN, INFO, DEBUG, TRACE)
4. `slimmer.rs`: Smart log reduction for AI context limits
5. `output/`: Multi-format report generation (console, HTML, JSON, markdown)

**Configuration System** (`src/config.rs`):
- TOML-based hierarchical configuration
- Precedence: CLI args > env vars > config file > defaults
- Project-level: `.loglens.toml`
- User-level: `~/.config/loglens/config.toml`
- API key resolution with multiple fallbacks

**Database Path Resolution** (`src/db_path.rs`):
- Centralized database path management
- Auto-detects project root by searching for workspace `Cargo.toml`
- Creates `data/` directory automatically
- Functions: `get_database_path()`, `get_data_dir()`, `ensure_data_dir()`
- Override via `LOGLENS_DATABASE_PATH` environment variable

**Project Management** (`src/project/`) - Feature: `project-management`:
- `init.rs`: Project initialization with auto-detection
- `detect.rs`: Project type detection (Node.js, Python, Rust, etc.)
- `database.rs`: SQLite schema initialization and migration
- `queries.rs`: Type-safe database operations for projects and analyses
- `registry.rs`: Global project registry management
- `validate.rs`: Project structure validation and repair

**MCP Server** (`src/mcp_server/`) - Feature: `mcp-server`:
- Full Model Context Protocol implementation using `rmcp` library
- Tools: `analyze_logs`, `parse_logs`, `filter_logs`, `add_log_file`, `get_analysis`, `query_analyses`
- JSON schema validation for tool parameters
- Async tool execution with progress tracking

### Web Backend (loglens-web)

**Server Architecture** (`src/main.rs`):
- Axum web framework with tower middleware
- Async runtime via Tokio
- SQLite database with SQLx for type-safe queries
- Static file serving for React frontend
- WebSocket support for real-time updates

**Application State** (`src/lib.rs`):
```rust
pub struct AppState {
    pub db: Database,                    // SQLite connection pool
    pub config: WebConfig,               // Server configuration
    pub circuit_breakers: Arc<...>,      // Resilient API calls
    pub cache_manager: Arc<...>,         // In-memory caching
    pub streaming_hub: Arc<...>,         // Real-time streaming
    pub streaming_manager: Arc<...>,     // Streaming source management
    pub optimized_db: Arc<...>,          // Optimized DB operations
    pub metrics_collector: Arc<...>,     // Performance metrics
}
```

**Key Subsystems**:
- `handlers/`: API route handlers for projects, files, analysis, streaming, knowledge, export
- `database.rs`: Connection pooling, migrations, schema management
- `cache.rs`: LRU cache with TTL for performance optimization
- `circuit_breaker.rs`: Fault-tolerant external API calls
- `streaming/`: Real-time log streaming from files, commands, TCP, HTTP
- `performance.rs`: Database query optimization and indexing
- `middleware/`: Logging, metrics collection, error handling
- `validation.rs`: Request validation and sanitization

**Streaming Architecture** (`src/streaming/`):
- Multiple source types: file tailing, command output, TCP listener, HTTP endpoint, stdin
- Per-project isolation with resource management
- Configurable buffers and parsers
- Automatic restart with exponential backoff
- Real-time statistics and metrics

### Frontend (loglens-web/frontend-react)

**Technology Stack**:
- React 18 with TypeScript
- Vite for fast builds and HMR
- React Query for data fetching and caching
- React Router for navigation
- Tailwind CSS for styling
- Recharts for data visualization
- WebAssembly for performance-critical operations

**Key Components** (`src/components/`):
- `Dashboard.tsx`: Project overview with statistics
- `ProjectList.tsx`: Project management interface
- `AnalysisView.tsx`: Comprehensive analysis results display
- `Settings.tsx`: AI provider and system configuration
- `StreamingManager.tsx`: Real-time log streaming interface
- `KnowledgeBase.tsx`: Problem-solution knowledge management

**Services** (`src/services/`):
- `api.ts`: Centralized API client with error handling
- `websocket.ts`: WebSocket connection management
- Type-safe API calls using TypeScript interfaces

### WASM Module (loglens-wasm)

- High-performance client-side log parsing
- Rust code compiled to WebAssembly
- Integration with React via wasm-bindgen
- Build: `wasm-pack build --target web --out-dir pkg`

## Data Flow

### Log Analysis Pipeline

1. **Input Acquisition**:
   - File upload (web) or file path (CLI)
   - Command execution output
   - Real-time streaming source

2. **Parsing**:
   - Regex-based pattern matching
   - Structured extraction: timestamp, level, message, metadata
   - Multiple format support: JSON, syslog, common log formats

3. **Filtering**:
   - Level-based filtering (configurable threshold)
   - Pattern-based inclusion/exclusion
   - Time range filtering

4. **Slimming**:
   - Smart reduction for AI context limits
   - Preserves error context and patterns
   - Modes: error-focused, representative, time-based

5. **AI Analysis**:
   - Provider selection (OpenRouter, OpenAI, Claude, Gemini)
   - Context preparation with metadata
   - Parallel analysis for large datasets

6. **Enhancement**:
   - Pattern detection (frequency, confidence)
   - Performance metrics (timing, bottlenecks)
   - Anomaly detection (statistical, confidence scoring)
   - Correlation analysis (cross-error, root cause)

7. **Report Generation**:
   - Executive summary with key insights
   - Categorized error breakdown
   - Visual charts and graphs
   - Actionable recommendations
   - Knowledge base integration

8. **Storage & Export**:
   - Database persistence (SQLite)
   - Multi-format export (HTML, PDF, JSON, CSV, Markdown)
   - Shareable links with expiration

## Database Architecture

### Single Unified Database

LogLens uses **one SQLite database** at `<project-root>/data/loglens.db`:
- Auto-created on first launch
- WAL mode for concurrent access
- Connection pooling for performance
- Schema migrations via SQLx

**Tables**:
- `projects`: Project metadata and configuration
- `files`: Uploaded log files with metadata
- `analyses`: Analysis runs with status and results
- `analysis_results`: Detailed analysis data
- `streaming_sources`: Real-time streaming configuration
- `knowledge_entries`: Problem-solution knowledge base
- `exports`: Export metadata and shareable links

**Key Principles**:
- Single source of truth
- No manual `DATABASE_URL` configuration
- Automatic directory creation
- Override via `LOGLENS_DATABASE_PATH` if needed

## API Key Management

API keys are resolved in the following order of precedence:

1. **Command-line parameter**: `--api-key`
2. **Environment variables**: `OPENROUTER_API_KEY`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`
3. **Configuration file**: `.loglens.toml` or `~/.config/loglens/config.toml`
4. **Error if none found**

Web interface stores API keys in settings (encrypted at rest).

## Testing Strategy

### Backend Tests
- **Unit tests**: In each module for isolated component behavior
- **Integration tests**: End-to-end pipeline verification
- **Mock provider**: `MockProvider` for testing without API calls
- **Feature-gated tests**: Tests requiring `project-management` or `mcp-server` features

### Frontend Tests
- **Component tests**: React Testing Library
- **Integration tests**: User flow testing
- **E2E tests**: Playwright for critical paths

### MCP Tests
- **Protocol compliance**: JSON-RPC 2.0 validation
- **Tool execution**: Input/output verification
- **Error handling**: Graceful failure scenarios

## Logging and Observability

LogLens uses structured logging via the `tracing` crate:

**Log Levels**:
- **ERROR**: Critical failures (auth failures, DB errors, API failures)
- **WARN**: Non-critical issues (rate limiting, missing config)
- **INFO**: Important operations (startup, analysis completion)
- **DEBUG**: Detailed diagnostics (request/response, processing steps)
- **TRACE**: Verbose debugging (rare, use sparingly)

**Configuration**:
```bash
# Standard logging
RUST_LOG=info

# Verbose logging
RUST_LOG=debug

# Module-specific logging
RUST_LOG=loglens_core=debug,loglens_web=info

# Component-specific
RUST_LOG=loglens_web::handlers=trace
```

**Error Context Guidelines**:
- Include: what failed, why it failed, where it failed
- Provide structured error information with context
- Include stack traces for unexpected errors
- Use consistent message formatting

## MCP Integration

LogLens provides full Model Context Protocol support for AI assistant integration:

**Available Tools**:
- `analyze_logs`: Direct log content analysis with AI
- `parse_logs`: Structured log parsing with metadata extraction
- `filter_logs`: Log filtering by level and patterns
- `add_log_file`: File-based analysis with automatic processing
- `get_analysis`: Retrieve detailed analysis results by ID
- `query_analyses`: Search and filter analyses with criteria

**Claude Desktop Integration**:
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

## Feature Flags

Important cargo features:

- **project-management**: Project initialization, registry, database operations
- **mcp-server**: Model Context Protocol server implementation

Build with features:
```bash
cargo build --features project-management
cargo build --features mcp-server
cargo build --all-features
```

## Common Patterns

### Adding a New AI Provider

1. Create provider module in `loglens-core/src/ai_provider/`
2. Implement `AIProvider` trait
3. Add provider to factory in `ai_provider/mod.rs`
4. Add configuration to `config.rs`
5. Add tests with mock HTTP responses

### Adding a New Analysis Type

1. Create analyzer module in `loglens-core/src/analyzer/`
2. Implement analysis logic with confidence scoring
3. Integrate in `analyzer.rs` orchestrator
4. Add UI components in frontend for visualization
5. Update API endpoints if needed

### Adding a New Streaming Source

1. Add source type to `streaming/sources.rs`
2. Implement `StreamingSource` trait
3. Add configuration validation
4. Update frontend streaming manager UI
5. Add tests for source lifecycle

## Performance Considerations

- **Database**: Use batch operations, proper indexing, connection pooling
- **Caching**: Cache expensive operations (analysis results, parsed logs)
- **Streaming**: Buffer management, backpressure handling
- **Frontend**: React Query for caching, lazy loading, code splitting
- **WASM**: Use for CPU-intensive operations (parsing, filtering)

## Security Notes

- **Input Validation**: All user input sanitized and validated
- **SQL Injection**: Parameterized queries via SQLx
- **XSS**: React built-in protection, no `dangerouslySetInnerHTML`
- **API Keys**: Encrypted at rest, never logged
- **File Uploads**: Size limits, type validation, sandboxed processing
- **CORS**: Configurable origins, strict by default
