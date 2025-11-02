# Threat Intelligence Integration Examples

Examples for integrating MISP threat feeds with NNOE.

## MISP Sync Configuration

### Single Instance Configuration

```bash
export MISP_URL="https://misp.example.com"
export MISP_API_KEY="your-api-key"
export ETCD_ENDPOINTS="http://127.0.0.1:2379"
export ETCD_PREFIX="/nnoe"
export SYNC_INTERVAL_SECS=3600
```

### Multiple Instance Configuration

MISP sync supports multiple MISP instances for aggregated threat feeds:

```bash
# Primary instance
export MISP_URL="https://misp1.example.com"
export MISP_API_KEY="api-key-1"

# Secondary instance (optional)
export MISP_URL_2="https://misp2.example.com"
export MISP_API_KEY_2="api-key-2"

# Tag filtering (optional)
export MISP_FILTER_TAGS="malware,phishing,apt"
# Only sync events with these tags

# Deduplication (default: true)
export MISP_DEDUP="true"
# Deduplicates threats by domain across all instances

export ETCD_ENDPOINTS="http://127.0.0.1:2379"
export ETCD_PREFIX="/nnoe"
export SYNC_INTERVAL_SECS=3600
```

**Tag Filtering Example:**

```bash
# Only sync events tagged with "malware" or "phishing"
export MISP_FILTER_TAGS="malware,phishing"

# Sync all events (no filtering)
# Omit MISP_FILTER_TAGS or set to empty string
```

**Deduplication:**

- Enabled by default (`MISP_DEDUP=true`)
- Uses HashSet to track processed domains
- Prevents duplicate entries across multiple MISP instances
- Maintains single threat entry per domain in etcd

### Running MISP Sync

```bash
# Docker - Single Instance
docker run -d \
  -e MISP_URL=https://misp.example.com \
  -e MISP_API_KEY=your-key \
  -e ETCD_ENDPOINTS=http://etcd:2379 \
  nnoe-misp-sync:latest

# Docker - Multiple Instances with Tag Filtering
docker run -d \
  -e MISP_URL=https://misp1.example.com \
  -e MISP_API_KEY=key1 \
  -e MISP_URL_2=https://misp2.example.com \
  -e MISP_API_KEY_2=key2 \
  -e MISP_FILTER_TAGS="malware,phishing" \
  -e MISP_DEDUP="true" \
  -e ETCD_ENDPOINTS=http://etcd:2379 \
  -e ETCD_PREFIX="/nnoe" \
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

## Tag Filtering Examples

### Filter by Single Tag

```bash
# Only sync malware events
export MISP_FILTER_TAGS="malware"
./misp-sync
```

### Filter by Multiple Tags

```bash
# Sync events with any of these tags
export MISP_FILTER_TAGS="malware,phishing,apt,ransomware"
./misp-sync
```

### Verify Filtered Sync

```bash
# Check threat entries in etcd
etcdctl get --prefix /nnoe/threats/domains/

# Check MISP sync logs for filtered events
journalctl -u misp-sync | grep "filtered\|synced"
```

## Deduplication Examples

### With Deduplication (Default)

```bash
export MISP_DEDUP="true"
./misp-sync

# Result: Each domain appears once in etcd, even if present in multiple MISP instances
```

### Without Deduplication

```bash
export MISP_DEDUP="false"
./misp-sync

# Result: Domains may appear multiple times (once per MISP instance)
# Not recommended for production
```

### Verify Deduplication

```bash
# Count unique domains in etcd
etcdctl get --prefix /nnoe/threats/domains/ | jq -r '.domain' | sort -u | wc -l

# Should match total count if deduplication working
etcdctl get --prefix /nnoe/threats/domains/ | wc -l
```

## Best Practices

1. **Multiple Instances:** Use multiple MISP instances for comprehensive threat coverage
2. **Tag Filtering:** Use tag filtering to focus on relevant threats (e.g., `malware,phishing`)
3. **Deduplication:** Always enable deduplication (`MISP_DEDUP=true`) to prevent duplicate entries
4. **Regular Sync:** Sync threat feeds frequently (default: 3600 seconds)
5. **Verify Sources:** Trust only verified MISP instances
6. **Monitor Blocks:** Review blocked domains regularly
7. **Whitelist:** Maintain whitelist for false positives
8. **Alerting:** Alert on high-severity threats
9. **RPZ Generation:** Verify dnsdist RPZ zone file is updated with threats

