# Kubernetes Deployment Guide

Complete guide for deploying NNOE on Kubernetes.

## Prerequisites

- Kubernetes cluster 1.20+
- kubectl configured
- StorageClass for persistent volumes
- 4GB+ RAM per node
- 20GB+ disk per node

## Quick Start

```bash
# Create namespace and deploy
kubectl apply -k deployments/kubernetes/

# Verify deployment
kubectl get pods -n nnoe
```

## Components

### Namespace

Create NNOE namespace:

```bash
kubectl apply -f deployments/kubernetes/management/namespace.yaml
# Or: kubectl create namespace nnoe
```

### etcd StatefulSet

3-node etcd cluster for high availability.

**StatefulSet** (`deployments/kubernetes/management/etcd-statefulset.yaml`):
- 3 replicas for quorum
- PersistentVolumeClaims for data persistence
- Health checks via `etcdctl endpoint health`
- Resource limits: 512Mi-2Gi memory, 250m-1000m CPU

**Service** (`deployments/kubernetes/management/etcd-service.yaml`):
- Headless service (ClusterIP: None) for StatefulSet
- Exposes client port (2379) and peer port (2380)

```bash
# Check etcd cluster
kubectl get statefulset -n nnoe etcd
kubectl get pods -n nnoe -l app=etcd

# Check etcd health
kubectl exec -n nnoe etcd-0 -- etcdctl endpoint health

# Check etcd service
kubectl get service -n nnoe etcd
```

### Agent DaemonSet

Deploys agent to all cluster nodes with hostNetwork for DNS/DHCP access.

```bash
# Check agent deployment
kubectl get daemonset -n nnoe nnoe-agent

# View agent pods
kubectl get pods -n nnoe -l app=nnoe-agent
```

**Key Features:**
- Uses `hostNetwork: true` for DNS (UDP 53) and DHCP (UDP 67/68) access
- Health probes: HTTP GET on port 8080 (`/health`)
- Metrics port: 9090 (exposed via Service)
- Resource limits: Requests (256Mi memory, 200m CPU), Limits (1Gi memory, 1000m CPU)
- Capabilities: NET_ADMIN, SYS_ADMIN (for network operations)

## Configuration

### Agent ConfigMap

Edit configuration:

```bash
kubectl edit configmap -n nnoe nnoe-agent-config
```

Or apply updated ConfigMap:

```bash
kubectl apply -f deployments/kubernetes/agent/configmap.yaml
```

Configuration is mounted at `/etc/nnoe/agent.toml` in agent pods.

**ConfigMap includes:**
- Node configuration (name, role: `agent` or `db-only`)
- etcd configuration (endpoints, prefix, optional TLS)
- Service configurations (DNS, DHCP, dnsdist, Cerbos, Lynis)
- Cache configuration
- Nebula configuration (if enabled)

**To set DB-only role:**
```yaml
[node]
role = "db-only"  # Skips service registration, maintains only etcd replication
```

### Environment Variables

Agent environment variables (in DaemonSet):

```yaml
env:
- name: NODE_NAME
  valueFrom:
    fieldRef:
      fieldPath: spec.nodeName  # Uses Kubernetes node name
- name: ETCD_ENDPOINTS
  value: "http://etcd.nnoe.svc.cluster.local:2379"  # etcd service DNS name
- name: ETCD_PREFIX
  value: "/nnoe"
```

**MISP Sync Deployment** (`deployments/kubernetes/misp-sync/deployment.yaml`):
- Environment variables from Secret: `MISP_URL`, `MISP_API_KEY`
- Optional: `MISP_URL_2`, `MISP_API_KEY_2` (second instance)
- `MISP_FILTER_TAGS`, `MISP_DEDUP` (tag filtering and deduplication)
- `ETCD_ENDPOINTS`, `ETCD_PREFIX`, `SYNC_INTERVAL_SECS`

## Scaling

### etcd Cluster

etcd must have odd number of nodes (1, 3, or 5) for quorum.

