# NNOE Monitoring Infrastructure

This directory contains Prometheus exporters and Grafana dashboards for monitoring NNOE deployments.

## Components

### Prometheus Exporter

The Prometheus exporter exposes NNOE metrics on port 9090.

**Build:**
```bash
cd management/monitoring/prometheus-exporter
cargo build --release
```

**Run:**
```bash
./target/release/nnoe-prometheus-exporter
```

**Metrics Endpoint:**
- `http://localhost:9090/metrics` - Prometheus metrics
- `http://localhost:9090/health` - Health check

**Available Metrics:**
- `nnoe_agent_uptime_seconds` - Agent uptime
- `nnoe_agent_etcd_connections` - Number of etcd connections
- `nnoe_agent_cache_size_bytes` - Cache size in bytes
- `nnoe_agent_cache_size_entries` - Number of cache entries
- `nnoe_agent_config_updates_total` - Total config updates
- `nnoe_agent_service_reloads_total` - Total service reloads
- `nnoe_dns_query_rate` - DNS queries per second
- `nnoe_dhcp_lease_count` - Current DHCP lease count
- `nnoe_blocked_queries_total` - Total blocked DNS queries

### Grafana Dashboard

Import the dashboard JSON file into Grafana:

1. Copy `grafana/dashboards/nnoe-dashboard.json` to your Grafana instance
2. Import via Grafana UI: Dashboards → Import → Upload JSON
3. Configure Prometheus data source

The dashboard includes:
- Agent status panels
- Service health indicators
- Performance metrics (latency, QPS)
- Resource usage (CPU, memory, cache)

## Integration

To integrate the exporter with the agent, the agent should expose metrics via HTTP or push them to a metrics endpoint. The current implementation is a standalone exporter that can be extended to collect metrics from running agents.

