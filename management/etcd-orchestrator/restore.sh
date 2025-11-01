#!/bin/bash
set -euo pipefail

# etcd Restore Script
# Restores etcd cluster from backup

BACKUP_FILE="${1:-}"
DATA_DIR="${2:-/var/lib/etcd}"
NEW_CLUSTER_TOKEN="${3:-nnoe-cluster-restored}"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file> [data-dir] [cluster-token]"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Restoring etcd from backup: $BACKUP_FILE"
echo "Data directory: $DATA_DIR"

# Decompress if needed
if [[ "$BACKUP_FILE" == *.gz ]]; then
    echo "Decompressing backup..."
    gunzip -c "$BACKUP_FILE" > "/tmp/etcd-backup.db"
    BACKUP_FILE="/tmp/etcd-backup.db"
fi

# Restore snapshot
mkdir -p "$DATA_DIR"
etcdutl snapshot restore "$BACKUP_FILE" \
    --data-dir="$DATA_DIR" \
    --initial-cluster-token="$NEW_CLUSTER_TOKEN"

if [ $? -eq 0 ]; then
    echo "Restore completed successfully"
    echo "Start etcd with: etcd --data-dir=$DATA_DIR"
else
    echo "Restore failed!"
    exit 1
fi

