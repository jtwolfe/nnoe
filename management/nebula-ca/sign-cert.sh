#!/bin/bash
set -euo pipefail

# Nebula Certificate Signing Script
# Signs a certificate for a node

NODE_NAME="${1:-}"
NODE_IP="${2:-}"
CA_CRT="${3:-/etc/nebula/ca/ca.crt}"
CA_KEY="${4:-/etc/nebula/ca/ca.key}"
OUTPUT_DIR="${5:-/etc/nebula/certs}"
DURATION="${6:-8760h}"  # 1 year
GROUPS="${7:-}"

if [ -z "$NODE_NAME" ] || [ -z "$NODE_IP" ]; then
    echo "Usage: $0 <node-name> <node-ip> [ca-crt] [ca-key] [output-dir] [duration] [groups]"
    echo "Example: $0 node-1 192.168.100.1 /etc/nebula/ca/ca.crt /etc/nebula/ca/ca.key"
    exit 1
fi

echo "Signing certificate for node: $NODE_NAME"
echo "IP: $NODE_IP"
echo "Output directory: $OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR"

# Build nebula-cert command
CERT_CMD="nebula-cert sign -name $NODE_NAME -ip $NODE_IP/24"
CERT_CMD="$CERT_CMD -duration $DURATION"
CERT_CMD="$CERT_CMD -ca-crt $CA_CRT -ca-key $CA_KEY"
CERT_CMD="$CERT_CMD -out-crt $OUTPUT_DIR/${NODE_NAME}.crt"
CERT_CMD="$CERT_CMD -out-key $OUTPUT_DIR/${NODE_NAME}.key"

if [ -n "$GROUPS" ]; then
    CERT_CMD="$CERT_CMD -groups $GROUPS"
fi

# Execute
eval "$CERT_CMD"

if [ $? -eq 0 ]; then
    echo "Certificate signed successfully:"
    echo "  Certificate: $OUTPUT_DIR/${NODE_NAME}.crt"
    echo "  Key: $OUTPUT_DIR/${NODE_NAME}.key"
else
    echo "Certificate signing failed!"
    exit 1
fi

