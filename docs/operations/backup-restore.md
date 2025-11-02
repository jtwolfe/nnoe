# Backup and Restore Procedures

This document describes backup and restore procedures for NNOE components.

## Overview

NNOE stores critical configuration and state in:
- **etcd**: Configuration data, zones, DHCP scopes, policies, HA state
- **sled cache**: Local agent cache (optional backup for performance)
- **phpIPAM database**: UI/IPAM data (MySQL/MariaDB)

## Backup Strategies

### etcd Backup

etcd contains all NNOE configuration and state. Regular backups are critical.

#### Automated Backup (Recommended)

Use the etcd orchestrator backup script:

```bash
# From management node
cd management/etcd-orchestrator
./backup.sh /backup/nnoe/etcd
```

This creates timestamped snapshots:
- `/backup/nnoe/etcd/etcd-snapshot-YYYYMMDD-HHMMSS.db`

#### Manual Backup

```bash
# Single node backup
ETCDCTL_API=3 etcdctl snapshot save /backup/etcd-snapshot.db \
  --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/nnoe/certs/ca.crt \
  --cert=/etc/nnoe/certs/client.crt \
  --key=/etc/nnoe/certs/client.key

# Verify backup
ETCDCTL_API=3 etcdctl snapshot status /backup/etcd-snapshot.db
```

#### Backup Schedule

Recommended backup frequency:
- **Production**: Every 6 hours (retain 7 days)
- **Development**: Daily (retain 3 days)

Example cron job:
```cron
0 */6 * * * /opt/nnoe/scripts/etcd-backup.sh
```

### phpIPAM Database Backup

phpIPAM uses MySQL/MariaDB for its backend.

#### Automated Backup

```bash
#!/bin/bash
# backup-phpipam.sh

BACKUP_DIR="/backup/nnoe/phpipam"
DATE=$(date +%Y%m%d-%H%M%S)
DB_NAME="phpipam"
DB_USER="phpipam"

mkdir -p "$BACKUP_DIR"

# Dump database
mysqldump -u "$DB_USER" -p"$DB_PASSWORD" "$DB_NAME" \
  > "$BACKUP_DIR/phpipam-$DATE.sql"

# Compress
gzip "$BACKUP_DIR/phpipam-$DATE.sql"

# Retain last 7 days
find "$BACKUP_DIR" -name "*.sql.gz" -mtime +7 -delete
```

#### Manual Backup

```bash
mysqldump -u phpipam -p phpipam > phpipam-backup.sql
```

### Agent Cache Backup (Optional)

The sled cache is regenerated automatically, but can be backed up for faster recovery:

```bash
# From agent node
tar -czf /backup/nnoe/cache/node-$(hostname)-$(date +%Y%m%d).tar.gz \
  /var/nnoe/cache
```

## Restore Procedures

### etcd Restore

#### From Backup Snapshot

```bash
# 1. Stop etcd on all nodes
systemctl stop etcd

# 2. Restore snapshot on first node
ETCDCTL_API=3 etcdctl snapshot restore /backup/etcd-snapshot.db \
  --data-dir=/var/lib/etcd/restored \
  --name=etcd-1 \
  --initial-advertise-peer-urls=https://192.168.1.10:2380 \
  --initial-cluster=etcd-1=https://192.168.1.10:2380,etcd-2=https://192.168.1.11:2380,etcd-3=https://192.168.1.12:2380

# 3. Update etcd service to use restored data-dir
# Edit /etc/systemd/system/etcd.service
# Change --data-dir=/var/lib/etcd to --data-dir=/var/lib/etcd/restored

# 4. Start etcd
systemctl start etcd

# 5. Verify cluster health
ETCDCTL_API=3 etcdctl endpoint health --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/nnoe/certs/ca.crt \
  --cert=/etc/nnoe/certs/client.crt \
  --key=/etc/nnoe/certs/client.key

# 6. Rebuild cluster (add remaining nodes)
# On other nodes, use etcd member add/remove commands
```

