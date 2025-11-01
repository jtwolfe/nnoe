# etcd Orchestrator

Scripts for managing etcd cluster deployment, membership, and backups.

## Scripts

### bootstrap.sh

Bootstrap a new etcd node or add to existing cluster.

```bash
./bootstrap.sh <node-name> <node-ip> [cluster-token] [initial-cluster] [data-dir]
```

Example:
```bash
./bootstrap.sh etcd-1 192.168.1.10 nnoe-cluster-1 "" /var/lib/etcd
```

### member-manage.sh

Manage etcd cluster members (list, add, remove).

```bash
./member-manage.sh <endpoint> {list|add|remove} [member-name] [member-ip]
```

Examples:
```bash
# List members
./member-manage.sh http://127.0.0.1:2379 list

# Add member
./member-manage.sh http://127.0.0.1:2379 add etcd-2 192.168.1.11

# Remove member
./member-manage.sh http://127.0.0.1:2379 remove <member-id>
```

### backup.sh

Create backup of etcd cluster.

```bash
./backup.sh [endpoint] [backup-dir]
```

Example:
```bash
./backup.sh http://127.0.0.1:2379 /var/backups/etcd
```

### restore.sh

Restore etcd cluster from backup.

```bash
./restore.sh <backup-file> [data-dir] [cluster-token]
```

Example:
```bash
./restore.sh /var/backups/etcd/etcd-backup-20250101-120000.db.gz /var/lib/etcd nnoe-cluster-restored
```

## Requirements

- etcdctl v3.5+
- bash 4.0+

## Cluster Deployment

1. Bootstrap first node:
   ```bash
   ./bootstrap.sh etcd-1 192.168.1.10
   etcd --config-file /etc/etcd/etcd.conf
   ```

2. Add additional nodes:
   ```bash
   ./bootstrap.sh etcd-2 192.168.1.11 nnoe-cluster-1 "etcd-1=http://192.168.1.10:2380"
   etcd --config-file /etc/etcd/etcd.conf
   ```

## Backup Strategy

- Run backups daily using cron
- Keep last 7 days of backups
- Test restore procedures regularly

