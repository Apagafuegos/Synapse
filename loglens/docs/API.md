# LogLens API Documentation

## Overview

LogLens is a Rust library for AI-powered log analysis that supports multiple AI providers and output formats. This document provides comprehensive API documentation for all public interfaces.

## Core API

### LogLens - Main Entry Point

The primary interface for log analysis operations.

```rust
use loglens::LogLens;

// Create new instance with default configuration
let loglens = LogLens::new()?;

// Create with custom configuration
let loglens = LogLens::with_config(config);
```

#### Methods

**`new() -> Result<Self>`**
- Creates a new LogLens instance with default configuration
- Loads configuration from environment and config files
- Returns error if configuration loading fails

**`with_config(config: Config) -> Self`**
- Creates LogLens instance with provided configuration
- Useful for custom configuration scenarios

**`analyze_lines(raw_lines: Vec<String>, level: &str, provider_name: &str, api_key: Option<&str>) -> Result<AnalysisResponse>`**
- Core analysis method for raw log lines
- **Parameters:**
  - `raw_lines`: Vector of log lines to analyze
  - `level`: Minimum log level filter ("ERROR", "WARN", "INFO", "DEBUG")
  - `provider_name`: AI provider ("openrouter", "openai", "claude", "gemini")
  - `api_key`: Optional API key (uses config/env if None)
- **Returns:** Analysis response with insights and findings

**`analyze_file(file_path: &str, level: &str, provider_name: &str, api_key: Option<&str>) -> Result<AnalysisResponse>`**
- Analyze logs from a file
- Reads file and processes through analysis pipeline

**`analyze_command(command: &str, level: &str, provider_name: &str, api_key: Option<&str>) -> Result<AnalysisResponse>`**
- Execute command and analyze its output
- Captures stdout/stderr for analysis

**`generate_full_report(raw_lines: Vec<String>, level: &str, provider_name: &str, api_key: Option<&str>, input_source: &str, output_format: OutputFormat) -> Result<String>`**
- Generate complete formatted report
- Includes metadata, analysis, and statistics

## AI Provider System

### AIProvider Trait

Core abstraction for AI analysis backends.

```rust
#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResponse, AIError>;
}
```

### Data Structures

**`AnalysisRequest`**
```rust
pub struct AnalysisRequest {
    pub logs: Vec<String>,      // Processed log lines
    pub system_prompt: String,   // AI analysis prompt
}
```

**`AnalysisResponse`**
```rust
pub struct AnalysisResponse {
    pub sequence_of_events: String,  // Timeline of events
    pub error_details: String,       // Error analysis
    pub failed_component: String,    // Component identification
}
```

**`AIError`**
```rust
pub enum AIError {
    RequestError(reqwest::Error),     // HTTP/network errors
    InvalidResponse(String),          // Malformed AI responses
    AuthenticationError,              // API key issues
    RateLimited,                     // Rate limiting
    UnsupportedProvider(String),     // Unknown provider
}
```

### Provider Factory

**`create_provider(provider_name: &str, api_key: &str) -> Result<Box<dyn AIProvider>>`**
- Factory function for creating AI providers
- **Supported providers:**
  - `"openrouter"` → OpenRouterProvider
  - `"openai"` → OpenAIProvider
  - `"claude"` | `"anthropic"` → ClaudeProvider
  - `"gemini"` → GeminiProvider

### Individual Providers

#### OpenRouterProvider
```rust
impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self
    pub fn with_model(mut self, model: String) -> Self
}
```
- Default model: "openai/gpt-3.5-turbo"
- Endpoint: https://openrouter.ai/api/v1/chat/completions

#### OpenAIProvider
```rust
impl OpenAIProvider {
    pub fn new(api_key: String) -> Self
    pub fn with_model(mut self, model: String) -> Self
}
```
- Default model: "gpt-3.5-turbo"
- Endpoint: https://api.openai.com/v1/chat/completions

#### ClaudeProvider
```rust
impl ClaudeProvider {
    pub fn new(api_key: String) -> Self
    pub fn with_model(mut self, model: String) -> Self
}
```
- Default model: "claude-3-sonnet-20240229"
- Endpoint: https://api.anthropic.com/v1/messages

#### GeminiProvider
```rust
impl GeminiProvider {
    pub fn new(api_key: String) -> Self
    pub fn with_model(mut self, model: String) -> Self
}
```
- Default model: "gemini-pro"
- Endpoint: https://generativelanguage.googleapis.com/v1/models

## Configuration System

### Config Structure

```rust
pub struct Config {
    pub providers: ProviderConfig,
    pub defaults: DefaultConfig,
}

pub struct ProviderConfig {
    pub openrouter: Option<ProviderSettings>,
    pub openai: Option<ProviderSettings>,
    pub claude: Option<ProviderSettings>,
    pub gemini: Option<ProviderSettings>,
}

pub struct ProviderSettings {
    pub model: Option<String>,
    pub timeout: Option<u64>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub api_key: Option<String>,
}
```

### Methods

**`Config::load() -> Result<Self>`**
- Load configuration from files and environment
- **Search order:**
  1. `./.loglens.toml` (project-level)
  2. `~/.config/loglens/config.toml` (user-level)
  3. Default configuration

**`get_api_key(&self, provider: &str) -> Option<String>`**
- Retrieve API key with precedence:
  1. Environment variable (`{PROVIDER}_API_KEY`)
  2. Configuration file
  3. None

**`get_provider_settings(&self, provider: &str) -> Option<&ProviderSettings>`**
- Get provider-specific settings

## Log Processing Pipeline

### LogEntry Structure

```rust
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: String,
}
```

