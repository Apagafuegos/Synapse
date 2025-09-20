# CRUSH.md - LogLens Development Guidelines

## Build Commands
- **Basic build**: `cargo build`
- **Release build**: `cargo build --release`
- **Build with TUI feature**: `cargo build --features tui`
- **Build with all features**: `cargo build --features tui,visualization`
- **Check build**: `cargo check`
- **Clean build**: `cargo clean && cargo build`

## Test Commands
- **Run all tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Run integration tests**: `cargo test --test integration`
- **With coverage**: `cargo tarpaulin` (if installed)
- **Benchmarks**: `cargo bench`
- **Doc tests**: `cargo test --doc`

## Lint/Format Commands
- **Format code**: `cargo fmt`
- **Check formatting**: `cargo fmt --check`
- **Lint code**: `cargo clippy`
- **Lint with all features**: `cargo clippy --all-targets --all-features`
- **Check before commit**: `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`

## Async/Await Implementation Guidelines

### Core Principles
- **Use async/await** for I/O-bound operations (network, file I/O, AI API calls)
- **Keep sync for CPU-bound** operations (parsing, calculations, data processing)
- **Feature-based compilation**: Use `#[cfg(feature = "tui")]` for async-dependent functionality

### Async Trait Implementation
All AI providers must implement the `LlmProvider` trait with proper async methods:

```rust
#[async_trait::async_trait]
impl LlmProvider for MyProvider {
    async fn analyze_logs(&self, request: LogAnalysisRequest) -> Result<LogAnalysisResponse, AiError> {
        // AI API call implementation
    }
    
    async fn generate_recommendations(&self, analysis: &str) -> Result<String, AiError> {
        // Recommendation generation logic
    }
    
    async fn health_check(&self) -> Result<ProviderHealth, AiError> {
        // Health check logic
    }
}
```

### Borrow Checker Patterns
- **Avoid multiple mutable borrows** of the same struct in async contexts
- **Clone early** when needed across await points
- **Use `Arc<Mutex<T>>`** for shared state across async tasks
- **Prefer borrowing** over cloning when possible

### Error Handling in Async Context
- Use `anyhow::Error` for complex error chains
- Convert to `String` for CLI error display
- Provide context with `.context()` for better error messages

## AI Integration Implementation

### Provider Registry Pattern
The AI system uses a registry pattern for managing multiple LLM providers:

```rust
// Create registry with configuration
let mut registry = ProviderRegistry::new(config_manager)?;

// Get default provider
let provider = registry.get_provider(&registry.get_default_provider_name())?;

// Test provider health
let health = registry.test_provider("openai").await?;
```

### AI Coordinator Usage
The `AIAnalysisCoordinator` provides high-level AI operations:

```rust
// Create coordinator
let mut coordinator = AIAnalysisCoordinator::new(config_manager, registry)?;

// Analyze logs with AI
let response = coordinator.analyze_logs(log_entries, request).await?;

// Generate recommendations
let recommendations = coordinator.generate_recommendations(analysis, None).await?;

// Get provider information
let info = coordinator.get_provider_info("openai").await?;
```

### Configuration for AI Features
AI functionality is configured through `AiConfig`:

```rust
// Example AI configuration
ai_config: AiConfig {
    enabled: true,
    default_provider: "openai".to_string(),
    providers: HashMap::from([
        ("openai".to_string(), ProviderConfig {
            api_key: "sk-...".to_string(),
            model: "gpt-4".to_string(),
            max_tokens: 4000,
            timeout_seconds: 30,
            // ... other provider settings
        })
    ]),
    analysis: AnalysisConfig {
        enabled: true,
        depth: AnalysisDepth::Detailed,
        auto_summarize: true,
        max_context_entries: 100,
    },
}
```

## Process Monitoring with AI

### Real-time Monitoring
For real-time process monitoring with AI analysis:

```bash
# Build with TUI feature for async support
cargo build --features tui

# Monitor a command with AI analysis
cargo run --features tui -- run --command "npm test" --follow --analysis-trigger "error"
```

### Batch Analysis
For post-execution analysis:

```bash
# Run command and analyze after completion
cargo run -- run --command "make test" --no-follow

# This will automatically analyze logs when process completes
```

### Configuration for Process Monitoring
Process monitoring is configured through `ProcessMonitoringConfig`:

```rust
process_monitoring: ProcessMonitoringConfig {
    enabled: true,
    buffer: BufferConfig {
        max_size: 10000,
        flush_interval_ms: 5000,
        auto_flush: true,
        keep_recent_lines: 1000,
    },
    triggers: TriggerConfig {
        enabled: true,
        cooldown_seconds: 60,
        max_triggers_per_minute: 10,
    },
    analysis: ProcessAnalysisConfig {
        enabled: true,
        depth: AnalysisDepth::Detailed,
        auto_analyze: true,
        batch_size: 100,
    },
}
```

## Usage Examples

### Basic Log Analysis with AI
```bash
# Analyze log file with AI summarization
cargo run -- analyze --file app.log --ai-summary

# Analyze with specific AI provider
cargo run -- analyze --file app.log --ai-provider "anthropic"

# Get AI recommendations for issues found
cargo run -- analyze --file app.log --ai-recommendations
```

### Process Monitoring Examples
```bash
# Monitor a build process with real-time AI analysis
cargo run --features tui -- run --command "cargo build" --follow

# Run tests with AI-triggered analysis on errors
cargo run --features tui -- run --command "npm test" --analysis-trigger "fail|error"

# Monitor with custom AI analysis depth
cargo run --features tui -- run --command "make test" --analysis-depth "comprehensive"
```

