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

# Create data directory
mkdir -p "$DATA_DIR"

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
echo "Start etcd with: etcd --config-file /etc/etcd/etcd.conf"

