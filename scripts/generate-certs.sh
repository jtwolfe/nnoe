#!/bin/bash
set -euo pipefail

# Certificate generation script for Nebula CA

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CERT_DIR="${1:-$PROJECT_ROOT/nebula/certs}"
NODE_NAME="${2:-node1}"

echo "Generating Nebula certificates..."
echo "Certificate directory: $CERT_DIR"
echo "Node name: $NODE_NAME"

mkdir -p "$CERT_DIR"

# Check if nebula-cert is available
if ! command -v nebula-cert &> /dev/null; then
    echo "Error: nebula-cert not found. Please install Nebula."
    echo "Visit: https://github.com/slackhq/nebula"
    exit 1
fi

# Generate CA if it doesn't exist
if [ ! -f "$CERT_DIR/ca.crt" ] || [ ! -f "$CERT_DIR/ca.key" ]; then
    echo "Generating CA certificate..."
    nebula-cert ca -name "NNOE CA" -duration "8760h" -out-crt "$CERT_DIR/ca.crt" -out-key "$CERT_DIR/ca.key"
    echo "CA certificate generated."
fi

# Generate node certificate
if [ ! -f "$CERT_DIR/${NODE_NAME}.crt" ] || [ ! -f "$CERT_DIR/${NODE_NAME}.key" ]; then
    echo "Generating certificate for node: $NODE_NAME"
    nebula-cert sign -name "$NODE_NAME" -ip "192.168.100.1/24" -duration "8760h" \
        -ca-crt "$CERT_DIR/ca.crt" -ca-key "$CERT_DIR/ca.key" \
        -out-crt "$CERT_DIR/${NODE_NAME}.crt" -out-key "$CERT_DIR/${NODE_NAME}.key"
    echo "Node certificate generated."
else
    echo "Certificate for $NODE_NAME already exists. Skipping."
fi

echo "Certificate generation complete!"
echo "Certificates are in: $CERT_DIR"

