#!/bin/bash
set -euo pipefail

# etcd Cluster Bootstrap Script
# Creates a new etcd cluster or adds a node to an existing cluster

NODE_NAME="${1:-etcd-1}"
NODE_IP="${2:-127.0.0.1}"
CLUSTER_TOKEN="${3:-nnoe-cluster-1}"
INITIAL_CLUSTER="${4:-}"
DATA_DIR="${5:-/var/lib/etcd}"

echo "Bootstrapping etcd node: $NODE_NAME"
echo "IP: $NODE_IP"
echo "Data directory: $DATA_DIR"

# Validate prerequisites
if ! command -v etcdctl &> /dev/null; then
    echo "Error: etcdctl not found. Please install etcd."
    exit 1
fi

if ! command -v etcd &> /dev/null; then
    echo "Error: etcd binary not found. Please install etcd."
    exit 1
fi

# Validate IP address format (basic check)
if ! [[ "$NODE_IP" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
    echo "Warning: $NODE_IP may not be a valid IPv4 address"
fi

# Check if data directory exists and is writable
if [ -d "$DATA_DIR" ] && [ ! -w "$DATA_DIR" ]; then
    echo "Error: Data directory $DATA_DIR is not writable"
    exit 1
fi

# Create data directory
mkdir -p "$DATA_DIR"

# Check if etcd is already running on this node
if pgrep -x etcd > /dev/null; then
    echo "Warning: etcd process is already running. Please stop it before bootstrapping."
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# If joining existing cluster, verify connectivity
if [ -n "$INITIAL_CLUSTER" ]; then
    # Extract first endpoint from INITIAL_CLUSTER for validation
    FIRST_ENDPOINT=$(echo "$INITIAL_CLUSTER" | cut -d',' -f1 | cut -d'=' -f2 | sed 's|2380|2379|')
    echo "Validating connectivity to existing cluster: $FIRST_ENDPOINT"
    
    if etcdctl --endpoints="$FIRST_ENDPOINT" endpoint health &>/dev/null; then
        echo "Successfully connected to existing cluster"
    else
        echo "Warning: Could not connect to existing cluster endpoint: $FIRST_ENDPOINT"
        echo "This may be expected if this is the first node. Continuing..."
    fi
fi

# Determine if this is the first node
if [ -z "$INITIAL_CLUSTER" ]; then
    echo "Initializing new etcd cluster"
    INITIAL_CLUSTER="$NODE_NAME=http://$NODE_IP:2380"
    INITIAL_CLUSTER_STATE="new"
else
    echo "Joining existing etcd cluster"
    INITIAL_CLUSTER="$INITIAL_CLUSTER,$NODE_NAME=http://$NODE_IP:2380"
    INITIAL_CLUSTER_STATE="existing"
fi

# Generate etcd configuration
cat > /etc/etcd/etcd.conf <<EOF
# NNOE etcd Configuration
name: $NODE_NAME
data-dir: $DATA_DIR

listen-client-urls: http://$NODE_IP:2379
advertise-client-urls: http://$NODE_IP:2379

listen-peer-urls: http://$NODE_IP:2380
initial-advertise-peer-urls: http://$NODE_IP:2380

initial-cluster: $INITIAL_CLUSTER
initial-cluster-token: $CLUSTER_TOKEN
initial-cluster-state: $INITIAL_CLUSTER_STATE

# Performance tuning
max-request-bytes: 10485760
quota-backend-bytes: 8589934592

# Security (TLS can be added)
client-transport-security:
  cert-file: ""
  key-file: ""
  client-cert-auth: false

peer-transport-security:
  cert-file: ""
  key-file: ""
  peer-client-cert-auth: false
EOF

echo "etcd configuration written to /etc/etcd/etcd.conf"
echo ""
echo "Next steps:"
echo "1. Review configuration: cat /etc/etcd/etcd.conf"
echo "2. Start etcd: etcd --config-file /etc/etcd/etcd.conf"
echo "3. Verify health: etcdctl --endpoints=http://$NODE_IP:2379 endpoint health"
echo ""
echo "To run as systemd service:"
echo "  sudo systemctl start etcd"
echo "  sudo systemctl enable etcd"

