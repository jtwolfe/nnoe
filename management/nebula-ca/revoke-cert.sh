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

# Try to find certificate file
CERT_FILE=""
if [ -f "/etc/nebula/certs/${NODE_NAME}.crt" ]; then
    CERT_FILE="/etc/nebula/certs/${NODE_NAME}.crt"
elif [ -f "./${NODE_NAME}.crt" ]; then
    CERT_FILE="./${NODE_NAME}.crt"
fi

# Generate or update CRL
echo "Updating Certificate Revocation List (CRL)..."
if [ ! -f "$CRL_FILE" ]; then
    echo "Creating new CRL..."
    nebula-cert sign-crl \
        -ca-crt "$CA_CRT" \
        -ca-key "$CA_KEY" \
        -out-crl "$CRL_FILE"
fi

# If certificate file is available, properly revoke it
if [ -n "$CERT_FILE" ] && [ -f "$CERT_FILE" ]; then
    echo "Revoking certificate using certificate file: $CERT_FILE"
    
    # Check if nebula-cert supports revocation
    if nebula-cert revoke --help &> /dev/null; then
        nebula-cert revoke \
            -ca-crt "$CA_CRT" \
            -ca-key "$CA_KEY" \
            -cert "$CERT_FILE" \
            -crl "$CRL_FILE"
        
        if [ $? -eq 0 ]; then
            echo "Certificate revoked and CRL updated"
        else
            echo "Warning: Certificate revocation command failed, but continuing..."
        fi
    else
        echo "Warning: nebula-cert revoke not available in this version"
        echo "Manually updating CRL..."
    fi
    
    # Update CRL with revoked certificate
    nebula-cert sign-crl \
        -ca-crt "$CA_CRT" \
        -ca-key "$CA_KEY" \
        -out-crl "$CRL_FILE"
else
    echo "Warning: Certificate file not found. Cannot properly revoke certificate."
    echo "Certificate will be marked as revoked in etcd, but CRL update may be incomplete."
fi

# Distribute CRL to etcd if endpoint provided
if [ -n "$ETCD_ENDPOINT" ] && [ -f "$CRL_FILE" ]; then
    echo "Distributing CRL to etcd..."
    CRL_DATA=$(cat "$CRL_FILE")
    etcdctl --endpoints="$ETCD_ENDPOINT" put \
        "$ETCD_PREFIX/nebula/crl" "$CRL_DATA"
    echo "CRL stored in etcd: $ETCD_PREFIX/nebula/crl"
fi

# Mark as revoked in etcd
echo "Marking certificate as revoked in etcd..."
if [ -n "$ETCD_ENDPOINT" ]; then
    REVOCATION_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date +"%Y-%m-%dT%H:%M:%SZ")
    REVOCATION_INFO="{\"revoked\": true, \"revoked_at\": \"$REVOCATION_TIME\", \"node\": \"$NODE_NAME\"}"
    
    etcdctl --endpoints="$ETCD_ENDPOINT" put \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/revoked" "$REVOCATION_INFO"
    
    # Remove from active certs
    etcdctl --endpoints="$ETCD_ENDPOINT" del \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/cert"
    etcdctl --endpoints="$ETCD_ENDPOINT" del \
        "$ETCD_PREFIX/nebula/certs/${NODE_NAME}/key"
    
    echo "Certificate revoked and removed from etcd"
else
    echo "Warning: No etcd endpoint provided. Certificate not marked in etcd."
fi

echo ""
echo "Certificate revocation completed!"
echo "CRL file: $CRL_FILE"
echo ""
echo "Next steps:"
echo "1. Distribute updated CRL to all Nebula nodes"
echo "2. Update Nebula configuration to use CRL"
echo "3. Restart Nebula on all nodes"