```bash
# Scale to 5 nodes (requires 5 nodes in cluster)
kubectl scale statefulset etcd -n nnoe --replicas=5
```

### Agent

Agent automatically scales with cluster nodes via DaemonSet.

**DaemonSet Features:**
- Deploys one pod per node
- Uses `hostNetwork: true` for DNS/DHCP access
- Health probes on port 8080
- Metrics exposed on port 9090
- Resource limits configured

**To limit agent to specific nodes:**

Add to DaemonSet spec:
```yaml
nodeSelector:
  nnoe-agent: "enabled"
tolerations:
- key: nnoe-agent
  operator: Exists
  effect: NoSchedule
```

**DB-Only Nodes:**
Set `node.role = "db-only"` in ConfigMap to create DB-only agent nodes that only maintain etcd replication without DNS/DHCP services.

## Storage

### Persistent Volumes

etcd uses PersistentVolumeClaims:

```bash
# Check PVCs
kubectl get pvc -n nnoe

# Check PVs
kubectl get pv | grep nnoe
```

### Storage Requirements

- etcd: 10GB per node
- Agent cache: ephemeral (emptyDir)

## Networking

### Service Discovery

**etcd Service** (`deployments/kubernetes/management/etcd-service.yaml`):
- Headless service (ClusterIP: None) for StatefulSet
- Client port: 2379
- Peer port: 2380
- Accessible via: `http://etcd.nnoe.svc.cluster.local:2379`

**Agent Service** (`deployments/kubernetes/agent/service.yaml`):
- ClusterIP service
- Metrics port: 9090
- Health port: 8080
- Accessible via: `http://nnoe-agent.nnoe.svc.cluster.local:9090` (metrics)

### DNS

Services can resolve via Kubernetes DNS:

- `etcd.nnoe.svc.cluster.local` - etcd service
- `etcd-0.etcd.nnoe.svc.cluster.local` - etcd StatefulSet pod
- `nnoe-agent.nnoe.svc.cluster.local` - agent service

### MISP Sync Deployment

MISP sync service is deployed as a Deployment:

```bash
# Check MISP sync deployment
kubectl get deployment -n nnoe nnoe-misp-sync

# View pods
kubectl get pods -n nnoe -l app=nnoe-misp-sync

# Check logs
kubectl logs -n nnoe -l app=nnoe-misp-sync -f
```

**Secret Configuration** (`deployments/kubernetes/misp-sync/secret.yaml.example`):
- Create secret from example:
  ```bash
  kubectl create secret generic misp-credentials -n nnoe \
    --from-literal=misp_url=https://misp.example.com \
    --from-literal=misp_api_key=your-api-key
  ```

## Updates

### Rolling Update

```bash
# Update image
kubectl set image daemonset/nnoe-agent \
  agent=ghcr.io/nnoe/agent:v0.2.0 -n nnoe

# Or use kustomize
kubectl apply -k deployments/kubernetes/
```

### etcd Upgrade

etcd upgrades require careful orchestration:

1. Upgrade one node at a time
2. Ensure quorum maintained
3. Verify cluster health after each node

## Monitoring

### Pod Logs

```bash
# All agent logs
kubectl logs -n nnoe -l app=nnoe-agent -f

# Specific pod
kubectl logs -n nnoe nnoe-agent-<node-name> -f
```

### Health Checks

Agent pods include liveness and readiness probes:

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 60
  periodSeconds: 30
  timeoutSeconds: 10
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 20
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
```

**Check Pod Health:**
```bash
# View all pods and status
kubectl get pods -n nnoe

# Describe specific pod
kubectl describe pod -n nnoe nnoe-agent-<node-name>

# Check health endpoint directly
kubectl port-forward -n nnoe nnoe-agent-<node-name> 8080:8080
curl http://localhost:8080/health
```

### Metrics

Agent exposes metrics via Service and ServiceMonitor.

**Service** (`deployments/kubernetes/agent/service.yaml`):
```yaml
apiVersion: v1
kind: Service
metadata:
  name: nnoe-agent
  namespace: nnoe
