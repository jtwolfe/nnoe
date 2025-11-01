# Ansible Deployment Guide

Automated deployment of NNOE using Ansible playbooks.

## Prerequisites

- Ansible 2.9+
- Python 3 on control machine
- SSH access to target hosts
- Python 3 on target hosts
- sudo/root access on target hosts

## Inventory Setup

Create inventory file:

```yaml
all:
  children:
    management:
      hosts:
        mgmt-1:
          ansible_host: 192.168.1.10
          nnoe_node_role: management
          etcd_initial_cluster: "mgmt-1=http://192.168.1.10:2380"
          etcd_cluster_state: new
        mgmt-2:
          ansible_host: 192.168.1.11
          nnoe_node_role: management
          etcd_initial_cluster: "mgmt-1=http://192.168.1.10:2380,mgmt-2=http://192.168.1.11:2380"
          etcd_cluster_state: existing
    
    agents:
      hosts:
        agent-1:
          ansible_host: 192.168.1.20
          nnoe_node_role: active
          etcd_endpoints: ["http://192.168.1.10:2379", "http://192.168.1.11:2379"]

  vars:
    ansible_user: root
    ansible_python_interpreter: /usr/bin/python3
    etcd_prefix: "/nnoe"
```

## Basic Deployment

### Test Connection

```bash
ansible all -i inventory/example.yml -m ping
```

### Deploy to All Hosts

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml
```

### Deploy to Specific Group

```bash
# Deploy only management nodes
ansible-playbook -i inventory/example.yml playbooks/deploy.yml -l management

# Deploy only agents
ansible-playbook -i inventory/example.yml playbooks/deploy.yml -l agents
```

## Role Configuration

### Common Role

Sets up users, directories, and dependencies.

**Variables:**
- `nnoe_user`: User for running agent (default: `nnoe`)
- `nnoe_group`: Group for agent (default: `nnoe`)
- `nnoe_home`: Home directory (default: `/var/lib/nnoe`)

### etcd Role

Installs and configures etcd cluster.

**Variables:**
- `etcd_version`: etcd version (default: `3.5.9`)
- `etcd_data_dir`: Data directory (default: `/var/lib/etcd`)
- `etcd_initial_cluster`: Initial cluster configuration
- `etcd_cluster_token`: Cluster token (default: `nnoe-cluster`)
- `etcd_cluster_state`: Cluster state (`new` or `existing`)

**Example:**
```yaml
etcd_initial_cluster: "mgmt-1=http://192.168.1.10:2380,mgmt-2=http://192.168.1.11:2380"
etcd_cluster_state: existing
```

### Agent Role

Builds and installs NNOE agent.

**Variables:**
- `nnoe_version`: Agent version to deploy
- `nnoe_build`: Build from source (default: `true`)
- `etcd_endpoints`: List of etcd endpoints
- `etcd_prefix`: etcd key prefix (default: `/nnoe`)
- `nnoe_node_role`: Node role (`management`, `active`, `db-only`)

**Example:**
```yaml
etcd_endpoints: ["http://192.168.1.10:2379", "http://192.168.1.11:2379"]
nnoe_node_role: active
```

### Services Role

Installs Knot DNS and Kea DHCP.

**Variables:**
- `dns_enabled`: Enable DNS service (default: `true`)
- `dhcp_enabled`: Enable DHCP service (default: `true`)

## Custom Variables

Override variables via command line:

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -e etcd_prefix="/custom/prefix" \
  -e dns_enabled=false
```

Or use group/host vars:

```yaml
# group_vars/all.yml
nnoe_version: "0.2.0"
etcd_prefix: "/nnoe"
```

## Deployment Workflow

### 1. Deploy Management Nodes

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -l management \
  -e etcd_cluster_state=new
```

### 2. Deploy Additional Management Nodes

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -l management \
  -e etcd_cluster_state=existing
```

### 3. Deploy Agents

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -l agents
```

## Verification

### Check Service Status

```bash
ansible all -i inventory/example.yml -m shell \
  -a "systemctl status nnoe-agent"
```

### Validate Configuration

```bash
ansible agents -i inventory/example.yml -m shell \
  -a "nnoe-agent validate -c /etc/nnoe/agent.toml"
```

### Check etcd Cluster

```bash
ansible management -i inventory/example.yml -m shell \
  -a "etcdctl endpoint health"
```

## Updating

### Update Agent

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -e nnoe_version="0.2.0" \
  -l agents
```

### Update Configuration

Edit `roles/agent/templates/agent.toml.j2` and redeploy:

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -l agents
```

## Troubleshooting

### Connection Issues

```bash
# Test SSH connection
ansible all -i inventory/example.yml -m ping

# Check Python availability
ansible all -i inventory/example.yml -m raw \
  -a "python3 --version"
```

### Playbook Failures

```bash
# Run with verbose output
ansible-playbook -i inventory/example.yml playbooks/deploy.yml -vvv

# Check specific task
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  --start-at-task="Install NNOE agent binary"
```

### Service Issues

```bash
# Check service status
ansible agents -i inventory/example.yml -m shell \
  -a "systemctl status nnoe-agent"

# View logs
ansible agents -i inventory/example.yml -m shell \
  -a "journalctl -u nnoe-agent -n 50"
```

## Dry Run

Test changes without applying:

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml --check
```

## Tags

Run specific role tasks:

```bash
# Only etcd tasks
ansible-playbook -i inventory/example.yml playbooks/deploy.yml --tags etcd

# Skip service installation
ansible-playbook -i inventory/example.yml playbooks/deploy.yml --skip-tags services
```

## Best Practices

1. **Use version control** for inventory and playbooks
2. **Test in staging** before production
3. **Use vault** for secrets:
   ```bash
   ansible-vault encrypt inventory/production.yml
   ```
4. **Use tags** for selective execution
5. **Verify after deployment** using verification tasks
6. **Keep inventory organized** by environment
7. **Document custom variables** in README

## Advanced Usage

### Parallel Execution

```bash
ansible-playbook -i inventory/example.yml playbooks/deploy.yml -f 10
```

### Custom Facts

Create `facts.d/nnoe.fact` on hosts:

```ini
[nnoe]
node_id=unique-node-id
datacenter=dc1
```

### Dynamic Inventory

Use cloud provider inventory scripts:

```bash
ansible-playbook -i aws_ec2.yml playbooks/deploy.yml
```

