#!/bin/bash
set -euo pipefail

# etcd Backup Validation Script
# Tests backup integrity and restore process

BACKUP_FILE="${1:-}"
TEST_DATA_DIR="${2:-/tmp/etcd-restore-test}"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file> [test-data-dir]"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Validating backup: $BACKUP_FILE"
echo "Test restore directory: $TEST_DATA_DIR"
echo ""

# Validate prerequisites
if ! command -v etcdctl &> /dev/null; then
    echo "Error: etcdctl not found"
    exit 1
fi

if ! command -v etcdutl &> /dev/null; then
    echo "Error: etcdutl not found"
    exit 1
fi

# Check snapshot status
echo "1. Checking snapshot status..."
SNAPSHOT_STATUS=$(etcdctl snapshot status "$BACKUP_FILE" 2>&1)
if [ $? -eq 0 ]; then
    echo "✓ Snapshot status valid"
    echo "$SNAPSHOT_STATUS"
else
    # Try decompressed if it's a .gz file
    if [[ "$BACKUP_FILE" == *.gz ]]; then
        echo "Decompressing for validation..."
        TEMP_FILE="/tmp/etcd-backup-validate-$$.db"
        if gunzip -c "$BACKUP_FILE" > "$TEMP_FILE"; then
            SNAPSHOT_STATUS=$(etcdctl snapshot status "$TEMP_FILE" 2>&1)
            if [ $? -eq 0 ]; then
                echo "✓ Snapshot status valid (after decompression)"
                echo "$SNAPSHOT_STATUS"
                BACKUP_FILE="$TEMP_FILE"
            else
                echo "✗ Snapshot validation failed"
                rm -f "$TEMP_FILE"
                exit 1
            fi
        else
            echo "✗ Failed to decompress backup"
            exit 1
        fi
    else
        echo "✗ Snapshot validation failed"
        exit 1
    fi
fi

echo ""

# Test restore
echo "2. Testing restore to temporary directory..."
rm -rf "$TEST_DATA_DIR"
mkdir -p "$TEST_DATA_DIR"

if etcdutl snapshot restore "$BACKUP_FILE" \
    --data-dir="$TEST_DATA_DIR" \
    --initial-cluster-token="validation-test" 2>&1; then
    echo "✓ Restore test successful"
else
    echo "✗ Restore test failed"
    rm -rf "$TEST_DATA_DIR"
    [ -n "${TEMP_FILE:-}" ] && rm -f "$TEMP_FILE"
    exit 1
fi

echo ""

# Verify restored data directory structure
echo "3. Verifying restored data..."
if [ -d "$TEST_DATA_DIR/member" ]; then
    echo "✓ Member directory exists"
    if [ -f "$TEST_DATA_DIR/member/snap/db" ] || [ -d "$TEST_DATA_DIR/member/wal" ]; then
        echo "✓ Database/WAL files present"
    else
        echo "✗ Database/WAL files missing"
        rm -rf "$TEST_DATA_DIR"
        [ -n "${TEMP_FILE:-}" ] && rm -f "$TEMP_FILE"
        exit 1
    fi
else
    echo "✗ Member directory missing"
    rm -rf "$TEST_DATA_DIR"
    [ -n "${TEMP_FILE:-}" ] && rm -f "$TEMP_FILE"
    exit 1
fi

echo ""

# Cleanup
echo "4. Cleaning up test restore..."
rm -rf "$TEST_DATA_DIR"
[ -n "${TEMP_FILE:-}" ] && rm -f "$TEMP_FILE"

echo ""
echo "✓ Backup validation completed successfully!"
echo "  Backup file is valid and can be restored."

