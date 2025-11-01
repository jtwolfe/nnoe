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

Edit environment variables in `docker-compose.dev.yml`:

```yaml
environment:
  - NODE_NAME=dev-agent-1
  - ETCD_ENDPOINTS=http://etcd:2379
  - ETCD_PREFIX=/nnoe
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
NODE_NAME=agent-prod-1
MISP_URL=https://misp.example.com
MISP_API_KEY=your-api-key-here
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

```yaml
ports:
  - "2379:2379"  # etcd
  - "8080:80"    # phpIPAM
```

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

All services include health checks:

```bash
# Check service health
docker-compose ps

# View health status
docker inspect nnoe-agent-dev | jq '.[0].State.Health'
```

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

