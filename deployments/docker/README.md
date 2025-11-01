# Docker Deployment

Docker Compose configurations for deploying NNOE.

## Development

```bash
cd deployments/docker
docker-compose -f docker-compose.dev.yml up -d
```

This starts:
- etcd (single node)
- NNOE agent
- MISP sync service
- phpIPAM (optional)

## Production

```bash
docker-compose -f docker-compose.prod.yml up -d
```

This starts:
- etcd cluster (3 nodes)
- HAProxy load balancer
- Multiple agent replicas
- MISP sync service

### Environment Variables

Set before running:

```bash
export NODE_NAME=agent-1
export MISP_URL=https://misp.example.com
export MISP_API_KEY=your-api-key
```

### Volumes

- `etcd-data`: etcd cluster data
- `agent-config`: Agent configuration
- `agent-cache`: Agent cache storage
- `agent-logs`: Agent log files

## Building Images

```bash
# Build agent
docker build -f Dockerfile.agent -t nnoe-agent:latest ../..

# Build MISP sync
docker build -f Dockerfile.misp-sync -t nnoe-misp-sync:latest ../..
```

## Health Checks

All services include health checks. Check status:

```bash
docker-compose ps
```

## Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f agent
```

