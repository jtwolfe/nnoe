# Docker Deployment Guide

Complete guide for deploying NNOE using Docker Compose.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- 2GB+ RAM available
- 10GB+ disk space

## Quick Start

```bash
cd deployments/docker
docker-compose -f docker-compose.dev.yml up -d
```

## Development Deployment

Development setup uses a single etcd node and is suitable for testing.

### Start Services

```bash
docker-compose -f docker-compose.dev.yml up -d
```

### Verify

```bash
# Check all services are running
docker-compose ps

# View logs
docker-compose logs -f agent

# Test etcd connection
docker exec nnoe-etcd-dev etcdctl endpoint health
```

### Configuration

Edit environment variables in `docker-compose.dev.yml` or use `.env` file:

**Agent Environment Variables:**
```yaml
environment:
  - NODE_NAME=${NODE_NAME:-dev-agent-1}
  - ETCD_ENDPOINTS=http://etcd:2379
  - ETCD_PREFIX=${ETCD_PREFIX:-/nnoe}
  - LOG_LEVEL=${LOG_LEVEL:-info}
  - RUST_LOG=${RUST_LOG:-nnoe_agent=info}
  - NODE_ROLE=${NODE_ROLE:-agent}  # 'agent' or 'db-only'
```

**MISP Sync Environment Variables:**
```yaml
environment:
  - MISP_URL=${MISP_URL:-http://localhost}
  - MISP_API_KEY=${MISP_API_KEY:-}
  - MISP_URL_2=${MISP_URL_2:-}  # Optional second instance
  - MISP_API_KEY_2=${MISP_API_KEY_2:-}
  - MISP_FILTER_TAGS=${MISP_FILTER_TAGS:-}  # Comma-separated tags
  - MISP_DEDUP=${MISP_DEDUP:-true}  # Enable deduplication
  - ETCD_ENDPOINTS=http://etcd:2379
  - ETCD_PREFIX=${ETCD_PREFIX:-/nnoe}
  - SYNC_INTERVAL_SECS=${SYNC_INTERVAL_SECS:-3600}
```

**phpIPAM Environment Variables:**
```yaml
environment:
  - NNOE_ETCD_ENDPOINTS=http://etcd:2379
  - NNOE_ETCD_PREFIX=${ETCD_PREFIX:-/nnoe}
  - NNOE_ENABLE_DNS_MANAGEMENT=${NNOE_ENABLE_DNS_MANAGEMENT:-true}
  - NNOE_ENABLE_DHCP_MANAGEMENT=${NNOE_ENABLE_DHCP_MANAGEMENT:-true}
  - NNOE_ENABLE_THREAT_VIEWER=${NNOE_ENABLE_THREAT_VIEWER:-true}
  - NNOE_GRAFANA_URL=${NNOE_GRAFANA_URL:-}  # Optional Grafana URL
  - NNOE_GRAFANA_DASHBOARD=${NNOE_GRAFANA_DASHBOARD:-}  # Optional dashboard ID
  - NNOE_ETCD_TLS_CA_CERT=${NNOE_ETCD_TLS_CA_CERT:-}  # Optional TLS CA cert path
  - NNOE_ETCD_TLS_CERT=${NNOE_ETCD_TLS_CERT:-}  # Optional TLS client cert path
  - NNOE_ETCD_TLS_KEY=${NNOE_ETCD_TLS_KEY:-}  # Optional TLS client key path
  - NNOE_ETCD_TLS_VERIFY=${NNOE_ETCD_TLS_VERIFY:-true}  # TLS verification
```

Agent configuration is mounted from `agent-config` volume. To update:

```bash
# Create config file
cat > /tmp/agent.toml <<EOF
[node]
name = "dev-agent-1"
role = "active"
...
EOF

# Copy to volume
docker cp /tmp/agent.toml nnoe-agent-dev:/etc/nnoe/agent.toml
docker restart nnoe-agent-dev
```

## Production Deployment

Production setup includes HA etcd cluster and load balancing.

### Setup

1. **Create environment file:**

```bash
cat > .env <<EOF
# Node Configuration
NODE_NAME=agent-prod-1
NODE_ROLE=agent  # or 'db-only' for DB-only nodes

# etcd Configuration
ETCD_ENDPOINTS=http://etcd-lb:2379
ETCD_PREFIX=/nnoe
ETCD_CLUSTER_TOKEN=nnoe-cluster-prod

# MISP Configuration
MISP_URL=https://misp.example.com
MISP_API_KEY=your-api-key-here
MISP_URL_2=https://misp2.example.com  # Optional second instance
MISP_API_KEY_2=your-api-key-2
MISP_FILTER_TAGS=malware,phishing  # Optional tag filter
MISP_DEDUP=true  # Enable deduplication

# Logging
LOG_LEVEL=info
RUST_LOG=nnoe_agent=info

# Scaling
AGENT_REPLICAS=2
ETCD_LB_PORT=2379
PHPIPAM_PORT=8080
EOF
```

2. **Start services:**

