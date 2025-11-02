# Monitoring Guide

Comprehensive guide for monitoring NNOE deployments.

## Metrics

### Agent Metrics

Agent metrics are exposed via Prometheus exporter on port 9090. The exporter connects directly to etcd (if configured via `ETCD_ENDPOINTS` and `ETCD_PREFIX`) to collect real-time metrics.

#### Core Agent Metrics

**Uptime and Status:**
- `nnoe_agent_uptime_seconds` (Gauge): Agent uptime in seconds
- `nnoe_agent_etcd_connected` (Gauge): etcd connection status (1=connected, 0=disconnected)

**Configuration and Service Management:**
- `nnoe_agent_config_updates_total` (Counter): Total number of configuration updates received from etcd
- `nnoe_agent_service_reloads_total` (Counter): Total number of service reloads performed

**Cache Metrics:**
- `nnoe_agent_cache_size_bytes` (Gauge): Current cache size in bytes
- `nnoe_agent_cache_size_entries` (Gauge): Current number of cache entries

**DNS Metrics:**
- `nnoe_dns_queries_total` (Counter): Total DNS queries processed (from dnsdist)
- `nnoe_dns_query_rate` (Gauge): DNS queries per second (calculated from counter)
- `nnoe_blocked_queries_total` (Counter): Total DNS queries blocked by policy or threat intelligence

**DHCP Metrics:**
- `nnoe_dhcp_leases_total` (Counter): Total number of DHCP leases allocated (cumulative)
- `nnoe_dhcp_leases_active` (Gauge): Current number of active DHCP leases (from etcd)

**HA Metrics:**
- `nnoe_agent_ha_state` (Gauge): HA state (0=Unknown, 1=Primary, 2=Standby)

#### Metrics Collection

The Prometheus exporter:
- Runs continuously, collecting metrics every 10 seconds
- Connects directly to etcd (if `ETCD_ENDPOINTS` and `ETCD_PREFIX` are configured)
- Fetches DHCP lease counts from etcd at `/nnoe/dhcp/leases` prefix
- Calculates DNS query rate from `nnoe_dns_queries_total` counter
- Updates etcd connection status based on connectivity checks

**Configuration:**
```bash
# Environment variables for Prometheus exporter
export ETCD_ENDPOINTS="http://etcd:2379"
export ETCD_PREFIX="/nnoe"
```

### Service Metrics

Service-specific metrics are exposed via the agent's Prometheus exporter:

**DNS (Knot):**
- Metrics are tracked at the agent level:
  - Zone count can be inferred from `/nnoe/dns/zones` keys in etcd
  - DNSSEC status tracked via Knot service health checks
- Agent metrics:
  - `nnoe_dns_queries_total`: Total queries processed
  - `nnoe_dns_query_rate`: Queries per second

**DHCP (Kea):**
- Lease metrics from etcd (collected by Prometheus exporter):
  - `nnoe_dhcp_leases_active`: Current active leases
  - `nnoe_dhcp_leases_total`: Cumulative leases allocated
- HA metrics:
  - `nnoe_agent_ha_state`: Current HA state (Primary/Standby/Unknown)
- Lease data stored in etcd at `/nnoe/dhcp/leases/{lease_id}`

**dnsdist:**
- DNS filtering metrics:
  - `nnoe_dns_queries_total`: Total queries processed
  - `nnoe_blocked_queries_total`: Queries blocked by policy/threats
  - `nnoe_dns_query_rate`: Queries per second
- RPZ zone file generated at `/var/lib/dnsdist/rpz/threats.rpz`
- Threat domains sourced from `/nnoe/threats/domains` in etcd

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

Complete alerting rules are available in `management/monitoring/grafana/alerts/nnoe-alerts.yml`.

**Example Prometheus Alerts:**

```yaml
groups:
- name: nnoe_agent
  rules:
  - alert: NNOEAgentDown
    expr: up{job="nnoe-prometheus-exporter"} == 0
    for: 1m
    annotations:
      summary: "NNOE agent is down"
  
  - alert: NNOEEtcdDisconnected
    expr: nnoe_agent_etcd_connected == 0
    for: 2m
    annotations:
      summary: "etcd connection lost"
  
  - alert: NNOEHighBlockedQueries
    expr: rate(nnoe_blocked_queries_total[5m]) > 100
    for: 5m
    annotations:
      summary: "High rate of blocked DNS queries"

- name: nnoe_dns
  rules:
  - alert: NNOEDNSQueryRateLow
    expr: nnoe_dns_query_rate < 1
    for: 10m
    annotations:
      summary: "DNS query rate unusually low"

- name: nnoe_dhcp
  rules:
  - alert: NNOEDHCPLeaseExhaustion
    expr: (nnoe_dhcp_leases_active / pool_size) > 0.9
    for: 5m
    annotations:
      summary: "DHCP pool nearing exhaustion"
```

## Dashboard

### Grafana Dashboard

A comprehensive Grafana dashboard is provided at `management/monitoring/grafana/dashboards/nnoe-dashboard.json`.

**Key Panels:**

1. **Agent Status:**
   - `nnoe_agent_uptime_seconds`: Agent uptime gauge
   - `nnoe_agent_etcd_connected`: etcd connectivity status
   - `nnoe_agent_ha_state`: HA state (Primary/Standby/Unknown)

2. **Configuration Management:**
   - `nnoe_agent_config_updates_total`: Total config updates (counter)
   - `nnoe_agent_service_reloads_total`: Total service reloads (counter)

3. **Cache Performance:**
   - `nnoe_agent_cache_size_bytes`: Cache size in bytes
   - `nnoe_agent_cache_size_entries`: Number of cache entries

4. **DNS Performance:**
   - `nnoe_dns_queries_total`: Total DNS queries (counter)
   - `nnoe_dns_query_rate`: DNS queries per second (calculated)
   - `nnoe_blocked_queries_total`: Blocked queries (counter)

5. **DHCP Metrics:**
   - `nnoe_dhcp_leases_active`: Active leases (from etcd)
   - `nnoe_dhcp_leases_total`: Total leases allocated (counter)

6. **Resources:**
   - CPU usage (from node exporter)
   - Memory usage (from node exporter)
   - Disk usage (from node exporter)

**Alerting Rules:**

Prometheus alerting rules are defined in `management/monitoring/grafana/alerts/nnoe-alerts.yml`:

- `NNOEAgentDown`: Agent not responding
- `NNOEEtcdDisconnected`: etcd connection lost
- `NNOEHighBlockedQueries`: High rate of blocked queries
- `NNOECacheSizeExceeded`: Cache size exceeds threshold
- `NNOEDNSQueryRateLow`: DNS query rate unusually low
- `NNOEDHCPLeaseExhaustion`: DHCP pool exhaustion warning

## Monitoring Tools

### Prometheus

Scrape agent metrics from Prometheus exporter:

```yaml
scrape_configs:
  - job_name: 'nnoe-agent'
    static_configs:
      - targets: ['localhost:9090']  # Prometheus exporter port
    metrics_path: '/metrics'
    scrape_interval: 30s
```

**Prometheus Exporter:**
- Runs as a separate service: `nnoe-prometheus-exporter`
- Exposes HTTP endpoint on port 9090
- Connects directly to etcd for metrics collection
- Environment variables: `ETCD_ENDPOINTS`, `ETCD_PREFIX`
- Health check endpoint: `/health` on port 9090

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

