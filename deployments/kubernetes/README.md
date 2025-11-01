# Kubernetes Deployment

Kubernetes manifests for deploying NNOE.

## Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- kustomize (optional, for kustomization.yaml)

## Quick Start

```bash
# Create namespace and deploy
kubectl apply -k deployments/kubernetes/

# Or deploy manually
kubectl apply -f deployments/kubernetes/management/namespace.yaml
kubectl apply -f deployments/kubernetes/management/etcd-statefulset.yaml
kubectl apply -f deployments/kubernetes/agent/daemonset.yaml
kubectl apply -f deployments/kubernetes/agent/configmap.yaml
```

## Components

### etcd StatefulSet

3-node etcd cluster for high availability.

```bash
kubectl get statefulset -n nnoe etcd
kubectl get pods -n nnoe -l app=etcd
```

### Agent DaemonSet

Deploys agent to all nodes.

```bash
kubectl get daemonset -n nnoe nnoe-agent
kubectl get pods -n nnoe -l app=nnoe-agent
```

### ConfigMap

Agent configuration.

```bash
kubectl get configmap -n nnoe nnoe-agent-config -o yaml
```

## Configuration

Edit `deployments/kubernetes/agent/configmap.yaml` to customize agent configuration.

## Scaling

```bash
# Scale etcd (must be 1, 3, or 5 for quorum)
kubectl scale statefulset etcd -n nnoe --replicas=3

# Agent automatically scales with nodes
```

## Updating

```bash
# Update image
kubectl set image daemonset/nnoe-agent agent=ghcr.io/nnoe/agent:v0.2.0 -n nnoe

# Or edit kustomization.yaml and apply
kubectl apply -k deployments/kubernetes/
```

## Troubleshooting

```bash
# View agent logs
kubectl logs -n nnoe -l app=nnoe-agent -f

# Check etcd status
kubectl exec -n nnoe etcd-0 -- etcdctl endpoint health

# Describe pods
kubectl describe pod -n nnoe -l app=nnoe-agent
```

