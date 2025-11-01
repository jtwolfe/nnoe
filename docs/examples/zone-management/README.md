# DNS Zone Management Examples

Examples for managing DNS zones with NNOE.

## Creating a Zone

### Via phpIPAM

```php
$nnoe = new NNOE();
$records = [
    ['name' => '@', 'type' => 'A', 'value' => '192.168.1.1'],
    ['name' => 'www', 'type' => 'A', 'value' => '192.168.1.2'],
    ['name' => 'mail', 'type' => 'A', 'value' => '192.168.1.3'],
    ['name' => '@', 'type' => 'MX', 'value' => '10 mail.example.com']
];
$nnoe->pushDnsZone('example.com', $records);
```

### Via etcd

```bash
# Create zone JSON
cat > zone.json <<EOF
{
  "domain": "example.com",
  "ttl": 3600,
  "records": [
    {"name": "@", "type": "A", "value": "192.168.1.1"},
    {"name": "www", "type": "A", "value": "192.168.1.2"}
  ]
}
EOF

# Push to etcd
etcdctl put /nnoe/dns/zones/example.com "$(cat zone.json)"
```

## Updating a Zone

### Add Record

```bash
# Get current zone
etcdctl get /nnoe/dns/zones/example.com | jq '.records += [{"name": "api", "type": "A", "value": "192.168.1.4"}]' | etcdctl put /nnoe/dns/zones/example.com
```

### Update Record

```bash
# Update A record
etcdctl get /nnoe/dns/zones/example.com | jq '.records[] |= if .name == "www" then .value = "192.168.1.10" else . end' | etcdctl put /nnoe/dns/zones/example.com
```

## Deleting a Zone

```bash
etcdctl del /nnoe/dns/zones/example.com
```

## DNSSEC

DNSSEC is automatically enabled in Knot. Keys are managed automatically.

### Verify DNSSEC

```bash
dig +dnssec example.com
```

## Bulk Operations

### Import from BIND Zone File

```bash
# Convert BIND format to JSON
# (requires custom script)
bind-to-json example.com.db | etcdctl put /nnoe/dns/zones/example.com
```

### Export to Zone File

```bash
etcdctl get /nnoe/dns/zones/example.com | jq -r '.records[] | "\(.name)\t\(.ttl // 3600)\t\(.type)\t\(.value)"'
```

## Best Practices

1. Use consistent TTL values
2. Keep zone data in etcd, not zone files
3. Use DNS record validation
4. Monitor zone propagation
5. Regular zone audits

