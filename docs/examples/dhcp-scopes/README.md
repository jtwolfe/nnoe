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

## IPv6 Scope Configuration

### Creating IPv6 Scope

```bash
cat > scope6.json <<EOF
{
  "subnet": "2001:db8::/64",
  "pool": {
    "start": "2001:db8::100",
    "end": "2001:db8::200"
  },
  "options": {
    "dns-servers": "2001:db8::1"
  }
}
EOF

etcdctl put /nnoe/dhcp/scopes/scope6-1 "$(cat scope6.json)"
```

### IPv6 Lease Data Structure

IPv6 leases include additional fields:
- `type`: IA type (IA_NA, IA_PD)
- `iaid`: Interface Association Identifier
- `duid`: DHCP Unique Identifier
- `preferred_lft`: Preferred lifetime
- `expires_at`: Lease expiration timestamp

```bash
# View IPv6 lease in etcd
etcdctl get /nnoe/dhcp/leases/2001:db8::100
# Returns JSON with IPv6-specific fields
```

## HA Pair Configuration

Configure HA pair in agent config:

```toml
[services.dhcp]
enabled = true
engine = "kea"
config_path = "/etc/kea/kea-dhcp4.conf"
ha_pair_id = "pair-1"
peer_node = "node-2"  # Peer node name
```

**How HA Coordination Works:**

1. **VIP Detection:** Agent checks for VIP using `ip addr show`
2. **State Determination:**
   - Primary: VIP present → Start Kea
   - Standby: VIP absent → Stop Kea
3. **Status Updates:** Agent writes HA status to etcd at `/nnoe/dhcp/ha-pairs/{pair_id}/nodes/{node}/status`
4. **Peer Monitoring:** Agent can query peer status from etcd

**Keepalived Configuration (for VIP management):**

```bash
# /etc/keepalived/keepalived.conf
vrrp_instance VI_1 {
    state MASTER  # or BACKUP
    interface eth0
    virtual_router_id 51
    priority 100  # Higher priority = Primary
    virtual_ipaddress {
        192.168.1.100/24  # VIP
    }
}
```

**Monitoring HA Status:**

```bash
# Check HA status in etcd
etcdctl get /nnoe/dhcp/ha-pairs/pair-1/nodes/node-1/status
# Returns: {"state": "Primary", "timestamp": "2025-01-15T10:30:00Z"}

# Check agent HA state metric
curl http://localhost:9090/metrics | grep nnoe_agent_ha_state
# 0=Unknown, 1=Primary, 2=Standby
```

## Monitoring Leases

### View Active IPv4 Leases

```bash
# Via Kea shell
kea-shell --host localhost --port 8000 lease4-get-all

# Via etcd (synced by Kea hooks)
etcdctl get --prefix /nnoe/dhcp/leases/

# Count active leases
etcdctl get --prefix /nnoe/dhcp/leases/ | grep -c "\"operation\":\"offer\""
```

### View Active IPv6 Leases

```bash
# Via Kea shell
kea-shell --host localhost --port 8000 lease6-get-all

# Via etcd (synced by Kea hooks)
etcdctl get --prefix /nnoe/dhcp/leases/ | grep "\"ip\":\"2001:"

# View IPv6 lease details
etcdctl get /nnoe/dhcp/leases/2001:db8::100
```

### Find Lease by MAC (IPv4) or DUID (IPv6)

```bash
# IPv4: Find by MAC address
kea-shell --host localhost --port 8000 lease4-get-by-hw-address 00:11:22:33:44:55

# IPv6: Find by DUID
kea-shell --host localhost --port 8000 lease6-get-by-duid 00:01:00:01:1a:2b:3c:4d:5e:6f
```

### Lease Expiration Handling

Leases automatically expire based on `expires_at` timestamp. Kea hooks call `lease4_expire` and `lease6_expire` to delete expired leases from etcd.

**Check Lease Expiration:**

```bash
# View lease with expiration
etcdctl get /nnoe/dhcp/leases/192.168.1.100 | jq '.expires_at'
# Returns Unix timestamp

# Convert to readable date
date -d @$(etcdctl get /nnoe/dhcp/leases/192.168.1.100 | jq -r '.expires_at')
```

**Manual Lease Cleanup:**

```bash
# Delete expired lease manually if needed
etcdctl del /nnoe/dhcp/leases/192.168.1.100
```

## Best Practices

1. **Pool Sizing:** Allocate sufficient pool size for both IPv4 and IPv6
2. **Dual Stack:** Configure both IPv4 and IPv6 scopes for complete coverage
3. **Lease Duration:** Set appropriate lease times (consider `expires_at` tracking)
4. **Options:** Use standard DHCP options for both IPv4 and IPv6
5. **Monitoring:** Track pool utilization via etcd lease counts
6. **Reservations:** Use static reservations for servers
7. **HA Pairs:** Always use HA pairs for production DHCP deployments
8. **VIP Management:** Ensure Keepalived or similar manages VIP correctly
9. **Status Monitoring:** Monitor HA state via etcd status keys or metrics (`nnoe_agent_ha_state`)
10. **Lease Expiration:** Monitor `expires_at` timestamps for lease lifecycle management

