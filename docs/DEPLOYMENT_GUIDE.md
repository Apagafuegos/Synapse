# Synapse Deployment Guide

## Quick Start Installation

### Method 1: Linux/macOS Installation (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-repo/Synapse.git
cd Synapse

# Run the installation script
./install.sh

# Verify installation
synapse --help
```

### Method 2: Docker Installation

```bash
# Clone the repository
git clone https://github.com/your-repo/Synapse.git
cd Synapse

# Build and start all services
docker-compose up -d

# Access dashboard at http://localhost:8080
# MCP server available on port 3001
```

## Usage

### Starting the Web Dashboard

```bash
# Method 1: Using installed binary
synapse --dashboard
synapse --dashboard --port 8080

# Method 2: Using Docker
docker run -p 8080:8080 -v ~/.synapse/data:/home/user/.synapse/data synapse --dashboard

### Method 3: Using Docker Compose
docker-compose up dashboard
```

### Starting the MCP Server

```bash
# Method 1: Using installed binary
synapse --mcp-server
synapse --mcp-server --mcp-port 3001

# Method 2: Using Docker
docker run -p 3001:3001 -v ~/.synapse/data:/home/user/.synapse/data synapse --mcp-server

### Method 3: Using Docker Compose
docker-compose up mcp
```

### Project Management

```bash
# Initialize a new project
cd your-project
synapse init

# Link existing project to dashboard
synapse link

# List all projects
synapse list-projects

# View projects in dashboard
synapse --dashboard
```

## Architecture

### Unified Data Directory

All Synapse data is stored in `~/.synapse/`:
```
~/.synapse/
├── data/
│   └── synapse.db          # Unified SQLite database
├── logs/                    # Application logs
└── config/
    └── config.toml          # Global configuration
```

### Services

1. **CLI Tool** (`synapse`)
   - Log analysis commands
   - Project management
   - Dashboard/MCP server launcher

2. **Web Dashboard** (`synapse --dashboard`)
   - Web interface at `http://localhost:8080`
   - Project visualization
   - Analysis results
   - Real-time monitoring

3. **MCP Server** (`synapse --mcp-server`)
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
export SYNAPSE_DATA_DIR="/custom/path"

# API Keys
export OPENROUTER_API_KEY="your-key"
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
export GEMINI_API_KEY="your-key"

# Server ports
export PORT=8080              # Dashboard port
export MCP_PORT=3001          # MCP server port

# Database
export DATABASE_URL="sqlite:/path/to/synapse.db"
```

## Project Integration

### 1. Initialize Project
```bash
cd your-project
synapse init
```

This creates:
```
your-project/.synapse/
├── config.toml       # Project configuration
├── metadata.json     # Project metadata
├── index.db          # Analysis database
├── analyses/         # Analysis results
└── logs/             # Log file cache
```

### 2. Link to Dashboard
```bash
synapse link
```

### 3. View in Dashboard
```bash
synapse --dashboard
# Navigate to Projects tab
```

## Uninstallation

```bash
# Run uninstall script
./uninstall.sh

# Manual removal (if needed)
rm -rf ~/.local/bin/synapse
rm -rf ~/.synapse
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
   ls -la ~/.synapse/
   
   # Reset data directory
   rm -rf ~/.synapse/data
   mkdir -p ~/.synapse/data
   ```

3. **Docker port conflicts**
   ```bash
   # Change ports in docker-compose.yml
   # Or use different ports with CLI flags
   synapse --dashboard --port 8081
   ```

### Logs

```bash
# Application logs
tail -f ~/.synapse/logs/app.log

# Docker logs
docker-compose logs dashboard
docker-compose logs mcp

# Debug logging
RUST_LOG=debug synapse --dashboard
```

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/your-repo/Synapse.git
cd Synapse
cargo build --release

# Install binary
cp target/release/synapse ~/.local/bin/
```

### Running Tests

```bash
cargo test
cargo test --package core
cargo test --package web
```

## Configuration Files

### Global Config: `~/.synapse/config/config.toml`
```toml
data_dir = "~/.synapse/data"
log_level = "info"

[ai]
default_provider = "openrouter"

[database]
path = "~/.synapse/data/synapse.db"

[dashboard]
port = 8080
host = "127.0.0.1"

[mcp_server]
port = 3001
host = "127.0.0.1"
```

### Project Config: `.synapse.toml`
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

If you have an existing Synapse installation:

1. **Backup existing data**
   ```bash
   cp -r ~/.synapse ~/.synapse.backup
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