### AI Provider Management
```bash
# Test all configured AI providers
cargo run --features tui -- config test-provider all

# Test specific provider
cargo run --features tui -- config test-provider openai

# List available providers
cargo run -- ai list-providers

# Get provider information
cargo run --features tui -- ai provider-info openai
```

### Advanced AI Commands
```bash
# Generate recommendations from log analysis
cargo run -- ai recommend --input "error logs from app.log" --provider openai

# Custom AI analysis with prompt
cargo run -- ai analyze --input "system logs" --focus "security" --provider anthropic
```

## Code Style Guidelines

### Rust Standards
- **Edition**: 2024
- **Line length**: Keep lines readable, aim for <100 characters when possible
- **Naming**: PascalCase for types/structs/enums, snake_case for variables/functions
- **Imports**: Group by standard library, then external crates, then local modules

### Async Code Standards
- **Function naming**: Use clear, descriptive names for async functions
- **Error propagation**: Use `?` operator consistently in async functions
- **Cancellation**: Support graceful cancellation where appropriate
- **Timeouts**: Implement reasonable timeouts for external API calls

### Error Handling
- Use `anyhow` for main application errors in binaries
- Use `thiserror` for library crate custom errors
- Return `Result<T, String>` for simple CLI errors
- Use descriptive error messages that help users troubleshoot
- In async contexts, provide context for async operations

### Code Patterns
- **Structs**: Use derive macros liberally (Debug, Clone, Serialize, Deserialize)
- **Enums**: Provide Display and FromStr implementations where appropriate
- **Traits**: Define clear interfaces for extensibility (like LlmProvider)
- **Builders**: Use builder pattern for complex struct construction
- **Constants**: Use const for static values, avoid magic numbers
- **Async traits**: Use `#[async_trait::async_trait]` for trait methods

### Dependencies
- Use feature flags for optional functionality (tui, visualization)
- Prefer established crates: clap (CLI), serde (serialization), chrono (time), regex (patterns)
- For async: tokio (runtime), async-trait (trait methods), futures (utilities)
- Minimize dependencies, check if functionality exists in std before adding crates

### Documentation
- Document public APIs with /// comments
- Explain complex async logic or performance-sensitive code
- Document async behavior, including cancellation and timeout behavior
- No comments on trivial code - let the code be self-documenting

### Testing Async Code
- **Unit tests**: Test sync parts of async functions separately
- **Integration tests**: Use `tokio::test` for async function tests
- **Mock async dependencies**: Use mock objects for external async services
- **Test error cases**: Ensure async error handling works correctly
- **Test timeouts**: Verify timeout behavior in async operations

## Performance Guidelines

### Async Performance
- **Avoid blocking operations** in async contexts
- **Use appropriate concurrency**: tokio::spawn for CPU-bound work in async contexts
- **Batch operations**: Group multiple small async operations when possible
- **Connection pooling**: Reuse connections for network async operations

### Memory Management
- **Be mindful of clones** across await points
- **Use Arc/Mutex** sparingly for shared async state
- **Stream processing**: Prefer streaming over loading large datasets in memory

## Debugging Async Code

### Common Issues
- **Borrow checker errors**: Usually need to clone or restructure code
- **Deadlocks**: Check for circular waits in async mutex usage
- **Timeout issues**: Ensure all async operations have reasonable timeouts
- **Resource leaks**: Ensure all async resources are properly cleaned up

### Debugging Tools
- **Logging**: Use tracing crate for async operation logging
- **Tokio console**: Use tokio-console for async task visualization
- **Debug assertions**: Add debug checks for async invariants

## Deployment

### Feature Flags
- **Default**: Basic functionality without async/TUI
- **tui**: Enables async runtime and TUI interface
- **visualization**: Enables plotting and charting features
- **all**: Enables all features

### Binary Compilation
- **Minimal**: `cargo build --no-default-features` (basic sync functionality)
- **Full**: `cargo build --features tui,visualization` (all features)
- **Release**: Always use `--release` for production builds

## Contributing

### Async Code Review Checklist
- [ ] All async trait implementations use `#[async_trait::async_trait]`
- [ ] Proper error handling with context for async operations
- [ ] No blocking operations in async contexts
- [ ] Appropriate timeouts for external async calls
- [ ] Memory usage considered across await points
- [ ] Tests cover both success and error async cases
- [ ] Documentation explains async behavior
- [ ] Feature flags properly used for optional async functionality

### Pull Request Process
1. Ensure all async code follows the guidelines above
2. Test with both default and tui features
3. Run full test suite including async integration tests
4. Update documentation for new async functionality
5. Ensure CI/CD pipeline passes for all feature combinations
- Write unit tests for core functionality
- Use integration tests for end-to-end workflows
- Test error conditions and edge cases
- Aim for good test coverage, especially business logic

### Performance Considerations
- Use efficient data structures (VecDeque for queues, HashMap for lookups)
- Process logs in streaming fashion, not loading entire files into memory
- Use trait objects (Box<dyn T>) when dynamic dispatch is needed
- Benchmark performance-critical paths

### Feature Organization
- Core functionality in lib.rs modules
- Feature-gated code behind #[cfg(feature = "...")] attributes
- Clean separation between CLI/binary concerns and library logic</content>
<parameter name="file_path">CRUSH.md