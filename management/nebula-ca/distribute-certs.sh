#!/bin/bash
set -euo pipefail

# Nebula Certificate Distribution Script
# Distributes certificates to etcd for agent retrieval

NODE_NAME="${1:-}"
CERT_FILE="${2:-}"
KEY_FILE="${3:-}"
ETCD_ENDPOINT="${4:-http://127.0.0.1:2379}"
ETCD_PREFIX="${5:-/nnoe}"

if [ -z "$NODE_NAME" ] || [ -z "$CERT_FILE" ] || [ -z "$KEY_FILE" ]; then
    echo "Usage: $0 <node-name> <cert-file> <key-file> [etcd-endpoint] [etcd-prefix]"
    exit 1
fi

if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
    echo "Certificate or key file not found!"
    exit 1
fi

echo "Distributing certificate for node: $NODE_NAME"
echo "etcd endpoint: $ETCD_ENDPOINT"
echo "etcd prefix: $ETCD_PREFIX"

# Read certificate and key
CERT_DATA=$(cat "$CERT_FILE")
KEY_DATA=$(cat "$KEY_FILE")

# Store certificate in etcd
etcdctl --endpoints="$ETCD_ENDPOINT" put \
    "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/cert" "$CERT_DATA"

if [ $? -eq 0 ]; then
    echo "Certificate stored in etcd: $ETCD_PREFIX/nebula/certs/${NODE_NAME}/cert"
else
    echo "Failed to store certificate in etcd"
    exit 1
fi

# Store key in etcd (in production, use TLS/encryption)
etcdctl --endpoints="$ETCD_ENDPOINT" put \
    "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/key" "$KEY_DATA"

if [ $? -eq 0 ]; then
    echo "Key stored in etcd: $ETCD_PREFIX/nebula/certs/${NODE_NAME}/key"
    echo ""
    echo "WARNING: Keys are stored in plain text. In production, use TLS encryption!"
else
    echo "Failed to store key in etcd"
    exit 1
fi

