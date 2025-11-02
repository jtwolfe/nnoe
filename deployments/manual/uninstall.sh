#!/bin/bash
set -euo pipefail

# Uninstall script for NNOE

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

INSTALL_PREFIX="${1:-/usr/local}"
CONFIG_DIR="${2:-/etc/nnoe}"
DATA_DIR="${3:-/var/lib/nnoe}"

echo "NNOE Uninstallation"
echo "==================="
echo "Install prefix: $INSTALL_PREFIX"
echo "Config directory: $CONFIG_DIR"
echo "Data directory: $DATA_DIR"
echo ""

# Confirm uninstallation
read -p "Are you sure you want to uninstall NNOE? This will remove all data. (yes/no): " -r
echo
if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

# Stop and disable service
if systemctl is-active --quiet nnoe-agent; then
    echo "Stopping nnoe-agent service..."
    sudo systemctl stop nnoe-agent
fi

if systemctl is-enabled --quiet nnoe-agent; then
    echo "Disabling nnoe-agent service..."
    sudo systemctl disable nnoe-agent
fi

# Remove systemd service file
if [ -f /etc/systemd/system/nnoe-agent.service ]; then
    echo "Removing systemd service..."
    sudo rm -f /etc/systemd/system/nnoe-agent.service
    sudo systemctl daemon-reload
fi

# Remove binary
if [ -f "$INSTALL_PREFIX/bin/nnoe-agent" ]; then
    echo "Removing binary..."
    sudo rm -f "$INSTALL_PREFIX/bin/nnoe-agent"
fi

# Ask about removing config and data
read -p "Remove configuration directory ($CONFIG_DIR)? (yes/no): " -r
echo
if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    if [ -d "$CONFIG_DIR" ]; then
        echo "Removing configuration directory..."
        sudo rm -rf "$CONFIG_DIR"
    fi
fi

read -p "Remove data directory ($DATA_DIR)? (yes/no): " -r
echo
if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    if [ -d "$DATA_DIR" ]; then
        echo "Removing data directory..."
        sudo rm -rf "$DATA_DIR"
    fi
fi

# Remove user (optional)
read -p "Remove nnoe user? (yes/no): " -r
echo
if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    if id -u nnoe &> /dev/null; then
        echo "Removing nnoe user..."
        sudo userdel nnoe
    fi
fi

echo ""
echo "Uninstallation complete!"

