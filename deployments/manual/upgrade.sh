#!/bin/bash
set -euo pipefail

# Upgrade script for NNOE

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

INSTALL_PREFIX="${1:-/usr/local}"
CONFIG_DIR="${2:-/etc/nnoe}"
DATA_DIR="${3:-/var/lib/nnoe}"

echo "NNOE Upgrade"
echo "============"
echo "Install prefix: $INSTALL_PREFIX"
echo "Config directory: $CONFIG_DIR"
echo "Data directory: $DATA_DIR"
echo ""

# Check if NNOE is installed
if [ ! -f "$INSTALL_PREFIX/bin/nnoe-agent" ]; then
    echo "Error: NNOE agent not found at $INSTALL_PREFIX/bin/nnoe-agent"
    echo "Please install NNOE first using install.sh"
    exit 1
fi

# Backup current version
CURRENT_VERSION=$("$INSTALL_PREFIX/bin/nnoe-agent" --version 2>/dev/null || echo "unknown")
echo "Current version: $CURRENT_VERSION"

# Create backup
BACKUP_DIR="$DATA_DIR/backup-$(date +%Y%m%d-%H%M%S)"
echo "Creating backup in $BACKUP_DIR..."
sudo mkdir -p "$BACKUP_DIR"
sudo cp "$INSTALL_PREFIX/bin/nnoe-agent" "$BACKUP_DIR/nnoe-agent.old"
sudo cp -r "$CONFIG_DIR" "$BACKUP_DIR/config" 2>/dev/null || true

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Stop service for upgrade
if systemctl is-active --quiet nnoe-agent; then
    echo "Stopping nnoe-agent service..."
    sudo systemctl stop nnoe-agent
    SERVICE_WAS_RUNNING=true
else
    SERVICE_WAS_RUNNING=false
fi

# Build new version
echo "Building new NNOE agent..."
cd "$PROJECT_ROOT"
cargo build --release --package nnoe-agent

# Install new binary
echo "Installing new binary..."
sudo cp target/release/nnoe-agent "$INSTALL_PREFIX/bin/nnoe-agent.new"
sudo chmod +x "$INSTALL_PREFIX/bin/nnoe-agent.new"

# Validate new binary
echo "Validating new binary..."
if ! "$INSTALL_PREFIX/bin/nnoe-agent.new" --version &>/dev/null; then
    echo "Error: New binary validation failed. Restoring old version..."
    sudo rm -f "$INSTALL_PREFIX/bin/nnoe-agent.new"
    if [ "$SERVICE_WAS_RUNNING" = true ]; then
        sudo systemctl start nnoe-agent
    fi
    exit 1
fi

NEW_VERSION=$("$INSTALL_PREFIX/bin/nnoe-agent.new" --version 2>/dev/null || echo "unknown")
echo "New version: $NEW_VERSION"

# Validate configuration with new binary
if [ -f "$CONFIG_DIR/agent.toml" ]; then
    echo "Validating configuration with new binary..."
    if ! "$INSTALL_PREFIX/bin/nnoe-agent.new" validate -c "$CONFIG_DIR/agent.toml"; then
        echo "Warning: Configuration validation failed with new version"
        read -p "Continue anyway? (yes/no): " -r
        echo
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            echo "Upgrade cancelled. Restoring old version..."
            sudo rm -f "$INSTALL_PREFIX/bin/nnoe-agent.new"
            if [ "$SERVICE_WAS_RUNNING" = true ]; then
                sudo systemctl start nnoe-agent
            fi
            exit 1
        fi
    fi
fi

# Replace binary
echo "Replacing binary..."
sudo mv "$INSTALL_PREFIX/bin/nnoe-agent.new" "$INSTALL_PREFIX/bin/nnoe-agent"

# Restart service if it was running
if [ "$SERVICE_WAS_RUNNING" = true ]; then
    echo "Starting nnoe-agent service..."
    sudo systemctl start nnoe-agent
    
    # Wait for service to be ready
    sleep 3
    
    if systemctl is-active --quiet nnoe-agent; then
        echo "Service started successfully"
    else
        echo "Warning: Service failed to start. Check logs: sudo journalctl -u nnoe-agent"
    fi
fi

echo ""
echo "Upgrade complete!"
echo "Backup saved to: $BACKUP_DIR"
echo "Old version: $CURRENT_VERSION"
echo "New version: $NEW_VERSION"

