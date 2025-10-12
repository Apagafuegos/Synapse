#!/bin/bash

# Synapse Uninstallation Script
# Removes all Synapse components and data

set -e  # Exit on any error

echo "🗑️  Synapse Uninstallation Script"
echo "================================"

# Check if Synapse is installed
if ! command -v synapse &> /dev/null; then
    echo "⚠️  Synapse not found in PATH"
else
    echo "📍 Found Synapse at: $(which synapse)"
fi

# Stop running processes
echo "🔄 Stopping any running Synapse processes..."
pkill -f synapse || true
sleep 1

# Stop systemd service if enabled
if systemctl --user is-active --quiet synapse-mcp 2>/dev/null; then
    echo "🔄 Stopping systemd service..."
    systemctl --user stop synapse-mcp || true
fi

if systemctl --user is-enabled --quiet synapse-mcp 2>/dev/null; then
    echo "🔄 Disabling systemd service..."
    systemctl --user disable synapse-mcp || true
fi

# Remove binary
if [[ -f "$HOME/.local/bin/synapse" ]]; then
    echo "🗑️  Removing binary from ~/.local/bin..."
    rm -f "$HOME/.local/bin/synapse"
fi

# Remove data directory
read -p "Remove data directory ~/.synapse? This will delete all databases and projects. (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗑️  Removing data directory..."
    rm -rf ~/.synapse
else
    echo "📁 Keeping data directory ~/.synapse"
fi

# Remove systemd service file
if [[ -f "$HOME/.config/systemd/user/synapse-mcp.service" ]]; then
    echo "🗑️  Removing systemd service file..."
    rm -f "$HOME/.config/systemd/user/synapse-mcp.service"
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
echo "   Synapse has been removed from your system"