#### Point-in-Time Recovery

etcd snapshots capture a specific revision. For point-in-time recovery:

1. **Identify target revision**: Check backup snapshot revision
   ```bash
   ETCDCTL_API=3 etcdctl snapshot status /backup/etcd-snapshot-20240101-120000.db
   # Output shows revision number (e.g., Revision: 12345)
   ```

2. **Restore from snapshot**: Use standard restore procedure above

3. **Recover incremental changes** (if available):
   - If using WAL (Write-Ahead Log) backups, replay from snapshot to target time
   - Otherwise, restore to snapshot time and accept data loss after that point

4. **Verify data consistency**: After restore, verify critical keys
   ```bash
   # Check specific keys exist and have expected values
   ETCDCTL_API=3 etcdctl get /nnoe/dns/zones/example.com \
     --endpoints=https://127.0.0.1:2379 \
     --cacert=/etc/nnoe/certs/ca.crt \
     --cert=/etc/nnoe/certs/client.crt \
     --key=/etc/nnoe/certs/client.key
   ```

5. **Rebuild agent caches**: Agents will automatically rebuild sled cache from etcd
   ```bash
   # Restart agents to force cache rebuild
   systemctl restart nnoe-agent
   ```

### sled Cache Backup/Restore

The sled cache is optional and can be backed up for faster agent recovery:

#### Backup

```bash
# From agent node
systemctl stop nnoe-agent
tar -czf /backup/nnoe/cache/node-$(hostname)-$(date +%Y%m%d-%H%M%S).tar.gz \
  /var/nnoe/cache
systemctl start nnoe-agent
```

#### Restore

```bash
# From agent node
systemctl stop nnoe-agent

# Restore cache
tar -xzf /backup/nnoe/cache/node-hostname-20240101-120000.tar.gz -C /

# Verify cache integrity
# (sled automatically validates on startup)

systemctl start nnoe-agent
```

**Note**: Cache restore is optional. Agents will automatically rebuild cache from etcd if cache is missing or corrupted.

### Nebula Certificate Backup

Nebula certificates and keys are critical for overlay network connectivity:

#### Backup

```bash
# Backup Nebula certificates and keys
tar -czf /backup/nnoe/nebula/nebula-certs-$(date +%Y%m%d).tar.gz \
  /etc/nebula/ca.crt \
  /etc/nebula/host.crt \
  /etc/nebula/host.key \
  /etc/nebula/config.yml

# Store in secure location (encrypted)
```

#### Restore

```bash
# Restore Nebula certificates
systemctl stop nebula

# Restore files
tar -xzf /backup/nnoe/nebula/nebula-certs-20240101.tar.gz -C /

# Verify permissions
chmod 600 /etc/nebula/host.key
chmod 644 /etc/nebula/host.crt /etc/nebula/ca.crt

# Restart Nebula
systemctl start nebula
```

**Important**: Nebula certificates must match the lighthouse configuration. Ensure certificates are restored from the same cluster/environment.

#### Using Restore Script

```bash
cd management/etcd-orchestrator
./restore.sh /backup/nnoe/etcd/etcd-snapshot-20240101-120000.db
```

### phpIPAM Database Restore

```bash
# 1. Stop phpIPAM
docker-compose stop phpipam

# 2. Restore database
gunzip < /backup/nnoe/phpipam/phpipam-20240101-120000.sql.gz | \
  mysql -u phpipam -p phpipam

# 3. Restart phpIPAM
docker-compose start phpipam
```

### Agent Cache Restore (Optional)

```bash
# From agent node
systemctl stop nnoe-agent
tar -xzf /backup/nnoe/cache/node-hostname-20240101.tar.gz -C /
systemctl start nnoe-agent
```

## Disaster Recovery

### Full Cluster Recovery

In case of complete cluster failure:

