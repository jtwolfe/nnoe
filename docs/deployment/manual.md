# Manual Installation Guide

Step-by-step guide for manual installation of NNOE on bare metal or VMs.

## Prerequisites

- Linux system (Ubuntu 20.04+, Debian 11+, RHEL 8+, or similar)
- Rust 1.70+ installed
- etcd 3.5+ installed and running
- Root or sudo access
- 2GB+ RAM
- 10GB+ disk space

## Installation Steps

### 1. Install Rust

If Rust is not installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### 2. Clone Repository

```bash
git clone https://github.com/nnoe/nnoe.git
cd nnoe
```

### 3. Build Agent

```bash
cargo build --release --package nnoe-agent
```

Binary will be in `target/release/nnoe-agent`.

### 4. Install Binary

```bash
sudo cp target/release/nnoe-agent /usr/local/bin/
sudo chmod +x /usr/local/bin/nnoe-agent
```

Verify:

```bash
nnoe-agent --version
```

### 5. Create System User

```bash
sudo useradd -r -s /bin/false -d /var/lib/nnoe -m nnoe
```

### 6. Create Directories

```bash
sudo mkdir -p /etc/nnoe
sudo mkdir -p /var/lib/nnoe/cache
sudo mkdir -p /var/log/nnoe
sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe
```

### 7. Create Configuration

```bash
sudo cp agent/examples/agent.toml.example /etc/nnoe/agent.toml
sudo chmod 644 /etc/nnoe/agent.toml
```

Edit configuration:

```bash
sudo nano /etc/nnoe/agent.toml
```

Key settings:

```toml
[node]
name = "your-node-name"
role = "active"  # or "management" or "db-only"

[etcd]
endpoints = ["http://127.0.0.1:2379"]  # Your etcd endpoints
prefix = "/nnoe"

[cache]
path = "/var/lib/nnoe/cache"
```

### 8. Install Systemd Service

```bash
sudo cp deployments/manual/systemd/nnoe-agent.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### 9. Enable and Start Service

```bash
sudo systemctl enable nnoe-agent
sudo systemctl start nnoe-agent
```

### 10. Verify Installation

```bash
# Check service status
sudo systemctl status nnoe-agent

# View logs
sudo journalctl -u nnoe-agent -f

# Validate configuration
sudo -u nnoe nnoe-agent validate -c /etc/nnoe/agent.toml
```

## Automated Installation

Use the provided install script:

```bash
cd deployments/manual
sudo ./install.sh
```

This automates steps 2-10 above.

## Service Management

### Start/Stop/Restart

```bash
sudo systemctl start nnoe-agent
sudo systemctl stop nnoe-agent
sudo systemctl restart nnoe-agent
```

### View Logs

```bash
# Follow logs
sudo journalctl -u nnoe-agent -f

# Last 100 lines
sudo journalctl -u nnoe-agent -n 100

# Since boot
sudo journalctl -u nnoe-agent -b
```

### Check Status

```bash
sudo systemctl status nnoe-agent
```

## Configuration

### Configuration File

Location: `/etc/nnoe/agent.toml`

### Node Roles

- **active**: Runs DNS/DHCP services (Knot, Kea, dnsdist)
- **management**: Runs phpIPAM, manages etcd cluster
- **db-only**: Runs etcd follower, no services

### etcd Connection

Ensure etcd is accessible:

```bash
# Test etcd connection
etcdctl --endpoints=http://127.0.0.1:2379 endpoint health

# List keys
etcdctl --endpoints=http://127.0.0.1:2379 get --prefix /nnoe
```

### Service Configuration

Enable/disable services in `agent.toml`:

```toml
[services.dns]
enabled = true
engine = "knot"
config_path = "/etc/knot/knot.conf"
zone_dir = "/var/lib/knot/zones"

[services.dhcp]
enabled = true
engine = "kea"
config_path = "/etc/kea/kea-dhcp4.conf"
```

## Installing Dependencies

### Knot DNS

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install knot
```

**RHEL/CentOS:**
```bash
sudo yum install knot
```

### Kea DHCP

**Ubuntu/Debian:**
```bash
sudo apt-get install kea
```

**RHEL/CentOS:**
```bash
sudo yum install kea
```

### etcd

See etcd documentation for installation:
https://etcd.io/docs/latest/install/

## Troubleshooting

### Agent Won't Start

1. **Check logs:**
   ```bash
   sudo journalctl -u nnoe-agent -n 50
   ```

2. **Validate config:**
   ```bash
   sudo -u nnoe nnoe-agent validate -c /etc/nnoe/agent.toml
   ```

3. **Test etcd connection:**
   ```bash
   etcdctl --endpoints=http://127.0.0.1:2379 endpoint health
   ```

4. **Check permissions:**
   ```bash
   ls -la /var/lib/nnoe
   sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe
   ```

### Permission Errors

```bash
# Fix ownership
sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe

# Fix permissions
sudo chmod 755 /var/lib/nnoe
sudo chmod 644 /etc/nnoe/agent.toml
```

### etcd Connection Failed

1. **Verify etcd is running:**
   ```bash
   sudo systemctl status etcd
   ```

2. **Check etcd endpoints:**
   ```bash
   etcdctl member list
   ```

3. **Test connectivity:**
   ```bash
   curl http://127.0.0.1:2379/health
   ```

4. **Check firewall:**
   ```bash
   sudo ufw status
   sudo firewall-cmd --list-ports  # RHEL/CentOS
   ```

### Service Dependencies

If Knot or Kea fail to start:

```bash
# Check service status
sudo systemctl status knot
sudo systemctl status kea-dhcp4

# Check service logs
sudo journalctl -u knot -n 50
sudo journalctl -u kea-dhcp4 -n 50

# Verify configs exist
ls -la /etc/knot/knot.conf
ls -la /etc/kea/kea-dhcp4.conf
```

## Updating

### Update Agent

1. **Stop service:**
   ```bash
   sudo systemctl stop nnoe-agent
   ```

2. **Pull latest code:**
   ```bash
   cd /path/to/nnoe
   git pull
   ```

3. **Rebuild:**
   ```bash
   cargo build --release --package nnoe-agent
   ```

4. **Install:**
   ```bash
   sudo cp target/release/nnoe-agent /usr/local/bin/
   ```

5. **Start:**
   ```bash
   sudo systemctl start nnoe-agent
   ```

### Update Configuration

1. **Edit config:**
   ```bash
   sudo nano /etc/nnoe/agent.toml
   ```

2. **Validate:**
   ```bash
   sudo -u nnoe nnoe-agent validate -c /etc/nnoe/agent.toml
   ```

3. **Reload:**
   ```bash
   sudo systemctl reload nnoe-agent
   # Or restart
   sudo systemctl restart nnoe-agent
   ```

## Uninstallation

```bash
# Stop and disable service
sudo systemctl stop nnoe-agent
sudo systemctl disable nnoe-agent

# Remove service file
sudo rm /etc/systemd/system/nnoe-agent.service
sudo systemctl daemon-reload

# Remove binary
sudo rm /usr/local/bin/nnoe-agent

# Remove user
sudo userdel nnoe

# Remove directories (optional)
sudo rm -rf /etc/nnoe
sudo rm -rf /var/lib/nnoe
sudo rm -rf /var/log/nnoe
```

## Best Practices

1. **Use dedicated user** (nnoe) for running agent
2. **Set proper permissions** on directories
3. **Enable logging** for troubleshooting
4. **Regular backups** of configuration
5. **Monitor service status** with systemd
6. **Use systemd journal** for logs
7. **Validate config** before starting service
8. **Test etcd connectivity** before deployment