spec:
  ports:
  - name: metrics
    port: 9090
    targetPort: 9090
  - name: health
    port: 8080
    targetPort: 8080
  type: ClusterIP
```

**ServiceMonitor** (`deployments/kubernetes/agent/servicemonitor.yaml`):
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: nnoe-agent
  namespace: nnoe
spec:
  selector:
    matchLabels:
      app: nnoe-agent
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

**Metrics Available:**
- `nnoe_agent_uptime_seconds`
- `nnoe_agent_etcd_connected`
- `nnoe_agent_config_updates_total`
- `nnoe_agent_service_reloads_total`
- `nnoe_dns_queries_total`
- `nnoe_blocked_queries_total`
- `nnoe_dhcp_leases_active`
- `nnoe_agent_ha_state`
- See `docs/operations/monitoring.md` for complete list

## Troubleshooting

### Agent Not Starting

```bash
# Check pod events
kubectl describe pod -n nnoe nnoe-agent-<node-name>

# Check logs
kubectl logs -n nnoe nnoe-agent-<node-name>

# Validate config
kubectl exec -n nnoe nnoe-agent-<node-name> -- \
  nnoe-agent validate -c /etc/nnoe/agent.toml
```

### etcd Issues

```bash
# Check etcd cluster status
kubectl exec -n nnoe etcd-0 -- etcdctl member list

# Check cluster health
kubectl exec -n nnoe etcd-0 -- etcdctl endpoint health

# View etcd logs
kubectl logs -n nnoe etcd-0
```

### Storage Issues

```bash
# Check PVC status
kubectl describe pvc -n nnoe etcd-data-etcd-0

# Check StorageClass
kubectl get storageclass
```

## Resource Management

### Resource Limits

Agent resource configuration (in DaemonSet):

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "200m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

**etcd StatefulSet resources:**
```yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "250m"
  limits:
    memory: "2Gi"
    cpu: "1000m"
```

**MISP Sync Deployment resources:**
```yaml
resources:
  requests:
    memory: "64Mi"
    cpu: "50m"
  limits:
    memory: "256Mi"
    cpu: "200m"
```

### Node Affinity

Deploy etcd to specific nodes:

```yaml
affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
      - matchExpressions:
        - key: node-role
          operator: In
          values:
          - etcd
```

## Security

### Service Accounts

Create dedicated service account:

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: nnoe-agent
  namespace: nnoe
```

### RBAC

If etcd requires authentication:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: nnoe-agent
  namespace: nnoe
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch"]
```

### Network Policies

Network policies are defined in `deployments/kubernetes/networkpolicy.yaml`.

**Agent Network Policy:**
- **Ingress:**
  - Prometheus scraping from monitoring namespace (port 9090)
  - Health checks from any pod in namespace (port 8080)
- **Egress:**
  - Connection to etcd pods (port 2379)
  - DNS resolution (UDP/TCP 53)
  - HTTP/HTTPS for external services (MISP, Cerbos) (ports 80, 443)
  - All traffic within nnoe namespace

**etcd Network Policy:**
- **Ingress:**
  - Agent pods connecting to etcd (port 2379)
  - etcd peer communication (port 2380)
- **Egress:**
  - All outbound traffic (required for cluster formation)

Apply network policies:
```bash
kubectl apply -f deployments/kubernetes/networkpolicy.yaml
```

## Backup and Recovery

### etcd Backup

```bash
# Create snapshot
kubectl exec -n nnoe etcd-0 -- etcdctl snapshot save /tmp/backup.db

# Copy from pod
kubectl cp nnoe/etcd-0:/tmp/backup.db ./etcd-backup.db
```

### Restore

See `management/etcd-orchestrator/restore.sh` for restore procedures.

## Best Practices

1. **Use StatefulSet** for etcd (maintains stable network identities)
2. **Set resource limits** to prevent resource exhaustion
3. **Use PersistentVolumes** for etcd data
4. **Enable monitoring** for all components
5. **Regular backups** of etcd cluster
6. **Test upgrades** in staging first
7. **Use namespaces** for isolation
8. **Implement network policies** for security

