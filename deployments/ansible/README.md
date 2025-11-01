# Ansible Deployment

Ansible playbooks and roles for automated NNOE deployment.

## Prerequisites

- Ansible 2.9+
- SSH access to target hosts
- Python 3 on target hosts

## Inventory

Edit `inventory/example.yml` with your hosts:

```yaml
all:
  children:
    management:
      hosts:
        mgmt-1:
          ansible_host: 192.168.1.10
    agents:
      hosts:
        agent-1:
          ansible_host: 192.168.1.20
```

## Deploy

```bash
# Deploy to all hosts
ansible-playbook -i inventory/example.yml playbooks/deploy.yml

# Deploy to specific group
ansible-playbook -i inventory/example.yml playbooks/deploy.yml -l agents

# Deploy with custom vars
ansible-playbook -i inventory/example.yml playbooks/deploy.yml \
  -e etcd_endpoints='["http://192.168.1.10:2379"]'
```

## Roles

### common

Creates users, directories, installs dependencies.

### etcd

Installs and configures etcd cluster.

### agent

Builds and installs NNOE agent.

### services

Installs Knot DNS and Kea DHCP.

## Variables

Key variables:

- `nnoe_version`: Version to deploy
- `etcd_endpoints`: etcd cluster endpoints
- `etcd_prefix`: etcd key prefix
- `nnoe_node_role`: Node role (management/active/db-only)
- `dns_enabled`: Enable DNS service
- `dhcp_enabled`: Enable DHCP service

## Testing

```bash
# Test connection
ansible all -i inventory/example.yml -m ping

# Dry run
ansible-playbook -i inventory/example.yml playbooks/deploy.yml --check
```

