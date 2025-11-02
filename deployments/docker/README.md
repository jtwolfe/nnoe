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

**Quick Setup:**

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your configuration:
   ```bash
   # Required
   NODE_NAME=agent-1
   MISP_URL=https://misp.example.com
   MISP_API_KEY=your-api-key
   
   # Optional
   ETCD_PREFIX=/nnoe
   LOG_LEVEL=info
   AGENT_REPLICAS=2
   ```

3. Start services:
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

**Available Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `NODE_NAME` | `agent` | Unique agent node name |
| `NODE_ROLE` | `agent` | Node role: `agent` or `db-only` |
| `ETCD_ENDPOINTS` | `http://etcd-lb:2379` | etcd cluster endpoints (comma-separated) |
| `ETCD_PREFIX` | `/nnoe` | etcd key prefix |
| `ETCD_CLUSTER_TOKEN` | `nnoe-cluster-prod` | etcd cluster token |
| `MISP_URL` | - | Primary MISP instance URL |
| `MISP_API_KEY` | - | Primary MISP API key |
| `MISP_URL_2` | - | Secondary MISP instance URL (optional) |
| `MISP_API_KEY_2` | - | Secondary MISP API key (optional) |
| `MISP_FILTER_TAGS` | - | Comma-separated tag filter (optional) |
| `MISP_DEDUP` | `true` | Enable deduplication |
| `SYNC_INTERVAL_SECS` | `3600` | MISP sync interval in seconds |
| `LOG_LEVEL` | `info` | Log level (debug, info, warn, error) |
| `RUST_LOG` | `nnoe_agent=info` | Rust logging filter |
| `AGENT_REPLICAS` | `2` | Number of agent replicas (production) |
| `PHPIPAM_PORT` | `8080` | phpIPAM web interface port |
| `ETCD_LB_PORT` | `2379` | HAProxy etcd load balancer port |

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
# View all services
docker-compose -f docker-compose.prod.yml ps

# Check specific service health
docker inspect --format='{{.State.Health.Status}}' nnoe-agent-dev

# View health check logs
docker inspect --format='{{json .State.Health}}' nnoe-etcd-dev | jq
```

**Health Check Endpoints:**

- **Agent**: `http://localhost:8080/health` (if health endpoint implemented)
- **etcd**: `etcdctl endpoint health`
- **MISP Sync**: Process check via `pgrep`
- **phpIPAM**: `http://localhost:8080/index.php`

## Logs

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f agent
```

