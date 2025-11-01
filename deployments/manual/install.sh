#!/bin/bash
set -euo pipefail

# Manual installation script for NNOE

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

INSTALL_PREFIX="${1:-/usr/local}"
CONFIG_DIR="${2:-/etc/nnoe}"
DATA_DIR="${3:-/var/lib/nnoe}"

echo "NNOE Manual Installation"
echo "========================"
echo "Install prefix: $INSTALL_PREFIX"
echo "Config directory: $CONFIG_DIR"
echo "Data directory: $DATA_DIR"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Create directories
echo "Creating directories..."
sudo mkdir -p "$CONFIG_DIR"
sudo mkdir -p "$DATA_DIR"/{cache,logs}
sudo mkdir -p "$INSTALL_PREFIX/bin"

# Create user
if ! id -u nnoe &> /dev/null; then
    echo "Creating nnoe user..."
    sudo useradd -r -s /bin/false -d "$DATA_DIR" nnoe
fi

# Build agent
echo "Building NNOE agent..."
cd "$PROJECT_ROOT"
cargo build --release --package nnoe-agent

# Install binary
echo "Installing binary..."
sudo cp target/release/nnoe-agent "$INSTALL_PREFIX/bin/"
sudo chmod +x "$INSTALL_PREFIX/bin/nnoe-agent"

# Install configuration
if [ ! -f "$CONFIG_DIR/agent.toml" ]; then
    echo "Installing configuration..."
    sudo cp agent/examples/agent.toml.example "$CONFIG_DIR/agent.toml"
    sudo chmod 644 "$CONFIG_DIR/agent.toml"
    echo "Please edit $CONFIG_DIR/agent.toml with your settings"
fi

# Install systemd service
echo "Installing systemd service..."
sudo cp "$SCRIPT_DIR/systemd/nnoe-agent.service" /etc/systemd/system/
sudo systemctl daemon-reload

# Set permissions
echo "Setting permissions..."
sudo chown -R nnoe:nnoe "$DATA_DIR"
sudo chmod 755 "$CONFIG_DIR"
sudo chmod 644 "$CONFIG_DIR/agent.toml"

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "1. Edit $CONFIG_DIR/agent.toml"
echo "2. Enable service: sudo systemctl enable nnoe-agent"
echo "3. Start service: sudo systemctl start nnoe-agent"
echo "4. Check status: sudo systemctl status nnoe-agent"

