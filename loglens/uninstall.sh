#!/bin/bash

# LogLens Uninstallation Script
# Removes LogLens from ~/.local/bin

echo "🗑️  LogLens Uninstallation Script"
echo "================================"

# Check if loglens is installed
if ! command -v loglens &> /dev/null; then
    echo "ℹ️  LogLens is not installed or not in PATH"
    exit 0
fi

LOGLENS_PATH=$(which loglens)
echo "📍 Found LogLens at: $LOGLENS_PATH"

# Stop any running processes
echo "🔄 Stopping any running LogLens processes..."
pkill -f loglens || true
sleep 1

# Remove the executable
if [[ -f "$LOGLENS_PATH" ]]; then
    echo "🗑️  Removing $LOGLENS_PATH..."
    rm -f "$LOGLENS_PATH"
    
    if [[ ! -f "$LOGLENS_PATH" ]]; then
        echo "✅ LogLens successfully uninstalled"
    else
        echo "❌ Failed to remove LogLens"
        exit 1
    fi
else
    echo "⚠️  LogLens executable not found"
fi

echo "🎉 Uninstallation complete!"