1. **Restore etcd from latest backup**
   - Restore snapshot on first node
   - Rebuild cluster from scratch

2. **Restore phpIPAM database**
   - Restore MySQL dump
   - Verify UI connectivity

3. **Reconfigure agents**
   - Agents will automatically reconnect to etcd
   - Cache will regenerate automatically

4. **Verify services**
   - Check DNS resolution
   - Check DHCP leases
   - Verify monitoring/metrics

### Partial Recovery (Single Node Failure)

1. **Management node failure**
   - If etcd leader fails, follower auto-promotes
   - Restore phpIPAM if needed
   - Verify agent connectivity

2. **Agent node failure**
   - Agent is stateless (config in etcd)
   - Restart agent on new node
   - Agent will pull config from etcd

3. **Database-only node failure**
   - etcd follower lost, but cluster survives with quorum
   - Add new node or restore from backup

## Backup Verification

### Verify etcd Backup

```bash
ETCDCTL_API=3 etcdctl snapshot status /backup/etcd-snapshot.db

# Expected output:
# Hash: abc123...
# Revision: 12345
# Total Keys: 1000
# Total Size: 5.2 MB
```

### Verify phpIPAM Backup

```bash
# Check SQL file integrity
gunzip -t /backup/nnoe/phpipam/phpipam-*.sql.gz

# Check database size
ls -lh /backup/nnoe/phpipam/phpipam-*.sql.gz
```

## Best Practices

1. **Automate backups**: Use cron/systemd timers
2. **Test restores**: Regularly test restore procedures
3. **Off-site storage**: Store backups in separate location
4. **Encryption**: Encrypt backups at rest
5. **Monitoring**: Monitor backup success/failure
6. **Documentation**: Keep restore procedures documented
7. **Retention**: Follow retention policies (7 days for prod)

## Backup Script Example

Complete backup script:

```bash
#!/bin/bash
# Full NNOE backup script

set -euo pipefail

BACKUP_ROOT="/backup/nnoe"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="$BACKUP_ROOT/$DATE"

mkdir -p "$BACKUP_DIR"

# Backup etcd
echo "Backing up etcd..."
ETCDCTL_API=3 etcdctl snapshot save "$BACKUP_DIR/etcd-snapshot.db" \
  --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/nnoe/certs/ca.crt \
  --cert=/etc/nnoe/certs/client.crt \
  --key=/etc/nnoe/certs/client.key

# Backup phpIPAM
echo "Backing up phpIPAM..."
mysqldump -u phpipam -p"$MYSQL_PASSWORD" phpipam | \
  gzip > "$BACKUP_DIR/phpipam.sql.gz"

# Backup agent configs (if custom)
if [ -d "/etc/nnoe/agent.d" ]; then
  tar -czf "$BACKUP_DIR/agent-configs.tar.gz" /etc/nnoe/agent.d
fi

# Compress entire backup
tar -czf "$BACKUP_ROOT/nnoe-backup-$DATE.tar.gz" -C "$BACKUP_ROOT" "$DATE"
rm -rf "$BACKUP_DIR"

# Upload to S3/object storage (optional)
# aws s3 cp "$BACKUP_ROOT/nnoe-backup-$DATE.tar.gz" \
#   s3://nnoe-backups/

echo "Backup completed: nnoe-backup-$DATE.tar.gz"

# Cleanup old backups (retain 7 days)
find "$BACKUP_ROOT" -name "nnoe-backup-*.tar.gz" -mtime +7 -delete
```

## Troubleshooting

### Backup Fails

- Check etcd connectivity
- Verify certificate permissions
- Check disk space
- Review etcd logs

### Restore Fails

- Verify backup file integrity
- Check etcd data directory permissions
- Ensure cluster configuration matches backup
- Review etcd member list

### Partial Data Loss

- Restore from most recent backup
- Check etcd revision numbers
- Verify agent cache consistency
- Re-sync phpIPAM if needed
