# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building
```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Check compilation without building
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Running LogLens
```bash
# Run from source (development)
cargo run -- --help

# Run release binary
./target/release/loglens --help

# Analyze a log file
cargo run -- --file test.log --level ERROR --provider openrouter

# Execute command and analyze output
cargo run -- --exec "some-command" --level WARN --provider claude

# Start MCP server
cargo run -- --mcp-server

# Run in MCP mode (JSON I/O)
echo '{"logs": ["[ERROR] Test"], "level": "ERROR", "provider": "openrouter"}' | cargo run -- --mcp-mode
```

## Project Architecture

### Core Components

**LogLens Library (`src/lib.rs`)**: Main library exposing unified API for log analysis
- Provides both CLI and MCP integration capabilities
- Handles configuration, API key management, and workflow orchestration
- Key methods: `analyze_lines()`, `generate_full_report()`, `process_mcp_request()`

**CLI Binary (`src/main.rs`)**: Command-line interface supporting multiple modes
- File analysis, command execution, MCP server mode, and JSON I/O mode
- Argument parsing with clap, async execution with tokio

**Analysis Engine (`src/analyzer.rs`)**: Core analysis orchestrator
- Integrates AI providers with specialized analyzers (patterns, performance, anomaly, correlation)
- Enhances basic AI analysis with advanced pattern recognition and metrics

### Key Subsystems

**AI Providers (`src/ai_provider/`)**: Pluggable AI backend system
- `mod.rs`: Provider factory and common traits (`AIProvider`, `AnalysisRequest`, `AnalysisResponse`)
- Individual providers: OpenRouter, OpenAI, Claude, Gemini
- Standardized analysis interface with provider-specific implementations

**MCP Server (`src/mcp_server/`)**: Model Context Protocol integration
- Full MCP server implementation using rmcp library
- Tools: `analyze_logs`, `parse_logs`, `filter_logs`
- JSON schema validation for tool parameters

**Log Processing Pipeline**:
1. **Input (`src/input.rs`)**: File reading and command execution
2. **Parser (`src/parser.rs`)**: Structured log entry extraction with regex patterns
3. **Filter (`src/filter.rs`)**: Level-based filtering (ERROR, WARN, INFO, DEBUG)
4. **Slimmer (`src/slimmer.rs`)**: Log reduction to manage AI context limits
5. **Output (`src/output/`)**: Multi-format report generation (console, HTML, JSON, markdown)

**Advanced Analyzers (`src/analyzer/`)**: Specialized analysis modules
- `patterns.rs`: Recurring pattern detection and frequency analysis
- `performance.rs`: Timing statistics, bottleneck identification, performance scoring
- `anomaly.rs`: Unusual pattern detection with confidence scoring
- `correlation.rs`: Cross-error correlation and root cause analysis

**Configuration (`src/config.rs`)**: Hierarchical configuration system
- TOML-based config with defaults for all AI providers
- Environment variable and config file precedence
- Project-level (`.loglens.toml`) and user-level (`~/.config/loglens/config.toml`) support

### Data Flow

1. **Input**: Raw logs from file or command execution
2. **Parsing**: Extract timestamp, level, and message into `LogEntry` structs
3. **Filtering**: Remove entries below specified log level
4. **Slimming**: Reduce log volume for AI analysis
5. **AI Analysis**: Primary analysis using selected provider
6. **Enhancement**: Advanced analysis (patterns, performance, anomalies, correlations)
7. **Report Generation**: Format results in specified output format

### Testing Strategy

- Unit tests in each module verify individual component behavior
- Mock AI provider (`MockProvider`) enables testing without API calls
- Integration tests verify end-to-end log processing pipeline
- MCP server tests validate protocol compliance and tool functionality

### API Key Management

API keys resolved in order of precedence:
1. Command-line parameter (`--api-key`)
2. Environment variables (`OPENROUTER_API_KEY`, `OPENAI_API_KEY`, etc.)
3. Configuration file settings
4. Error if none found

### Output Formats

- **Console**: Human-readable terminal output with sections
- **HTML**: Rich formatted report using Askama templates (`templates/report_template.html`)
- **JSON**: Structured data for programmatic consumption
- **Markdown**: Documentation-friendly format with headers and code blocks

### MCP Integration

LogLens operates as both MCP tool consumer (via JSON I/O mode) and MCP server provider:
- **Server Mode**: Exposes log analysis capabilities to MCP clients
- **Tool Integration**: Can be called by other MCP-aware systems
- **Protocol Compliance**: Full handshake implementation with proper error handling

## Logging Architecture

LogLens uses structured logging via the `tracing` crate for comprehensive observability across all components.

### Log Levels

- **ERROR**: Critical failures preventing operation (auth failures, DB errors, file I/O failures, API failures)
- **WARN**: Non-critical issues needing attention (rate limiting, missing optional config, unexpected conditions)
- **INFO**: Important operational information (service startup/shutdown, major milestones, analysis summaries)
- **DEBUG**: Detailed diagnostics (request/response details, processing steps, performance metrics)

### Backend Logging (Rust)

**Environment Configuration**:
```bash
RUST_LOG=info          # Standard logging
RUST_LOG=debug         # Verbose logging for development
RUST_LOG=loglens=debug # Module-specific debug logging
```

**Components with Enhanced Logging**:
- **CLI Module**: Command processing, file operations, API key resolution, analysis pipeline stages
- **Core Library**: Main API functions (`analyze_lines`, `process_mcp_request`), analyzer workflow, AI provider operations
- **AI Providers**: Provider initialization, request/response handling, authentication errors, rate limiting
- **Input Module**: File reading with encoding detection, command execution with status tracking
- **Web Server**: Startup validation, database initialization, cache/circuit breaker setup, background tasks

**Example Usage**:
```rust
error!("Failed to read file {}: {}", file_path, e);
info!("Analysis completed in {}ms", duration);
debug!("Processing chunk {} of {}", current, total);
```

### Frontend Logging (React)

**Logger Utility** (`utils/logger.ts`):
- Structured logging with levels (DEBUG, INFO, WARN, ERROR)
- Component-based categorization
- Log storage and export capabilities
- Error boundary integration

**Example Usage**:
```typescript
import { logger, LogLevel } from './utils/logger';

// Set log level dynamically
logger.setLogLevel(LogLevel.DEBUG);

// Log with context
logger.error('API', 'Request failed', { endpoint, status, error });
logger.info('Component', 'User action', { action: 'submit', form: 'login' });
```

### Error Context Guidelines

All error logs include:
- What failed (operation, component, function)
- Why it failed (error message, status code)
- Where it failed (module, file, line context)
- Structured error information with stack traces when available
- Clear separation between error types
- Consistent message formatting