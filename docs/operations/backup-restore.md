# Backup and Restore Guide

Procedures for backing up and restoring NNOE deployments.

## Backup Strategy

### What to Backup

1. **etcd Cluster Data**: Configuration and state
2. **Agent Configuration**: `/etc/nnoe/agent.toml`
3. **Service Configurations**: Knot, Kea, dnsdist configs
4. **Certificates**: Nebula certificates, TLS certs
5. **Zone Files**: DNS zone files (if not in etcd)

## etcd Backup

### Creating Backups

**Using etcdctl:**

```bash
# Create snapshot
etcdctl snapshot save /backup/etcd-backup.db \
  --endpoints=http://127.0.0.1:2379

# Compress backup
gzip /backup/etcd-backup.db
```

**Using Script:**

```bash
./management/etcd-orchestrator/backup.sh \
  http://127.0.0.1:2379 \
  /backup/etcd
```

### Automated Backups

**Cron Job:**

```bash
# Add to crontab
0 2 * * * /path/to/backup.sh http://127.0.0.1:2379 /backup/etcd
```

**systemd Timer:**

```ini
[Unit]
Description=etcd Backup

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

### Backup Retention

- Keep daily backups for 7 days
- Keep weekly backups for 4 weeks
- Keep monthly backups for 12 months

## etcd Restore

### From Snapshot

```bash
# Stop etcd
systemctl stop etcd

# Restore snapshot
etcdutl snapshot restore /backup/etcd-backup.db \
  --data-dir=/var/lib/etcd \
  --initial-cluster-token=restored-cluster

# Start etcd
systemctl start etcd
```

**Using Script:**

```bash
./management/etcd-orchestrator/restore.sh \
  /backup/etcd-backup.db.gz \
  /var/lib/etcd \
  restored-cluster-token
```

### Cluster Restore

1. **Restore to First Node:**
   ```bash
   etcdutl snapshot restore backup.db \
     --initial-cluster="etcd-1=http://192.168.1.10:2380" \
     --initial-cluster-token=restored-token
   ```

2. **Add Additional Nodes:**
   ```bash
   # Add nodes to cluster after first node restored
   ```

## Configuration Backup

### Agent Configuration

```bash
# Backup config
cp /etc/nnoe/agent.toml /backup/agent-$(date +%Y%m%d).toml

# Restore
cp /backup/agent-20250101.toml /etc/nnoe/agent.toml
systemctl restart nnoe-agent
```

### Service Configurations

```bash
# Backup Knot config
cp /etc/knot/knot.conf /backup/knot-$(date +%Y%m%d).conf

# Backup Kea config
cp /etc/kea/kea-dhcp4.conf /backup/kea-$(date +%Y%m%d).conf

# Restore
cp /backup/knot-20250101.conf /etc/knot/knot.conf
knotc reload
```

## Certificate Backup

### Nebula Certificates

```bash
# Backup CA and certificates
tar czf /backup/nebula-certs-$(date +%Y%m%d).tar.gz \
  /etc/nebula/ca/ \
  /etc/nebula/certs/

# Restore
tar xzf /backup/nebula-certs-20250101.tar.gz -C /
```

### TLS Certificates

```bash
# Backup TLS certs
cp -r /etc/nnoe/certs /backup/certs-$(date +%Y%m%d)
```

## Complete System Backup

### Backup Script

```bash
#!/bin/bash
BACKUP_DIR="/backup/nnoe-$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# etcd backup
etcdctl snapshot save "$BACKUP_DIR/etcd.db" \
  --endpoints=http://127.0.0.1:2379

# Config backup
cp -r /etc/nnoe "$BACKUP_DIR/etc-nnoe"
cp -r /etc/knot "$BACKUP_DIR/etc-knot"
cp -r /etc/kea "$BACKUP_DIR/etc-kea"

# Certificate backup
cp -r /etc/nebula "$BACKUP_DIR/etc-nebula"

# Compress
tar czf "$BACKUP_DIR.tar.gz" "$BACKUP_DIR"
rm -rf "$BACKUP_DIR"

# Upload to remote storage (optional)
# scp "$BACKUP_DIR.tar.gz" backup-server:/backups/
```

## Disaster Recovery

### Recovery Procedure

1. **Assess Damage**: Identify what needs restoration
2. **Restore etcd**: Restore cluster from backup
3. **Restore Configs**: Restore configuration files
4. **Restore Certificates**: Restore certificates
5. **Start Services**: Start and verify services
6. **Validate**: Verify system functionality

### Recovery Checklist

- [ ] etcd cluster restored and healthy
- [ ] Agent configuration restored
- [ ] Service configs restored
- [ ] Certificates valid
- [ ] Services running
- [ ] DNS resolving correctly
- [ ] DHCP issuing leases
- [ ] Monitoring functional

## Backup Verification

### Test Restores

Regularly test backup restoration:

```bash
# Create test environment
docker run -d --name etcd-test quay.io/coreos/etcd:v3.5.9

# Restore to test environment
etcdutl snapshot restore backup.db \
  --data-dir=/tmp/etcd-test

# Verify data
etcdctl --endpoints=http://localhost:2379 get --prefix /nnoe
```

## Best Practices

1. **Regular Backups**: Daily automated backups
2. **Off-site Storage**: Store backups remotely
3. **Test Restores**: Verify backups regularly
4. **Document Procedures**: Document all backup/restore steps
5. **Monitor Backup Success**: Alert on backup failures
6. **Retention Policy**: Clear retention policy
7. **Encryption**: Encrypt sensitive backups

## Backup Tools

### etcd

- `etcdctl snapshot save`: Create snapshots
- `etcdutl snapshot restore`: Restore snapshots

### Files

- `tar`: Archive files
- `rsync`: Sync files
- `rclone`: Cloud storage sync

### Automation

- Cron jobs
- systemd timers
- CI/CD pipelines

