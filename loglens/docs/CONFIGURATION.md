# LogLens Configuration Guide

## Overview

LogLens supports flexible configuration through TOML files and environment variables, allowing you to customize AI providers, default settings, and behavior for different environments.

## Configuration Sources

Configuration is loaded in the following priority order:

1. **Command line parameters** (highest priority)
2. **Environment variables**
3. **Project-level config file** (`./.loglens.toml`)
4. **User-level config file** (`~/.config/loglens/config.toml`)
5. **Default values** (lowest priority)

## Configuration File Format

LogLens uses TOML format for configuration files. Here's a complete example:

```toml
# .loglens.toml

[defaults]
provider = "openrouter"
log_level = "ERROR"

[providers.openrouter]
model = "openai/gpt-4"
timeout = 30
max_tokens = 2000
temperature = 0.1
# api_key = "sk-or-..." # Optional - prefer environment variables

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

[providers.anthropic]
# Alias for claude provider
model = "claude-3-sonnet-20240229"
timeout = 30
max_tokens = 3000
temperature = 0.1
```

## Configuration Sections

### [defaults]

Global default settings that apply when not specified elsewhere.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `provider` | String | `"openrouter"` | Default AI provider to use |
| `log_level` | String | `"ERROR"` | Default minimum log level filter |

**Example:**
```toml
[defaults]
provider = "claude"
log_level = "WARN"
```

### [providers.{name}]

Provider-specific configuration for each AI service.

#### Common Provider Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `model` | String | Provider-specific | AI model to use |
| `timeout` | Integer | `30` | Request timeout in seconds |
| `max_tokens` | Integer | `1000` | Maximum tokens in response |
| `temperature` | Float | `0.1` | Randomness in AI responses (0.0-1.0) |
| `api_key` | String | None | API key (prefer environment variables) |

#### Provider-Specific Defaults

**OpenRouter** (`providers.openrouter`)
```toml
[providers.openrouter]
model = "openai/gpt-3.5-turbo"  # Can use any OpenRouter model
timeout = 30
max_tokens = 1000
temperature = 0.1
```

**OpenAI** (`providers.openai`)
```toml
[providers.openai]
model = "gpt-3.5-turbo"         # gpt-4, gpt-3.5-turbo
timeout = 30
max_tokens = 1000
temperature = 0.1
```

**Claude/Anthropic** (`providers.claude` or `providers.anthropic`)
```toml
[providers.claude]
model = "claude-3-sonnet-20240229"  # claude-3-opus, claude-3-sonnet, claude-3-haiku
timeout = 30
max_tokens = 1000
temperature = 0.1
```

**Google Gemini** (`providers.gemini`)
```toml
[providers.gemini]
model = "gemini-pro"            # gemini-pro, gemini-pro-vision
timeout = 30
max_tokens = 1000
temperature = 0.1
```

## Environment Variables

### API Keys

API keys should be set as environment variables for security:

```bash
# OpenRouter
export OPENROUTER_API_KEY="sk-or-v1-..."

# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic/Claude
export ANTHROPIC_API_KEY="sk-ant-..."

# Google Gemini
export GEMINI_API_KEY="AI..."
```

### Other Settings

You can override any configuration setting via environment variables:

```bash
# Override default provider
export LOGLENS_DEFAULT_PROVIDER="claude"

# Override default log level
export LOGLENS_DEFAULT_LOG_LEVEL="INFO"

# Provider-specific model override
export LOGLENS_OPENAI_MODEL="gpt-4"
export LOGLENS_CLAUDE_MODEL="claude-3-opus-20240229"
```

## Configuration File Locations

### Project-Level Configuration

Place `.loglens.toml` in your project directory:

```
my-project/
├── .loglens.toml
├── logs/
└── src/
```

This configuration applies only when running LogLens from within this directory or its subdirectories.

### User-Level Configuration

Create a global configuration file:

```bash
# Create config directory
mkdir -p ~/.config/loglens

# Create config file
cat > ~/.config/loglens/config.toml << 'EOF'
[defaults]
provider = "openrouter"
log_level = "ERROR"

[providers.openrouter]
model = "openai/gpt-4"
temperature = 0.0
EOF
```

This configuration applies to all LogLens usage for your user account.

## Configuration Examples

### Development Environment

For local development with detailed analysis:

```toml
# .loglens.toml (project directory)
[defaults]
provider = "openrouter"
log_level = "INFO"  # More verbose for development

[providers.openrouter]
model = "openai/gpt-3.5-turbo"  # Cost-effective for development
timeout = 60
max_tokens = 2000
temperature = 0.2  # Slightly more creative
```

### Production Environment

For production monitoring with focus on errors:

```toml
# ~/.config/loglens/config.toml (user-level)
[defaults]
provider = "claude"
log_level = "ERROR"  # Focus on critical issues

[providers.claude]
model = "claude-3-sonnet-20240229"  # Reliable and fast
timeout = 30
max_tokens = 4000  # Detailed analysis
temperature = 0.0  # Deterministic responses
```

### Multi-Provider Setup

For comparing results across providers:

```toml
[defaults]
provider = "openrouter"
log_level = "ERROR"

# Primary provider - cost-effective
[providers.openrouter]
model = "openai/gpt-3.5-turbo"
timeout = 30
max_tokens = 1500

# Secondary provider - detailed analysis
[providers.claude]
model = "claude-3-opus-20240229"
timeout = 45
max_tokens = 4000
temperature = 0.0

# Tertiary provider - quick checks
[providers.openai]
model = "gpt-4"
timeout = 20
max_tokens = 1000
```

### High-Volume Scenario

For processing large volumes of logs:

```toml
[defaults]
provider = "openrouter"
log_level = "ERROR"

[providers.openrouter]
model = "openai/gpt-3.5-turbo"  # Fast and cost-effective
timeout = 15  # Short timeout for quick processing
max_tokens = 800  # Concise responses
temperature = 0.0  # Consistent results
```

## Model Selection Guide

### OpenRouter Models

OpenRouter provides access to multiple model families:

```toml
[providers.openrouter]
# OpenAI models via OpenRouter
model = "openai/gpt-4"                    # Best quality, higher cost
model = "openai/gpt-3.5-turbo"           # Good balance, lower cost

# Anthropic models via OpenRouter
model = "anthropic/claude-3-opus"        # Highest quality
model = "anthropic/claude-3-sonnet"      # Good balance
model = "anthropic/claude-3-haiku"       # Fastest, lowest cost

# Google models via OpenRouter
model = "google/gemini-pro"              # Good general purpose
model = "google/gemini-pro-vision"       # For visual content

# Other providers
model = "meta-llama/llama-2-70b-chat"    # Open source option
```

### Direct Provider Models

**OpenAI Direct:**
```toml
[providers.openai]
model = "gpt-4"                          # Latest GPT-4
model = "gpt-4-turbo"                    # Faster GPT-4 variant
model = "gpt-3.5-turbo"                  # Cost-effective option
```

**Claude Direct:**
```toml
[providers.claude]
model = "claude-3-opus-20240229"         # Most capable
model = "claude-3-sonnet-20240229"       # Balanced
model = "claude-3-haiku-20240307"        # Fastest
```

**Gemini Direct:**
```toml
[providers.gemini]
model = "gemini-pro"                     # Standard model
model = "gemini-pro-vision"              # With vision capabilities
```

## Advanced Configuration

### Temperature Settings

Control response creativity and consistency:

```toml
[providers.openrouter]
temperature = 0.0    # Deterministic, consistent responses
temperature = 0.1    # Slightly varied but focused
temperature = 0.5    # Balanced creativity
temperature = 1.0    # Maximum creativity (not recommended for logs)
```

### Timeout Configuration

Adjust timeouts based on your needs:

```toml
[providers.claude]
timeout = 15    # Quick analysis for simple logs
timeout = 30    # Standard timeout for most use cases
timeout = 60    # Extended timeout for complex analysis
timeout = 120   # Maximum timeout for very large logs
```

### Token Limits

Balance detail vs. cost:

```toml
[providers.openai]
max_tokens = 500     # Brief summaries
max_tokens = 1000    # Standard analysis
max_tokens = 2000    # Detailed analysis
max_tokens = 4000    # Comprehensive analysis (higher cost)
```

## Validation and Testing

### Validate Configuration

Test your configuration:

```bash
# Test with a simple log
echo "[ERROR] Test message" | loglens --mcp-mode

# Verify specific provider
loglens --exec "echo '[ERROR] Test'" --provider claude --level ERROR
```

### Configuration Debugging

Check which configuration is being used:

```bash
# Enable debug logging
RUST_LOG=debug loglens --file test.log --level ERROR
```

This will show:
- Configuration file locations checked
- API keys found (masked)
- Provider settings loaded
- Model and parameters used

## Security Considerations

### API Key Security

1. **Never commit API keys to version control**
   ```gitignore
   # .gitignore
   .loglens.toml
   *.local.toml
   ```

2. **Use environment variables**
   ```bash
   # In CI/CD pipelines
   export OPENROUTER_API_KEY="${OPENROUTER_API_KEY}"
   ```

3. **Separate keys by environment**
   ```bash
   # Development
   export OPENROUTER_API_KEY="sk-or-dev-..."

   # Production
   export OPENROUTER_API_KEY="sk-or-prod-..."
   ```

### File Permissions

Secure configuration files:

```bash
# Make config file readable only by owner
chmod 600 ~/.config/loglens/config.toml

# Secure entire config directory
chmod 700 ~/.config/loglens/
```

## Migration and Backup

### Configuration Migration

When upgrading LogLens versions:

1. **Backup existing configuration**
   ```bash
   cp ~/.config/loglens/config.toml ~/.config/loglens/config.toml.backup
   ```

2. **Check for new configuration options**
   ```bash
   loglens --help  # Check for new parameters
   ```

3. **Update configuration format if needed**

### Environment-Specific Configurations

Use different configurations for different environments:

```bash
# Development
ln -sf .loglens.dev.toml .loglens.toml

# Production
ln -sf .loglens.prod.toml .loglens.toml

# Testing
ln -sf .loglens.test.toml .loglens.toml
```

## Troubleshooting Configuration

### Common Issues

1. **Configuration not found**
   ```bash
   # Check current directory
   ls -la .loglens.toml

   # Check user config
   ls -la ~/.config/loglens/config.toml
   ```

2. **Invalid TOML syntax**
   ```bash
   # Validate TOML syntax
   cargo install toml-cli
   toml get .loglens.toml
   ```

3. **API key not loaded**
   ```bash
   # Check environment variables
   env | grep -i api_key

   # Test specific provider
   loglens --provider openrouter --api-key "test-key" --help
   ```

### Configuration Priority Testing

Test configuration precedence:

```bash
# 1. Set environment variable
export LOGLENS_DEFAULT_PROVIDER="claude"

# 2. Create config file with different setting
echo '[defaults]\nprovider = "openai"' > .loglens.toml

# 3. Use command line parameter (should win)
loglens --provider openrouter --exec "echo test" --level ERROR

# Expected: Uses openrouter (command line wins)
```

## Best Practices

1. **Use environment variables for secrets**
2. **Keep project configs minimal and focused**
3. **Document configuration choices in README**
4. **Test configuration changes before deployment**
5. **Use version control for configuration templates**
6. **Implement configuration validation in CI/CD**
7. **Monitor API usage and costs**
8. **Rotate API keys regularly**

## Configuration Schema Reference

Complete configuration schema:

```toml
[defaults]
provider = "string"     # openrouter|openai|claude|anthropic|gemini
log_level = "string"    # ERROR|WARN|INFO|DEBUG

[providers.openrouter]
model = "string"        # Any OpenRouter model
timeout = 30            # seconds
max_tokens = 1000       # integer
temperature = 0.1       # 0.0-1.0
api_key = "string"      # optional, prefer env vars

[providers.openai]
model = "string"        # gpt-4|gpt-3.5-turbo|etc
timeout = 30            # seconds
max_tokens = 1000       # integer
temperature = 0.1       # 0.0-1.0
api_key = "string"      # optional, prefer env vars

[providers.claude]
model = "string"        # claude-3-opus-20240229|etc
timeout = 30            # seconds
max_tokens = 1000       # integer
temperature = 0.1       # 0.0-1.0
api_key = "string"      # optional, prefer env vars

[providers.anthropic]  # alias for claude
model = "string"        # claude-3-opus-20240229|etc
timeout = 30            # seconds
max_tokens = 1000       # integer
temperature = 0.1       # 0.0-1.0
api_key = "string"      # optional, prefer env vars

[providers.gemini]
model = "string"        # gemini-pro|gemini-pro-vision
timeout = 30            # seconds
max_tokens = 1000       # integer
temperature = 0.1       # 0.0-1.0
api_key = "string"      # optional, prefer env vars
```