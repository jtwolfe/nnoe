# Troubleshooting Guide

Common issues and solutions for NNOE deployments.

## Agent Issues

### Agent Won't Start

**Symptoms:**
- Service fails to start
- Error messages in logs

**Solutions:**

1. **Check Configuration:**
   ```bash
   nnoe-agent validate -c /etc/nnoe/agent.toml
   ```

2. **Check etcd Connection:**
   ```bash
   etcdctl --endpoints=http://127.0.0.1:2379 endpoint health
   ```

3. **Check Permissions:**
   ```bash
   ls -la /var/lib/nnoe
   sudo chown -R nnoe:nnoe /var/lib/nnoe /var/log/nnoe
   ```

4. **View Logs:**
   ```bash
   sudo journalctl -u nnoe-agent -n 50
   ```

### Agent Crashes Repeatedly

**Symptoms:**
- Agent restarts continuously
- Systemd shows restart loop

**Solutions:**

1. **Check Resource Limits:**
   ```bash
   free -h
   df -h
   ```

2. **Check for Configuration Errors:**
   ```bash
   nnoe-agent validate -c /etc/nnoe/agent.toml
   ```

3. **Check Service Dependencies:**
   ```bash
   systemctl status etcd
   systemctl status knot
   ```

4. **Enable Debug Logging:**
   ```toml
   [logging]
   level = "debug"
   ```

### Agent Not Receiving Config Changes

**Symptoms:**
- Changes in etcd not reflected
- Services not updating

**Solutions:**

1. **Verify etcd Watch:**
   ```bash
   etcdctl watch /nnoe/dns/zones --prefix
   ```

2. **Check etcd Connection:**
   ```bash
   etcdctl endpoint health
   ```

3. **Restart Agent:**
   ```bash
   sudo systemctl restart nnoe-agent
   ```

4. **Check Cache:**
   ```bash
   ls -la /var/lib/nnoe/cache
   ```

## etcd Issues

### etcd Connection Failed

**Symptoms:**
- "Failed to connect to etcd" errors
- Timeout errors

**Solutions:**

1. **Verify etcd is Running:**
   ```bash
   systemctl status etcd
   ```

2. **Check Network Connectivity:**
   ```bash
   curl http://127.0.0.1:2379/health
   telnet 127.0.0.1 2379
   ```

3. **Check Firewall:**
   ```bash
   sudo ufw status
   sudo firewall-cmd --list-ports
   ```

4. **Verify Endpoints:**
   ```toml
   [etcd]
   endpoints = ["http://127.0.0.1:2379"]
   ```

### etcd Cluster Unhealthy

**Symptoms:**
- "etcd cluster is unavailable"
- Members not responding

**Solutions:**

1. **Check Cluster Status:**
   ```bash
   etcdctl member list
   etcdctl endpoint health
   ```

2. **Verify Quorum:**
   - Need majority of nodes healthy
   - 3-node cluster needs 2 nodes
   - 5-node cluster needs 3 nodes

3. **Restart Failed Nodes:**
   ```bash
   systemctl restart etcd
   ```

4. **Check Node Connectivity:**
   ```bash
   ping etcd-node-1
   telnet etcd-node-1 2380
   ```

## DNS Service Issues

### Knot DNS Not Starting

**Symptoms:**
- DNS queries fail
- Knot service errors

**Solutions:**

1. **Check Knot Status:**
   ```bash
   systemctl status knot
   journalctl -u knot -n 50
   ```

2. **Validate Config:**
   ```bash
   knotc conf-check
   ```

3. **Check Zone Files:**
   ```bash
   ls -la /var/lib/knot/zones
   knotc zone-check example.com
   ```

4. **Check Permissions:**
   ```bash
   ls -la /etc/knot/knot.conf
   chown knot:knot /var/lib/knot
   ```

### DNS Zones Not Updating

**Symptoms:**
- Zone changes not reflected
- Old records still served

**Solutions:**

1. **Verify etcd Zone Data:**
   ```bash
   etcdctl get /nnoe/dns/zones/example.com
   ```

2. **Check Agent Logs:**
   ```bash
   journalctl -u nnoe-agent | grep zone
   ```

3. **Reload Knot:**
   ```bash
   knotc reload
   systemctl reload knot
   ```

4. **Check Zone File:**
   ```bash
   cat /var/lib/knot/zones/example.com.zone
   ```

## DHCP Service Issues

### Kea DHCP Not Starting

**Symptoms:**
- DHCP leases not issued
- Kea service errors

**Solutions:**

1. **Check Kea Status:**
   ```bash
   systemctl status kea-dhcp4
   journalctl -u kea-dhcp4 -n 50
   ```