### Processing Functions

**`parse_log_lines(lines: &[String]) -> Vec<LogEntry>`**
- Parse raw log lines into structured entries
- Extracts timestamps, log levels, and messages
- **Supported patterns:**
  - ISO 8601 timestamps
  - Log levels: ERROR, WARN, INFO, DEBUG, TRACE, FATAL
  - Removes common prefixes and noise

**`filter_logs_by_level(entries: Vec<LogEntry>, min_level: &str) -> Result<Vec<LogEntry>>`**
- Filter logs by minimum severity level
- **Level hierarchy:** ERROR > WARN > INFO > DEBUG

**`slim_logs(entries: Vec<LogEntry>) -> Vec<LogEntry>`**
- Reduce log volume for AI analysis
- **Optimizations:**
  - Consolidates consecutive duplicate messages
  - Truncates very long messages (>500 chars)
  - Simplifies stack traces
  - Removes repetitive patterns

## Advanced Analysis

### Pattern Analysis

```rust
pub struct PatternAnalyzer {
    // Detects recurring patterns and error chains
}

pub struct PatternAnalysis {
    pub recurring_patterns: Vec<RecurringPattern>,
    pub error_chains: Vec<ErrorChain>,
    pub grouped_errors: Vec<ErrorGroup>,
    pub anomalies: Vec<Anomaly>,
}
```

### Performance Analysis

```rust
pub struct PerformanceAnalyzer {
    // Analyzes timing patterns and bottlenecks
}

pub struct PerformanceMetrics {
    pub performance_score: f64,        // 0-100 score
    pub timing_statistics: TimingStats,
    pub bottlenecks: Vec<Bottleneck>,
    pub trend_analysis: Vec<TimeSeriesPoint>,
}
```

### Anomaly Detection

```rust
pub struct AnomalyDetector {
    // Identifies unusual patterns and security issues
}

pub struct AnomalyReport {
    pub anomalies: Vec<Anomaly>,
    pub security_alerts: Vec<SecurityAlert>,
    pub unusual_patterns: Vec<UnusualPattern>,
    pub summary: AnomalySummary,
}
```

### Correlation Analysis

```rust
pub struct CorrelationAnalyzer {
    // Finds relationships between errors and services
}

pub struct CorrelationAnalysis {
    pub error_correlations: Vec<ErrorCorrelation>,
    pub service_dependencies: Vec<ServiceDependency>,
    pub cascading_failures: Vec<CascadingFailure>,
    pub root_causes: Vec<RootCause>,
}
```

## Output System

### OutputFormat Enum

```rust
pub enum OutputFormat {
    Console,    // Terminal-friendly output
    Html,       // Rich HTML report
    Json,       // Machine-readable JSON
    Markdown,   // Documentation format
}
```

### OutputGenerator Trait

```rust
pub trait OutputGenerator {
    fn generate(&self, report: &AnalysisReport) -> Result<String>;
    fn file_extension(&self) -> &str;
}
```

### Report Structures

```rust
pub struct AnalysisReport {
    pub metadata: ReportMetadata,
    pub analysis: AnalysisResponse,
    pub logs: Vec<ProcessedLogEntry>,
    pub stats: ReportStats,
}

pub struct ReportMetadata {
    pub timestamp: String,
    pub provider: String,
    pub total_logs: usize,
    pub filtered_logs: usize,
    pub log_level: String,
    pub input_source: String,
}

pub struct ReportStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub unique_messages: usize,
    pub consolidated_duplicates: usize,
}
```

## MCP Integration

### MCP Server

LogLens provides a Model Context Protocol (MCP) server for integration with AI applications.

```rust
pub struct LogLensServer {
    // MCP server implementation
}

impl LogLensServer {
    pub fn new() -> Self
}
```

### MCP Data Structures

```rust
pub struct McpRequest {
    pub logs: Vec<String>,
    pub level: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub input_source: Option<String>,
    pub output_format: Option<String>,
}

pub struct McpResponse {
    pub analysis: AnalysisResponse,
    pub report: Option<String>,
    pub metadata: McpMetadata,
}
```

### Available MCP Tools

1. **`analyze_logs`** - Full AI analysis of log entries
2. **`parse_logs`** - Parse raw logs into structured format
3. **`filter_logs`** - Filter logs by level and patterns

## Error Handling

All public APIs use `Result<T>` for error handling:

- **Configuration errors:** Invalid TOML, missing files
- **AI provider errors:** Network issues, authentication, rate limits
- **Processing errors:** Invalid log formats, parsing failures
- **I/O errors:** File access, permission issues

## Thread Safety

- All core types implement `Send + Sync`
- AI providers are thread-safe
- Configuration is immutable after loading
- Analyzers maintain internal state but are single-threaded

## Performance Considerations

- **Memory usage:** Log slimming reduces memory footprint
- **API calls:** Batching and retry logic implemented
- **Concurrency:** Async/await for I/O operations
- **Caching:** None implemented (stateless design)

## Example Usage

```rust
use loglens::{LogLens, OutputFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize LogLens
    let loglens = LogLens::new()?;

    // Read log file
    let logs = loglens::read_log_file("app.log").await?;

    // Analyze with AI
    let analysis = loglens.analyze_lines(
        logs,
        "ERROR",
        "openrouter",
        None // Use environment API key
    ).await?;

    // Generate HTML report
    let report = loglens.generate_full_report(
        logs,
        "ERROR",
        "openrouter",
        None,
        "app.log",
        OutputFormat::Html
    ).await?;

    println!("Analysis: {:?}", analysis);
    println!("Report generated: {} bytes", report.len());

    Ok(())
}
```