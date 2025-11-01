# Monitoring Guide

Comprehensive guide for monitoring NNOE deployments.

## Metrics

### Agent Metrics

Agent exposes metrics via Prometheus (when implemented):

- `nnoe_agent_uptime_seconds`: Agent uptime
- `nnoe_agent_etcd_connections`: etcd connection count
- `nnoe_agent_cache_size_bytes`: Cache size
- `nnoe_agent_config_updates_total`: Config update count
- `nnoe_agent_service_reloads_total`: Service reload count

### Service Metrics

Service-specific metrics:

**DNS (Knot):**
- Query rate
- Response time
- Zone count
- DNSSEC status

**DHCP (Kea):**
- Lease count
- Pool utilization
- Lease duration
- Failover status

**dnsdist:**
- Query rate
- Blocked queries
- Upstream health
- Latency

## Logging

### Log Levels

- `trace`: Very detailed debugging
- `debug`: Debugging information
- `info`: General information
- `warn`: Warning messages
- `error`: Error messages

### Log Format

**JSON Format:**
```toml
[logging]
json = true
```

**Structured Fields:**
- `timestamp`: Event timestamp
- `level`: Log level
- `target`: Logging target
- `message`: Log message
- `fields`: Additional fields

### Log Locations

- **Systemd Journal:** `journalctl -u nnoe-agent`
- **File:** `/var/log/nnoe/agent.log` (if configured)
- **Docker:** `docker logs nnoe-agent`

## Health Checks

### Agent Health

```bash
# Check service status
systemctl status nnoe-agent

# Validate configuration
nnoe-agent validate -c /etc/nnoe/agent.toml

# Check etcd connectivity
etcdctl endpoint health
```

### Service Health

```bash
# DNS service
systemctl status knot
knotc zone-status

# DHCP service
systemctl status kea-dhcp4
kea-shell --host localhost --port 8000 status-get

# dnsdist
systemctl status dnsdist
dnsdist -C /etc/dnsdist/dnsdist.conf --check-config
```

## Alerting

### Key Metrics to Monitor

1. **Agent Availability:**
   - Service running
   - etcd connectivity
   - Configuration valid

2. **Service Availability:**
   - DNS responding
   - DHCP issuing leases
   - dnsdist filtering

3. **Performance:**
   - Config propagation latency
   - Query response time
   - Cache hit rate

4. **Resource Usage:**
   - CPU utilization
   - Memory usage
   - Disk space

### Alert Rules

**Example Prometheus Alert:**

```yaml
groups:
- name: nnoe_alerts
  rules:
  - alert: AgentDown
    expr: up{job="nnoe-agent"} == 0
    for: 1m
    annotations:
      summary: "NNOE agent is down"
  
  - alert: HighConfigLatency
    expr: nnoe_agent_config_latency_seconds > 1
    for: 5m
    annotations:
      summary: "Config propagation latency is high"
```

## Dashboard

### Grafana Dashboard

Key panels:

1. **Agent Status:**
   - Uptime
   - Version
   - etcd connections

2. **Service Status:**
   - DNS query rate
   - DHCP lease count
   - Blocked queries

3. **Performance:**
   - Config update latency
   - Cache hit rate
   - Service reload time

4. **Resources:**
   - CPU usage
   - Memory usage
   - Disk usage

## Monitoring Tools

### Prometheus

Scrape agent metrics:

```yaml
scrape_configs:
  - job_name: 'nnoe-agent'
    static_configs:
      - targets: ['localhost:9090']
```

### Log Aggregation

**ELK Stack:**
- Collect logs from journald
- Parse structured logs
- Create dashboards

**Loki:**
- Lightweight log aggregation
- Prometheus integration
- Grafana integration

### APM Tools

- **Jaeger**: Distributed tracing
- **OpenTelemetry**: Observability framework

## Best Practices

1. **Enable Structured Logging:** Use JSON format
2. **Set Appropriate Log Levels:** Info in production
3. **Monitor Key Metrics:** Availability, latency, errors
4. **Set Up Alerts:** For critical issues
5. **Regular Reviews:** Check logs and metrics weekly
6. **Document Baselines:** Normal operating ranges
7. **Test Alerting:** Verify alerts work correctly