```bash
docker-compose -f docker-compose.prod.yml up -d
```

### High Availability

The production setup includes:

- **etcd Cluster**: 3 nodes for quorum
- **HAProxy**: Load balancer for etcd access
- **Agent Replicas**: Multiple agent instances

### Scaling

Scale agent replicas:

```bash
docker-compose -f docker-compose.prod.yml up -d --scale agent=4
```

### Data Persistence

Volumes persist data:

```bash
# Backup etcd data
docker run --rm -v nnoe-etcd-1-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/etcd-backup.tar.gz -C /data .

# Restore
docker run --rm -v nnoe-etcd-1-data:/data -v $(pwd):/backup \
  alpine tar xzf /backup/etcd-backup.tar.gz -C /data
```

## Building Images

### Agent Image

```bash
docker build -f deployments/docker/Dockerfile.agent \
  -t nnoe-agent:latest ../..
```

### MISP Sync Image

```bash
docker build -f deployments/docker/Dockerfile.misp-sync \
  -t nnoe-misp-sync:latest ../..
```

## Networking

### Default Network

Services use `nnoe-dev` (dev) or `nnoe-internal` (prod) network.

### External Access

Expose ports:

**Development (`docker-compose.dev.yml`):**
```yaml
ports:
  - "2379:2379"  # etcd client port
  - "2380:2380"  # etcd peer port
  - "${PHPIPAM_PORT:-8080}:80"  # phpIPAM web interface
```

**Production (`docker-compose.prod.yml`):**
```yaml
ports:
  - "${ETCD_LB_PORT:-2379}:2379"  # HAProxy etcd load balancer
  - "9090:9090"  # Prometheus exporter (if deployed)
```

**Agent Health and Metrics:**
- Health endpoint: `http://localhost:8080/health` (internal container port)
- Metrics endpoint: `http://localhost:9090/metrics` (if Prometheus exporter deployed)

**Note**: Production setup uses HAProxy load balancer for etcd access. Direct etcd ports are not exposed externally for security.

### Network Isolation

Services communicate via Docker network. No external access required for inter-service communication.

## Troubleshooting

### Agent Not Starting

```bash
# Check logs
docker logs nnoe-agent-dev

# Verify etcd connectivity
docker exec nnoe-agent-dev nnoe-agent validate -c /etc/nnoe/agent.toml

# Check etcd is healthy
docker exec nnoe-etcd-dev etcdctl endpoint health
```

### etcd Connection Issues

```bash
# Verify etcd is listening
docker exec nnoe-etcd-dev netstat -tlnp | grep 2379

# Test connection from agent
docker exec nnoe-agent-dev curl http://etcd:2379/health
```

### Volume Issues

```bash
# List volumes
docker volume ls | grep nnoe

# Inspect volume
docker volume inspect nnoe-etcd-data

# Remove volume (data loss!)
docker volume rm nnoe-etcd-data
```

## Health Checks

All services include health checks with `service_healthy` dependency conditions.

**Check Service Health:**
```bash
# View all services and their health status
docker-compose -f docker-compose.dev.yml ps

# Check specific service health
docker inspect nnoe-agent-dev | jq '.[0].State.Health'

# View detailed health check status
docker inspect --format='{{json .State.Health}}' nnoe-etcd-dev | jq
```

**Health Check Configurations:**

- **etcd**: `etcdctl endpoint health`
  - Interval: 10s, Timeout: 5s, Retries: 5
  - Start period: 30s (production) or immediate (dev)

- **Agent**: HTTP GET `http://localhost:8080/health`
  - Interval: 30s, Timeout: 10s, Retries: 3
  - Start period: 40s (dev) or 60s (production)
  - Requires `curl` in container

- **MISP Sync**: Process check via `pgrep -f misp-sync`
  - Interval: 60s, Timeout: 5s, Retries: 3
  - Start period: 10s

- **phpIPAM**: HTTP GET `http://localhost/index.php`
  - Interval: 30s, Timeout: 10s, Retries: 3
  - Start period: 60s

- **etcd-lb (HAProxy)**: Configuration check `haproxy -c -f /usr/local/etc/haproxy/haproxy.cfg`
  - Interval: 10s, Timeout: 5s, Retries: 3

**Dependency Chains:**
- Agent depends on `etcd` being healthy (dev) or `etcd-lb` being healthy (prod)
- MISP sync depends on `etcd` being healthy
- phpIPAM depends on `etcd` being healthy
- Services wait for dependencies to be healthy before starting

## Updating

```bash
# Pull latest images
docker-compose pull

# Rebuild and restart
docker-compose up -d --build
```

## Cleanup

```bash
# Stop services
docker-compose down

# Remove volumes (data loss!)
docker-compose down -v

# Remove images
docker rmi nnoe-agent nnoe-misp-sync
```

## Best Practices

1. **Use named volumes** for persistent data
2. **Set resource limits** in production
3. **Use secrets** for sensitive data (MISP API keys)
4. **Enable health checks** for all services
5. **Monitor logs** regularly
6. **Backup volumes** before updates

