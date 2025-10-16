#!/bin/bash

# Synapse Binary Installation Script
# Downloads pre-compiled binaries instead of compiling from source

VERSION=${1:-latest}
INSTALL_DIR="$HOME/.local/bin"

echo "🚀 Synapse Binary Installation Script"
echo "===================================="
echo "Version: $VERSION"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $OS in
    linux)
        BINARY_OS="linux"
        ;;
    windows|mingw*|cygwin*)
        BINARY_OS="windows"
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

case $ARCH in
    x86_64)
        BINARY_ARCH="x86_64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Determine filename
if [[ "$BINARY_OS" == "windows" ]]; then
    FILENAME="synapse-${BINARY_OS}-${BINARY_ARCH}.zip"
    BINARY_NAME="synapse.exe"
else
    FILENAME="synapse-${BINARY_OS}-${BINARY_ARCH}.tar.gz"
    BINARY_NAME="synapse"
fi

# Get download URL
if [[ "$VERSION" == "latest" ]]; then
    RELEASE_INFO=$(curl -s https://api.github.com/repos/$(git config remote.origin.url | sed 's/.*github.com[:\/]\([^\/]*\/[^\/]*\).*/\1/')/releases/latest)
    DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep "browser_download_url.*$FILENAME" | cut -d '"' -f 4)
else
    DOWNLOAD_URL="https://github.com/$(git config remote.origin.url | sed 's/.*github.com[:\/]\([^\/]*\/[^\/]*\).*/\1/')/releases/download/v$VERSION/$FILENAME"
fi

if [[ -z "$DOWNLOAD_URL" ]]; then
    echo "❌ Could not find download URL for $FILENAME"
    exit 1
fi

echo "📥 Downloading: $FILENAME"
echo "📍 URL: $DOWNLOAD_URL"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
cd /tmp
curl -L "$DOWNLOAD_URL" -o "$FILENAME"

if [[ "$BINARY_OS" == "windows" ]]; then
    7z x "$FILENAME" "$BINARY_NAME"
else
    tar xzf "$FILENAME" "$BINARY_NAME"
fi

# Install binary
mv "$BINARY_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✅ Installation complete!"
echo "📍 Binary installed to: $INSTALL_DIR/$BINARY_NAME"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Add $INSTALL_DIR to your PATH:"
    echo "   export PATH=\"\$PATH:$INSTALL_DIR\""
fi

echo ""
echo "Usage:"
echo "  synapse --help"
echo "  synapse --file /path/to/log.log"