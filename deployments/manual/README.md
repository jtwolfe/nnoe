# Manual Installation

Manual installation guide for NNOE on bare metal or VMs.

## Prerequisites

- Rust 1.82+ or stable (`cargo --version`)
- etcd installed and running
- Systemd (for service management)

## Quick Install

```bash
cd deployments/manual
sudo ./install.sh
```

This will:
1. Build the agent from source
2. Install to `/usr/local/bin/nnoe-agent`
3. Create configuration in `/etc/nnoe/agent.toml`
4. Create systemd service
5. Set up directories and permissions

## Manual Steps

### 1. Build Agent

```bash
cd /path/to/nnoe
cargo build --release --package nnoe-agent
```

### 2. Install Binary

```bash
sudo cp target/release/nnoe-agent /usr/local/bin/
sudo chmod +x /usr/local/bin/nnoe-agent
```

### 3. Create User

```bash
sudo useradd -r -s /bin/false -d /var/lib/nnoe -m nnoe
```

### 4. Create Directories

```bash
sudo mkdir -p /etc/nnoe /var/lib/nnoe/cache /var/log/nnoe
sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe
```

### 5. Configure

```bash
sudo cp agent/examples/agent.toml.example /etc/nnoe/agent.toml
sudo nano /etc/nnoe/agent.toml
```

Edit with your etcd endpoints and service configurations.

### 6. Install Systemd Service

```bash
sudo cp deployments/manual/systemd/nnoe-agent.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable nnoe-agent
sudo systemctl start nnoe-agent
```

### 7. Verify

```bash
sudo systemctl status nnoe-agent
sudo journalctl -u nnoe-agent -f
```

## Service Management

```bash
# Start
sudo systemctl start nnoe-agent

# Stop
sudo systemctl stop nnoe-agent

# Restart
sudo systemctl restart nnoe-agent

# View logs
sudo journalctl -u nnoe-agent -f

# Validate config
nnoe-agent validate -c /etc/nnoe/agent.toml
```

## Troubleshooting

### Agent won't start

```bash
# Check logs
sudo journalctl -u nnoe-agent -n 50

# Validate config
sudo -u nnoe nnoe-agent validate -c /etc/nnoe/agent.toml

# Test etcd connection
etcdctl --endpoints=http://127.0.0.1:2379 endpoint health
```

### Permission issues

```bash
# Fix ownership
sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe
```

