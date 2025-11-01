#!/bin/bash
set -euo pipefail

# Nebula Certificate Revocation Script
# Revokes a certificate and updates etcd

NODE_NAME="${1:-}"
CA_CRT="${2:-/etc/nebula/ca/ca.crt}"
CA_KEY="${3:-/etc/nebula/ca/ca.key}"
CRL_FILE="${4:-/etc/nebula/ca/crl.pem}"
ETCD_ENDPOINT="${5:-http://127.0.0.1:2379}"
ETCD_PREFIX="${6:-/nnoe}"

if [ -z "$NODE_NAME" ]; then
    echo "Usage: $0 <node-name> [ca-crt] [ca-key] [crl-file] [etcd-endpoint] [etcd-prefix]"
    exit 1
fi

echo "Revoking certificate for node: $NODE_NAME"

# Check if nebula-cert supports revocation
if ! nebula-cert revoke --help &> /dev/null; then
    echo "Warning: nebula-cert revoke not available in this version"
    echo "Manually removing certificate from etcd..."
    
    etcdctl --endpoints="$ETCD_ENDPOINT" del \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/cert"
    etcdctl --endpoints="$ETCD_ENDPOINT" del \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/key"
    
    # Mark as revoked in etcd
    etcdctl --endpoints="$ETCD_ENDPOINT" put \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/revoked" "true"
    
    echo "Certificate marked as revoked in etcd"
    exit 0
fi

# Generate CRL if it doesn't exist
if [ ! -f "$CRL_FILE" ]; then
    nebula-cert sign-crl \
        -ca-crt "$CA_CRT" \
        -ca-key "$CA_KEY" \
        -out-crl "$CRL_FILE"
fi

# Revoke certificate (this would require the original cert file)
echo "Certificate revocation requires the original certificate file"
echo "For now, marking as revoked in etcd..."

# Mark as revoked in etcd
etcdctl --endpoints="$ETCD_ENDPOINT" put \
    "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/revoked" "true"

# Remove from active certs
etcdctl --endpoints="$ETCD_ENDPOINT" del \
    "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/cert"
etcdctl --endpoints="$ETCD_ENDPOINT" del \
    "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/key"

echo "Certificate revoked and removed from etcd"

