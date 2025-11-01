# Threat Intelligence Integration Examples

Examples for integrating MISP threat feeds with NNOE.

## MISP Sync Configuration

### Environment Variables

```bash
export MISP_URL="https://misp.example.com"
export MISP_API_KEY="your-api-key"
export ETCD_ENDPOINTS="http://127.0.0.1:2379"
export ETCD_PREFIX="/nnoe"
export SYNC_INTERVAL_SECS=3600
```

### Running MISP Sync

```bash
# Docker
docker run -d \
  -e MISP_URL=https://misp.example.com \
  -e MISP_API_KEY=your-key \
  -e ETCD_ENDPOINTS=http://etcd:2379 \
  nnoe-misp-sync:latest

# Binary
./misp-sync
```

## Manual Threat Entry

### Add Threat Domain

```bash
cat > threat.json <<EOF
{
  "domain": "malicious.example.com",
  "source": "Manual",
  "severity": "high",
  "timestamp": "2025-01-01T12:00:00Z",
  "category": "malware"
}
EOF

etcdctl put /nnoe/threats/domains/malicious.example.com "$(cat threat.json)"
```

### Verify Threat Blocking

```bash
# Query blocked domain
dig malicious.example.com

# Check dnsdist logs
journalctl -u dnsdist | grep malicious.example.com
```

## Threat Feed Sources

### MISP Feed

Automatically synced via misp-sync service.

### Custom Feeds

Create custom sync service:

```rust
// Sync from custom source
let threats = fetch_threats_from_source().await?;
for threat in threats {
    etcd_client.put(
        format!("/nnoe/threats/domains/{}", threat.domain),
        serde_json::to_string(&threat)?
    ).await?;
}
```

## RPZ Configuration

dnsdist automatically generates RPZ rules from threat feeds.

### Verify RPZ Rules

```bash
# Check Lua script
cat /etc/dnsdist/lua/rules.lua | grep malicious
```

### Manual Block

```bash
# Add to etcd
etcdctl put /nnoe/threats/domains/blocked.example.com '{"domain":"blocked.example.com","source":"Manual","severity":"high"}'

# Reload dnsdist
dnsdist -C /etc/dnsdist/dnsdist.conf reload
```

## Threat Severity Levels

- **high**: Critical threats, immediate blocking
- **medium**: Moderate threats, review recommended
- **low**: Low risk, optional blocking

## Best Practices

1. **Regular Sync:** Sync threat feeds frequently
2. **Verify Sources:** Trust only verified sources
3. **Monitor Blocks:** Review blocked domains
4. **Whitelist:** Maintain whitelist for false positives
5. **Alerting:** Alert on high-severity threats

