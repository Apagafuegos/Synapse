#!/bin/bash

# LogLens Uninstallation Script
# Removes all LogLens components and data

set -e  # Exit on any error

echo "🗑️  LogLens Uninstallation Script"
echo "================================"

# Check if LogLens is installed
if ! command -v loglens &> /dev/null; then
    echo "⚠️  LogLens not found in PATH"
else
    echo "📍 Found LogLens at: $(which loglens)"
fi

# Stop running processes
echo "🔄 Stopping any running LogLens processes..."
pkill -f loglens || true
sleep 1

# Stop systemd service if enabled
if systemctl --user is-active --quiet loglens-mcp 2>/dev/null; then
    echo "🔄 Stopping systemd service..."
    systemctl --user stop loglens-mcp || true
fi

if systemctl --user is-enabled --quiet loglens-mcp 2>/dev/null; then
    echo "🔄 Disabling systemd service..."
    systemctl --user disable loglens-mcp || true
fi

# Remove binary
if [[ -f "$HOME/.local/bin/loglens" ]]; then
    echo "🗑️  Removing binary from ~/.local/bin..."
    rm -f "$HOME/.local/bin/loglens"
fi

# Remove data directory
read -p "Remove data directory ~/.loglens? This will delete all databases and projects. (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  Removing data directory..."
    rm -rf ~/.loglens
else
    echo "📁 Keeping data directory ~/.loglens"
fi

# Remove systemd service file
if [[ -f "$HOME/.config/systemd/user/loglens-mcp.service" ]]; then
    echo "🗑️  Removing systemd service file..."
    rm -f "$HOME/.config/systemd/user/loglens-mcp.service"
    systemctl --user daemon-reload || true
fi

# Check PATH and suggest cleanup
if [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "💡 Note: ~/.local/bin is still in your PATH"
    echo "   If you want to remove it from PATH, edit your ~/.bashrc or ~/.zshrc"
fi

echo ""
echo "✅ Uninstallation complete!"
echo "   LogLens has been removed from your system"