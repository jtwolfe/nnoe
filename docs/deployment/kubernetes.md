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

### etcd StatefulSet

3-node etcd cluster for high availability.

```bash
# Check etcd cluster
kubectl get statefulset -n nnoe etcd
kubectl get pods -n nnoe -l app=etcd

# Check etcd health
kubectl exec -n nnoe etcd-0 -- etcdctl endpoint health
```

### Agent DaemonSet

Deploys agent to all cluster nodes.

```bash
# Check agent deployment
kubectl get daemonset -n nnoe nnoe-agent

# View agent pods
kubectl get pods -n nnoe -l app=nnoe-agent
```

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

### Environment Variables

Override via DaemonSet:

```yaml
env:
- name: ETCD_ENDPOINTS
  value: "http://etcd.nnoe.svc.cluster.local:2379"
- name: NODE_NAME
  valueFrom:
    fieldRef:
      fieldPath: spec.nodeName
```

## Scaling

### etcd Cluster

etcd must have odd number of nodes (1, 3, or 5) for quorum.

```bash
# Scale to 5 nodes (requires 5 nodes in cluster)
kubectl scale statefulset etcd -n nnoe --replicas=5
```

### Agent

Agent automatically scales with cluster nodes via DaemonSet.

To limit agent to specific nodes:

```yaml
nodeSelector:
  nnoe-agent: "enabled"
tolerations:
- key: nnoe-agent
  operator: Exists
  effect: NoSchedule
```

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

etcd is accessible via:

```
http://etcd.nnoe.svc.cluster.local:2379
```

### DNS

Services can resolve via Kubernetes DNS:

- `etcd.nnoe.svc.cluster.local`
- `etcd-0.etcd.nnoe.svc.cluster.local`

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

```bash
# Check pod health
kubectl get pods -n nnoe

# Describe pod
kubectl describe pod -n nnoe nnoe-agent-<node-name>
```

### Metrics

Expose Prometheus metrics:

```yaml
ports:
- name: metrics
  containerPort: 9090
```

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

Set appropriate limits:

```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
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

Restrict network access:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: nnoe-agent-policy
  namespace: nnoe
spec:
  podSelector:
    matchLabels:
      app: nnoe-agent
  policyTypes:
  - Ingress
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: etcd
    ports:
    - protocol: TCP
      port: 2379
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

