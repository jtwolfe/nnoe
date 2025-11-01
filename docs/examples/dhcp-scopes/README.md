# DHCP Scope Management Examples

Examples for managing DHCP scopes with NNOE.

## Creating a Scope

### Via phpIPAM

```php
$nnoe = new NNOE();
$scope = [
    'subnet' => '192.168.1.0/24',
    'pool' => ['start' => '192.168.1.100', 'end' => '192.168.1.200'],
    'gateway' => '192.168.1.1',
    'options' => [
        'router' => '192.168.1.1',
        'dns-servers' => '192.168.1.1,8.8.8.8',
        'domain-name' => 'example.com'
    ]
];
$nnoe->pushDhcpScope('scope-1', $scope);
```

### Via etcd

```bash
cat > scope.json <<EOF
{
  "subnet": "192.168.1.0/24",
  "pool": {
    "start": "192.168.1.100",
    "end": "192.168.1.200"
  },
  "gateway": "192.168.1.1",
  "options": {
    "router": "192.168.1.1",
    "dns-servers": "192.168.1.1,8.8.8.8"
  }
}
EOF

etcdctl put /nnoe/dhcp/scopes/scope-1 "$(cat scope.json)"
```

## Updating a Scope

### Modify Pool Range

```bash
etcdctl get /nnoe/dhcp/scopes/scope-1 | \
  jq '.pool.start = "192.168.1.150" | .pool.end = "192.168.1.250"' | \
  etcdctl put /nnoe/dhcp/scopes/scope-1
```

### Add DHCP Options

```bash
etcdctl get /nnoe/dhcp/scopes/scope-1 | \
  jq '.options["ntp-servers"] = "192.168.1.1"' | \
  etcdctl put /nnoe/dhcp/scopes/scope-1
```

## HA Pair Configuration

Configure HA pair in agent config:

```toml
[services.dhcp]
enabled = true
engine = "kea"
config_path = "/etc/kea/kea-dhcp4.conf"
ha_pair_id = "pair-1"
```

Use Keepalived for VIP failover.

## Monitoring Leases

### View Active Leases

```bash
kea-shell --host localhost --port 8000 lease4-get-all
```

### Find Lease by MAC

```bash
kea-shell --host localhost --port 8000 lease4-get-by-hw-address 00:11:22:33:44:55
```

## Best Practices

1. **Pool Sizing:** Allocate sufficient pool size
2. **Lease Duration:** Set appropriate lease times
3. **Options:** Use standard DHCP options
4. **Monitoring:** Track pool utilization
5. **Reservations:** Use static reservations for servers

