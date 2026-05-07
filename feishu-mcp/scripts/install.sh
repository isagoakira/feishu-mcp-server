#!/bin/bash
# Install script for feishu-mcp-server

set -e

echo "Installing feishu-mcp-server..."

# Create config directory
mkdir -p ~/.feishu-mcp

# Copy example config if not exists
if [ ! -f ~/.feishu-mcp/config.yaml ]; then
    cp config.example.yaml ~/.feishu-mcp/config.yaml
    echo "Created config at ~/.feishu-mcp/config.yaml"
    echo "Please edit it with your App ID and App Secret"
fi

# Build release
echo "Building release..."
cargo build --release

# Install binary
install target/release/feishu-mcp ~/.feishu-mcp/feishu-mcp || cp target/release/feishu-mcp ~/.feishu-mcp/feishu-mcp

echo "Installation complete!"
echo "Run '~/.feishu-mcp/feishu-mcp' to start the server"