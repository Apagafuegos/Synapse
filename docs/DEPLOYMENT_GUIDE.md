# LogLens Deployment Guide

## Quick Start Installation

### Method 1: Linux/macOS Installation (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-repo/LogLens.git
cd LogLens

# Run the installation script
./install.sh

# Verify installation
loglens --help
```

### Method 2: Docker Installation

```bash
# Clone the repository
git clone https://github.com/your-repo/LogLens.git
cd LogLens

# Build and start all services
docker-compose up -d

# Access dashboard at http://localhost:8080
# MCP server available on port 3001
```

## Usage

### Starting the Web Dashboard

```bash
# Method 1: Using installed binary
loglens --dashboard
loglens --dashboard --port 8080

# Method 2: Using Docker
docker run -p 8080:8080 -v ~/.loglens/data:/home/user/.loglens/data loglens --dashboard

### Method 3: Using Docker Compose
docker-compose up dashboard
```

### Starting the MCP Server

```bash
# Method 1: Using installed binary
loglens --mcp-server
loglens --mcp-server --mcp-port 3001

# Method 2: Using Docker
docker run -p 3001:3001 -v ~/.loglens/data:/home/user/.loglens/data loglens --mcp-server

### Method 3: Using Docker Compose
docker-compose up mcp
```

### Project Management

```bash
# Initialize a new project
cd your-project
loglens init

# Link existing project to dashboard
loglens link

# List all projects
loglens list-projects

# View projects in dashboard
loglens --dashboard
```

## Architecture

### Unified Data Directory

All LogLens data is stored in `~/.loglens/`:
```
~/.loglens/
├── data/
│   └── loglens.db          # Unified SQLite database
├── logs/                    # Application logs
└── config/
    └── config.toml          # Global configuration
```

### Services

1. **CLI Tool** (`loglens`)
   - Log analysis commands
   - Project management
   - Dashboard/MCP server launcher

2. **Web Dashboard** (`loglens --dashboard`)
   - Web interface at `http://localhost:8080`
   - Project visualization
   - Analysis results
   - Real-time monitoring

3. **MCP Server** (`loglens --mcp-server`)
   - Model Context Protocol server
   - Available on port 3001
   - Tools: `analyze_logs`, `parse_logs`, `filter_logs`

### Docker Services

```bash
# Start full stack
docker-compose up -d

# Start individual services
docker-compose up dashboard
docker-compose up mcp

# Start CLI helper container
docker-compose --profile cli up cli
```

## Environment Variables

```bash
# Data directory override
export LOGLENS_DATA_DIR="/custom/path"

# API Keys
export OPENROUTER_API_KEY="your-key"
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
export GEMINI_API_KEY="your-key"

# Server ports
export PORT=8080              # Dashboard port
export MCP_PORT=3001          # MCP server port

# Database
export DATABASE_URL="sqlite:/path/to/loglens.db"
```

## Project Integration

### 1. Initialize Project
```bash
cd your-project
loglens init
```

This creates:
```
your-project/.loglens/
├── config.toml       # Project configuration
├── metadata.json     # Project metadata
├── index.db          # Analysis database
├── analyses/         # Analysis results
└── logs/             # Log file cache
```

### 2. Link to Dashboard
```bash
loglens link
```

### 3. View in Dashboard
```bash
loglens --dashboard
# Navigate to Projects tab
```

## Uninstallation

```bash
# Run uninstall script
./uninstall.sh

# Manual removal (if needed)
rm -rf ~/.local/bin/loglens
rm -rf ~/.loglens
rm -f ~/.config/systemd/user/mcp.service
```

## Troubleshooting

### Common Issues

1. **Command not found**
   ```bash
   # Check PATH
   echo $PATH | grep -o ~/.local/bin
   
   # Add to PATH if missing
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. **Database errors**
   ```bash
   # Check data directory permissions
   ls -la ~/.loglens/
   
   # Reset data directory
   rm -rf ~/.loglens/data
   mkdir -p ~/.loglens/data
   ```

3. **Docker port conflicts**
   ```bash
   # Change ports in docker-compose.yml
   # Or use different ports with CLI flags
   loglens --dashboard --port 8081
   ```

### Logs

```bash
# Application logs
tail -f ~/.loglens/logs/app.log

# Docker logs
docker-compose logs dashboard
docker-compose logs mcp

# Debug logging
RUST_LOG=debug loglens --dashboard
```

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/your-repo/LogLens.git
cd LogLens
cargo build --release

# Install binary
cp target/release/loglens ~/.local/bin/
```

### Running Tests

```bash
cargo test
cargo test --package core
cargo test --package web
```

## Configuration Files

### Global Config: `~/.loglens/config/config.toml`
```toml
data_dir = "~/.loglens/data"
log_level = "info"

[ai]
default_provider = "openrouter"

[database]
path = "~/.loglens/data/loglens.db"

[dashboard]
port = 8080
host = "127.0.0.1"

[mcp_server]
port = 3001
host = "127.0.0.1"
```

### Project Config: `.loglens.toml`
```toml
name = "my-project"
description = "My awesome project"

[ai]
provider = "openrouter"
model = "deepseek/deepseek-chat-v3.1:free"

[analysis]
default_level = "ERROR"
slim_mode = "light"
```

## Migration from Previous Versions

If you have an existing LogLens installation:

1. **Backup existing data**
   ```bash
   cp -r ~/.loglens ~/.loglens.backup
   ```

2. **Run new installer**
   ```bash
   ./install.sh
   ```

3. **Migrate databases** (if needed)
   ```bash
   # The installer will attempt to migrate existing databases
   # Manual migration may be required for custom setups
   ```

## Support

- **Documentation**: [Link to docs]
- **Issues**: [Link to GitHub Issues]
- **Discussions**: [Link to GitHub Discussions]