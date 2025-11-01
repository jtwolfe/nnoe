#!/bin/bash
set -euo pipefail

# etcd Backup Script
# Creates a backup of etcd cluster data

ETCD_ENDPOINT="${1:-http://127.0.0.1:2379}"
BACKUP_DIR="${2:-/var/backups/etcd}"
BACKUP_NAME="etcd-backup-$(date +%Y%m%d-%H%M%S)"

echo "Creating etcd backup: $BACKUP_NAME"

mkdir -p "$BACKUP_DIR"

# Create snapshot
etcdctl --endpoints="$ETCD_ENDPOINT" snapshot save "$BACKUP_DIR/$BACKUP_NAME.db"

if [ $? -eq 0 ]; then
    echo "Backup created: $BACKUP_DIR/$BACKUP_NAME.db"
    
    # Compress backup
    gzip "$BACKUP_DIR/$BACKUP_NAME.db"
    echo "Backup compressed: $BACKUP_DIR/$BACKUP_NAME.db.gz"
    
    # Keep only last 7 days of backups
    find "$BACKUP_DIR" -name "etcd-backup-*.db.gz" -mtime +7 -delete
else
    echo "Backup failed!"
    exit 1
fi

