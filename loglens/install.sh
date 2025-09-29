#!/bin/bash

# LogLens Installation Script
# Compiles and installs LogLens to ~/.local/bin

set -e  # Exit on any error

echo "🚀 LogLens Installation Script"
echo "=============================="

# Check if we're in the correct directory
if [[ ! -f "Cargo.toml" ]] || ! grep -q "name = \"loglens\"" Cargo.toml; then
    echo "❌ Error: This script must be run from the LogLens project directory"
    echo "   Make sure you're in the directory containing Cargo.toml"
    exit 1
fi

# Check if Rust/Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Found Cargo: $(cargo --version)"

# Create local bin directory if it doesn't exist
echo "📁 Creating ~/.local/bin directory..."
mkdir -p ~/.local/bin

# Kill any running loglens processes to avoid "Text file busy" error
echo "🔄 Stopping any running LogLens processes..."
pkill -f loglens || true
sleep 1

# Build release version
echo "🔨 Building LogLens (release mode)..."
cargo build --release

if [[ ! -f "target/release/loglens" ]]; then
    echo "❌ Error: Build failed - executable not found"
    exit 1
fi

# Get file size
SIZE=$(du -h target/release/loglens | cut -f1)
echo "✅ Build successful! Executable size: $SIZE"

# Install to PATH
echo "📦 Installing to ~/.local/bin/loglens..."
cp target/release/loglens ~/.local/bin/
chmod +x ~/.local/bin/loglens

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "⚠️  Warning: ~/.local/bin is not in your PATH"
    echo "   Add this line to your ~/.bashrc or ~/.zshrc:"
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

# Test installation
echo "🧪 Testing installation..."
if command -v loglens &> /dev/null; then
    echo "✅ LogLens installed successfully!"
    echo "📍 Location: $(which loglens)"
    echo ""
    echo "Usage examples:"
    echo "  loglens --help                    # Show help"
    echo "  loglens --file /var/log/app.log   # Analyze log file"
    echo "  loglens --mcp-server             # Start MCP server"
    echo ""
    echo "MCP Server tools available:"
    echo "  - analyze_logs: AI-powered log analysis"
    echo "  - parse_logs: Parse raw logs into structured format"
    echo "  - filter_logs: Filter logs by level and patterns"
else
    echo "❌ Installation failed - loglens not found in PATH"
    exit 1
fi

echo "🎉 Installation complete!"
