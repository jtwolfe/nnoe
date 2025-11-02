#!/bin/bash
set -euo pipefail

# etcd Backup Script
# Creates a backup of etcd cluster data with validation

ETCD_ENDPOINT="${1:-http://127.0.0.1:2379}"
BACKUP_DIR="${2:-/var/backups/etcd}"
BACKUP_NAME="etcd-backup-$(date +%Y%m%d-%H%M%S)"
RETENTION_DAYS="${3:-7}"

echo "Creating etcd backup: $BACKUP_NAME"
echo "Endpoint: $ETCD_ENDPOINT"
echo "Backup directory: $BACKUP_DIR"
echo "Retention: $RETENTION_DAYS days"

# Validate etcdctl
if ! command -v etcdctl &> /dev/null; then
    echo "Error: etcdctl not found. Please install etcd."
    exit 1
fi

# Validate endpoint connectivity
echo "Validating etcd endpoint connectivity..."
if ! etcdctl --endpoints="$ETCD_ENDPOINT" endpoint health &>/dev/null; then
    echo "Error: Cannot connect to etcd endpoint: $ETCD_ENDPOINT"
    exit 1
fi
echo "Endpoint is healthy"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Check disk space (require at least 100MB free)
AVAILABLE_SPACE=$(df "$BACKUP_DIR" | awk 'NR==2 {print $4}')
if [ "$AVAILABLE_SPACE" -lt 104857600 ]; then  # 100MB in KB
    echo "Warning: Less than 100MB free space in backup directory"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create snapshot
echo "Creating snapshot..."
if etcdctl --endpoints="$ETCD_ENDPOINT" snapshot save "$BACKUP_DIR/$BACKUP_NAME.db"; then
    echo "Snapshot created: $BACKUP_DIR/$BACKUP_NAME.db"
    
    # Validate snapshot
    echo "Validating snapshot..."
    SNAPSHOT_STATUS=$(etcdctl snapshot status "$BACKUP_DIR/$BACKUP_NAME.db" -w table 2>&1)
    if [ $? -eq 0 ]; then
        echo "$SNAPSHOT_STATUS"
    else
        echo "Warning: Could not validate snapshot status"
    fi
    
    # Compress backup
    echo "Compressing backup..."
    if gzip "$BACKUP_DIR/$BACKUP_NAME.db"; then
        echo "Backup compressed: $BACKUP_DIR/$BACKUP_NAME.db.gz"
        
        # Get backup file size
        BACKUP_SIZE=$(du -h "$BACKUP_DIR/$BACKUP_NAME.db.gz" | cut -f1)
        echo "Backup size: $BACKUP_SIZE"
    else
        echo "Error: Failed to compress backup"
        exit 1
    fi
    
    # Keep only last N days of backups
    echo "Cleaning up old backups (keeping last $RETENTION_DAYS days)..."
    OLD_BACKUPS=$(find "$BACKUP_DIR" -name "etcd-backup-*.db.gz" -mtime +$RETENTION_DAYS | wc -l)
    if [ "$OLD_BACKUPS" -gt 0 ]; then
        echo "Removing $OLD_BACKUPS old backup(s)"
        find "$BACKUP_DIR" -name "etcd-backup-*.db.gz" -mtime +$RETENTION_DAYS -delete
    else
        echo "No old backups to remove"
    fi
    
    # List current backups
    echo ""
    echo "Current backups in $BACKUP_DIR:"
    ls -lh "$BACKUP_DIR"/etcd-backup-*.db.gz 2>/dev/null | tail -5 || echo "  (none)"
    
    echo ""
    echo "Backup completed successfully: $BACKUP_DIR/$BACKUP_NAME.db.gz"
else
    echo "Backup failed!"
    exit 1
fi