2. **Validate Config:**
   ```bash
   kea-shell --host localhost --port 8000 config-get
   ```

3. **Check Config File:**
   ```bash
   cat /etc/kea/kea-dhcp4.conf
   kea-config-checker -c /etc/kea/kea-dhcp4.conf
   ```

4. **Check Interfaces:**
   ```bash
   ip addr show
   # Verify interfaces in config match
   ```

### DHCP Leases Not Issued

**Symptoms:**
- Clients can't get IPs
- No lease assignments

**Solutions:**

1. **Check Scope Configuration:**
   ```bash
   etcdctl get /nnoe/dhcp/scopes/scope-1
   ```

2. **Verify Pool Availability:**
   ```bash
   kea-shell --host localhost --port 8000 lease4-get-all
   ```

3. **Check Network Interface:**
   ```bash
   ip link show eth0
   # Ensure interface is up
   ```

4. **Test DHCP Manually:**
   ```bash
   dhclient -v eth0
   ```

## Performance Issues

### High CPU Usage

**Symptoms:**
- Agent using excessive CPU
- System slowdown

**Solutions:**

1. **Check for Loops:**
   ```bash
   top -p $(pgrep nnoe-agent)
   strace -p $(pgrep nnoe-agent)
   ```

2. **Reduce Watch Frequency:**
   - Optimize etcd watch patterns
   - Reduce config update frequency

3. **Check Cache Size:**
   ```toml
   [cache]
   max_size_mb = 50  # Reduce if needed
   ```

4. **Enable Profiling:**
   ```bash
   RUST_LOG=nnoe_agent=debug cargo run
   ```

### High Memory Usage

**Symptoms:**
- Agent memory growing
- OOM kills

**Solutions:**

1. **Check Memory:**
   ```bash
   ps aux | grep nnoe-agent
   ```

2. **Reduce Cache:**
   ```toml
   [cache]
   max_size_mb = 50
   default_ttl_secs = 300
   ```

3. **Check for Leaks:**
   - Use memory profiler
   - Monitor over time

4. **Set Limits:**
   ```systemd
   [Service]
   MemoryLimit=512M
   ```

### Slow Config Propagation

**Symptoms:**
- Changes take long to apply
- Delayed service updates

**Solutions:**

1. **Check etcd Performance:**
   ```bash
   etcdctl endpoint status
   ```

2. **Verify Network Latency:**
   ```bash
   ping etcd-server
   ```

3. **Check Watch Latency:**
   - Monitor etcd watch delays
   - Optimize watch patterns

4. **Reduce Cache TTL:**
   ```toml
   [cache]
   default_ttl_secs = 60  # Reduce for faster updates
   ```

## Network Issues

### Nebula Overlay Not Working

**Symptoms:**
- Nodes can't communicate
- Nebula connection failed

**Solutions:**

1. **Check Nebula Status:**
   ```bash
   systemctl status nebula
   ```

2. **Verify Certificates:**
   ```bash
   ls -la /etc/nebula/*.crt
   nebula-cert verify -ca-crt ca.crt node.crt
   ```

3. **Check Lighthouse:**
   ```bash
   # Verify lighthouse is reachable
   ping lighthouse-ip
   ```

4. **Check Firewall:**
   ```bash
   # Nebula uses UDP 4194
   sudo ufw allow 4194/udp
   ```

### Port Conflicts

**Symptoms:**
- Services can't bind to ports
- "Address already in use" errors

**Solutions:**

1. **Check Port Usage:**
   ```bash
   sudo netstat -tlnp | grep :53
   sudo ss -tlnp | grep :53
   ```

2. **Kill Conflicting Process:**
   ```bash
   sudo fuser -k 53/udp
   ```

3. **Change Ports:**
   ```toml
   # Use different ports if needed
   ```

## Log Analysis

### Enable Debug Logging

```toml
[logging]
level = "debug"
json = false
```

### Common Log Patterns

**Connection Errors:**
```
ERROR Failed to connect to etcd
→ Check etcd is running and accessible
```

**Watch Errors:**
```
ERROR Watch error for prefix
→ Check etcd connectivity and permissions
```

**Service Errors:**
```
ERROR Failed to reload Knot
→ Check Knot service status and config
```

## Getting Help

1. **Check Logs:**
   ```bash
   journalctl -u nnoe-agent -n 100
   ```

2. **Validate Configuration:**
   ```bash
   nnoe-agent validate -c /etc/nnoe/agent.toml
   ```

3. **Collect System Info:**
   ```bash
   nnoe-agent version
   systemctl status nnoe-agent
   etcdctl endpoint health
   ```

4. **Create Issue:**
   - Include error messages
   - System information
   - Configuration (sanitized)
   - Steps to reproduce

