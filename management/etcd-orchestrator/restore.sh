#!/bin/bash
set -euo pipefail

# etcd Restore Script
# Restores etcd cluster from backup with validation

BACKUP_FILE="${1:-}"
DATA_DIR="${2:-/var/lib/etcd}"
NEW_CLUSTER_TOKEN="${3:-nnoe-cluster-restored}"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file> [data-dir] [cluster-token]"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Restoring etcd from backup: $BACKUP_FILE"
echo "Data directory: $DATA_DIR"
echo "Cluster token: $NEW_CLUSTER_TOKEN"
echo ""

# Validate prerequisites
if ! command -v etcdutl &> /dev/null; then
    echo "Error: etcdutl not found. Please install etcd."
    exit 1
fi

# Check if data directory exists and has data
if [ -d "$DATA_DIR" ] && [ "$(ls -A "$DATA_DIR" 2>/dev/null)" ]; then
    echo "Warning: Data directory $DATA_DIR already contains data"
    echo "Restoring will overwrite existing data!"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Validate backup file
TEMP_BACKUP="$BACKUP_FILE"
if [[ "$BACKUP_FILE" == *.gz ]]; then
    echo "Decompressing backup..."
    TEMP_BACKUP="/tmp/etcd-backup-$$.db"
    if ! gunzip -c "$BACKUP_FILE" > "$TEMP_BACKUP"; then
        echo "Error: Failed to decompress backup file"
        exit 1
    fi
    echo "Decompression successful"
fi

# Validate snapshot
echo "Validating snapshot..."
SNAPSHOT_STATUS=$(etcdctl snapshot status "$TEMP_BACKUP" 2>&1)
if [ $? -eq 0 ]; then
    echo "Snapshot status:"
    echo "$SNAPSHOT_STATUS" | head -5
    echo ""
    
    # Extract key information
    REVISION=$(echo "$SNAPSHOT_STATUS" | grep "Revision:" | awk '{print $2}' || echo "unknown")
    TOTAL_KEYS=$(echo "$SNAPSHOT_STATUS" | grep "Total keys:" | awk '{print $3}' || echo "unknown")
    TOTAL_SIZE=$(echo "$SNAPSHOT_STATUS" | grep "Total size:" | awk '{print $3}' || echo "unknown")
    
    echo "Snapshot information:"
    echo "  Revision: $REVISION"
    echo "  Total keys: $TOTAL_KEYS"
    echo "  Total size: $TOTAL_SIZE"
    echo ""
else
    echo "Warning: Could not fully validate snapshot, but continuing..."
fi

# Restore snapshot
echo "Restoring snapshot to $DATA_DIR..."
mkdir -p "$DATA_DIR"

if etcdutl snapshot restore "$TEMP_BACKUP" \
    --data-dir="$DATA_DIR" \
    --initial-cluster-token="$NEW_CLUSTER_TOKEN"; then
    
    echo ""
    echo "Restore completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Verify restored data directory: ls -lh $DATA_DIR"
    echo "2. Update etcd configuration to use: --data-dir=$DATA_DIR"
    echo "3. Start etcd: etcd --data-dir=$DATA_DIR"
    echo "4. Verify health: etcdctl endpoint health"
    echo ""
    echo "Note: This restore creates a new cluster. If you need to restore to an existing cluster,"
    echo "      you must restore on all nodes and reconfigure the cluster membership."
    
    # Clean up temp file if created
    if [ "$TEMP_BACKUP" != "$BACKUP_FILE" ]; then
        rm -f "$TEMP_BACKUP"
    fi
else
    echo "Restore failed!"
    
    # Clean up temp file if created
    if [ "$TEMP_BACKUP" != "$BACKUP_FILE" ]; then
        rm -f "$TEMP_BACKUP"
    fi
    
    exit 1
fi

