#!/bin/bash
set -euo pipefail

# Nebula Certificate Rotation Script
# Automatically rotates certificates that are nearing expiration

NODE_NAME="${1:-}"
CA_CRT="${2:-/etc/nebula/ca/ca.crt}"
CA_KEY="${3:-/etc/nebula/ca/ca.key}"
CERT_DIR="${4:-/etc/nebula/certs}"
DAYS_BEFORE_EXPIRY="${5:-30}"  # Rotate if expiring within 30 days
ETCD_ENDPOINT="${6:-}"
ETCD_PREFIX="${7:-/nnoe}"

if [ -z "$NODE_NAME" ]; then
    echo "Usage: $0 <node-name> [ca-crt] [ca-key] [cert-dir] [days-before-expiry] [etcd-endpoint] [etcd-prefix]"
    echo ""
    echo "Rotates Nebula certificate if it expires within the specified number of days."
    echo "If etcd-endpoint is provided, automatically distributes new certificate to etcd."
    exit 1
fi

CERT_FILE="$CERT_DIR/${NODE_NAME}.crt"
KEY_FILE="$CERT_DIR/${NODE_NAME}.key"

# Check if certificate exists
if [ ! -f "$CERT_FILE" ]; then
    echo "Error: Certificate file not found: $CERT_FILE"
    exit 1
fi

# Check certificate expiration
CERT_INFO=$(nebula-cert print -path "$CERT_FILE" 2>&1 || true)

if echo "$CERT_INFO" | grep -q "notAfter"; then
    EXPIRY_DATE=$(echo "$CERT_INFO" | grep "notAfter" | awk '{print $2" "$3}')
    
    # Convert to epoch
    if command -v date &> /dev/null; then
        EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s 2>/dev/null || date -j -f "%b %d %H:%M:%S %Y %Z" "$EXPIRY_DATE" +%s 2>/dev/null || echo "0")
        CURRENT_EPOCH=$(date +%s)
        DAYS_UNTIL_EXPIRY=$(( (EXPIRY_EPOCH - CURRENT_EPOCH) / 86400 ))
        
        echo "Certificate expires in $DAYS_UNTIL_EXPIRY days ($EXPIRY_DATE)"
        
        if [ "$DAYS_UNTIL_EXPIRY" -gt "$DAYS_BEFORE_EXPIRY" ]; then
            echo "Certificate is still valid (expires in $DAYS_UNTIL_EXPIRY days, threshold: $DAYS_BEFORE_EXPIRY days)"
            echo "No rotation needed."
            exit 0
        fi
        
        echo "Certificate expires within $DAYS_BEFORE_EXPIRY days. Rotating..."
    else
        echo "Warning: Cannot parse expiration date. Proceeding with rotation..."
    fi
else
    echo "Warning: Could not determine certificate expiration. Proceeding with rotation..."
fi

# Extract node IP from existing certificate (try to preserve it)
if echo "$CERT_INFO" | grep -q "IPs"; then
    NODE_IP=$(echo "$CERT_INFO" | grep "IPs" | awk '{print $2}' | cut -d'/' -f1)
    echo "Detected node IP from certificate: $NODE_IP"
else
    echo "Error: Could not determine node IP from certificate"
    echo "Please specify node IP manually:"
    read -p "Node IP: " NODE_IP
fi

# Extract groups from existing certificate
GROUPS=""
if echo "$CERT_INFO" | grep -q "Groups"; then
    GROUPS=$(echo "$CERT_INFO" | grep "Groups" | awk '{for(i=2;i<=NF;i++) printf "%s ", $i; print ""}' | xargs)
    echo "Preserving groups: $GROUPS"
fi

# Backup old certificate
BACKUP_DIR="$CERT_DIR/backup"
mkdir -p "$BACKUP_DIR"
BACKUP_NAME="backup-$(date +%Y%m%d-%H%M%S)"
cp "$CERT_FILE" "$BACKUP_DIR/${NODE_NAME}.crt.$BACKUP_NAME"
cp "$KEY_FILE" "$BACKUP_DIR/${NODE_NAME}.key.$BACKUP_NAME"
echo "Backed up old certificate to: $BACKUP_DIR/${NODE_NAME}.crt.$BACKUP_NAME"

# Sign new certificate using sign-cert.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SIGN_SCRIPT="$SCRIPT_DIR/sign-cert.sh"

if [ -f "$SIGN_SCRIPT" ]; then
    echo "Signing new certificate..."
    "$SIGN_SCRIPT" "$NODE_NAME" "$NODE_IP" "$CA_CRT" "$CA_KEY" "$CERT_DIR" "" "$GROUPS"
else
    echo "Error: sign-cert.sh not found"
    exit 1
fi

if [ $? -eq 0 ]; then
    echo "Certificate rotation completed successfully!"
    
    # Distribute to etcd if endpoint provided
    if [ -n "$ETCD_ENDPOINT" ]; then
        DISTRIBUTE_SCRIPT="$SCRIPT_DIR/distribute-certs.sh"
        if [ -f "$DISTRIBUTE_SCRIPT" ]; then
            echo "Distributing new certificate to etcd..."
            "$DISTRIBUTE_SCRIPT" "$NODE_NAME" "$CERT_FILE" "$KEY_FILE" "$ETCD_ENDPOINT" "$ETCD_PREFIX"
        else
            echo "Warning: distribute-certs.sh not found. Skipping etcd distribution."
        fi
    fi
    
    echo ""
    echo "Next steps:"
    echo "1. Verify new certificate: nebula-cert print -path $CERT_FILE"
    echo "2. Restart Nebula service on node: $NODE_NAME"
    echo "3. Verify Nebula connectivity"
    echo ""
    echo "Note: Old certificate is backed up in $BACKUP_DIR"
else
    echo "Certificate rotation failed!"
    exit 1
fi

