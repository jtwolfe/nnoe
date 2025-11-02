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

## Role Mappings Issues

### Role Mappings Not Working

**Symptoms:**
- DNS policies not applying correctly
- Client roles not being extracted
- dnsdist not using role mappings

**Solutions:**

1. **Verify Role Mapping in etcd:**
   ```bash
   etcdctl get /nnoe/role-mappings/192.168.1.10
   # Should return: {"roles": ["iot", "guest"]}
   ```

2. **Check dnsdist Lua Script:**
   ```bash
   cat /var/lib/dnsdist/lua/policy.lua | grep role_map
   # Verify role lookup function is generated
   ```

3. **Check Agent Logs for Role Mapping Updates:**
   ```bash
   journalctl -u nnoe-agent | grep "role-mappings"
   ```

4. **Verify IP/Subnet Format:**
   - Use exact IP (e.g., `192.168.1.10`) or CIDR (e.g., `192.168.1.0/24`)
   - Ensure IP matches client IP seen by dnsdist

5. **Reload dnsdist After Role Mapping Change:**
   ```bash
   systemctl reload dnsdist
   # Or restart: systemctl restart dnsdist
   ```

### DNS Policy Not Matching Roles

**Symptoms:**
- Policies configured but not enforced
- All queries allowed despite policy

**Solutions:**

1. **Check Cerbos Policy:**
   ```bash
   etcdctl get /nnoe/policies/policy-id
   ```

2. **Verify dnsdist Can Query Cerbos:**
   ```bash
   # Test Cerbos connection from agent node
   curl http://cerbos:8222/health
   ```

3. **Check Role Extraction in Lua:**
   ```bash
   # Enable dnsdist logging
   # Check if role is correctly extracted from client IP
   ```

4. **Verify Role Mappings Format:**
   ```json
   {
     "roles": ["role1", "role2"]
   }
   ```

## HA Coordination Issues

### HA Pair Not Coordinating

**Symptoms:**
- Both nodes trying to be Primary
- Kea service running on both nodes
- VIP failover not working

**Solutions:**

1. **Check VIP Status:**
   ```bash
   ip addr show
   # Verify VIP is only on one node
   ```

2. **Check HA Status in etcd:**
   ```bash
   etcdctl get /nnoe/dhcp/ha-pairs/pair-1/nodes/node-1/status
   etcdctl get /nnoe/dhcp/ha-pairs/pair-1/nodes/node-2/status
   ```

3. **Verify HA Pair Configuration:**
   ```toml
   [services.dhcp]
   ha_pair_id = "pair-1"
   peer_node = "node-2"
   ```

4. **Check Agent Logs:**
   ```bash
   journalctl -u nnoe-agent | grep -i "ha\|primary\|standby"
   ```

5. **Test VIP Detection:**
   ```bash
   # Agent uses: ip addr show
   # Manually test: ip addr show | grep <VIP>
   ```

### Kea Service Starting on Standby

**Symptoms:**
- Kea running on Standby node
- DHCP conflicts
- Lease issues

**Solutions:**

1. **Verify HA State:**
   - Check agent logs for HA state determination
   - Verify VIP detection logic

2. **Force Standby State:**
   ```bash
   # Remove VIP from standby node
   ip addr del <VIP>/24 dev eth0
   # Restart agent
   systemctl restart nnoe-agent
   ```

3. **Check etcd Status Updates:**
   ```bash
   # Status should update every 60 seconds
   # Check TTL on status keys
   ```

4. **Verify Keepalived Configuration:**
   - Ensure Keepalived is managing VIP correctly
   - Check Keepalived logs

## IPv6 Lease Issues

### IPv6 Leases Not Syncing to etcd

**Symptoms:**
- IPv4 leases work, IPv6 don't
- No IPv6 lease data in etcd

**Solutions:**

1. **Check Kea IPv6 Configuration:**
   ```bash
   cat /etc/kea/kea-dhcp6.conf
   # Verify hooks-libraries includes libdhcp_etcd.so
   ```

2. **Verify Hook Library Loaded:**
   ```bash
   kea-shell --host localhost --port 8000 config-get | grep hooks
   ```

3. **Check Kea Logs:**
   ```bash
   journalctl -u kea-dhcp6-server | grep -i lease6
   ```

4. **Test IPv6 Lease Manually:**
   ```bash
   # Trigger IPv6 lease from client
   # Check etcd for lease data
   etcdctl get --prefix /nnoe/dhcp/leases/
   ```

5. **Verify Base64 Encoding:**
   - Kea hooks use base64 encoding for etcd v3 API
   - Check hook library compiled with OpenSSL support

### IPv6 Lease Expiration Not Working

**Symptoms:**
- Expired IPv6 leases not removed from etcd
- Stale lease data

**Solutions:**

1. **Check lease6_expire Callout:**
   - Verify hook library includes `lease6_expire` callout
   - Check Kea configuration includes hook

2. **Verify expires_at Field:**
   ```bash
   etcdctl get /nnoe/dhcp/leases/<ipv6-lease-id>
   # Should include "expires_at" timestamp
   ```

3. **Check Kea Lease Expiration Logic:**
   - Kea should call `lease6_expire` when lease expires
   - Verify Kea is processing expired leases

4. **Manual Cleanup:**
   ```bash
   # Manually delete expired leases if needed
   etcdctl del /nnoe/dhcp/leases/<lease-id>
   ```

## Metrics Collection Issues

### Prometheus Exporter Not Collecting Metrics

**Symptoms:**
- No metrics in Prometheus
- Exporter shows disconnected
- Missing metric data

**Solutions:**

1. **Check Exporter Status:**
   ```bash
   systemctl status nnoe-prometheus-exporter
   curl http://localhost:9090/health
   ```

2. **Verify etcd Connection:**
   ```bash
   # Check environment variables
   echo $ETCD_ENDPOINTS
   echo $ETCD_PREFIX
   ```

3. **Test etcd Connection from Exporter:**
   ```bash
   # Exporter connects directly to etcd
   etcdctl --endpoints=$ETCD_ENDPOINTS endpoint health
   ```

4. **Check Metrics Endpoint:**
   ```bash
   curl http://localhost:9090/metrics
   # Should show Prometheus metrics
   ```

5. **Verify Metric Names:**
   - Check for `nnoe_agent_*` metrics
   - Verify etcd_connected is 1.0 (not 0.0)

### Missing DHCP Lease Metrics

**Symptoms:**
- `nnoe_dhcp_leases_active` shows 0
- Lease count not updating

**Solutions:**

1. **Verify etcd Lease Data:**
   ```bash
   etcdctl get --prefix /nnoe/dhcp/leases/ | wc -l
   ```

2. **Check Exporter etcd Prefix:**
   ```bash
   # Exporter uses ETCD_PREFIX environment variable
   # Default: /nnoe
   ```

3. **Check Exporter Logs:**
   ```bash
   journalctl -u nnoe-prometheus-exporter | grep -i etcd
   ```

4. **Verify Lease Path:**
   - Exporter reads from `/nnoe/dhcp/leases` prefix
   - Ensure leases are stored at correct path

### Metrics Not Appearing in Grafana

**Symptoms:**
- Metrics visible in Prometheus but not Grafana
- Dashboard shows "No data"

**Solutions:**

1. **Verify Prometheus Scraping:**
   ```bash
   # Check Prometheus targets
   # ServiceMonitor should be configured
   ```

2. **Check ServiceMonitor:**
   ```bash
   kubectl get servicemonitor -n nnoe nnoe-agent
   # Or verify Prometheus scrape config
   ```

3. **Verify Metric Names in Dashboard:**
   - Dashboard uses exact metric names
   - Check metric names match exporter output

4. **Check Time Range:**
   - Ensure Grafana time range includes data
   - Check Prometheus retention settings

## DB-Only Node Issues

### DB-Only Node Running Services

**Symptoms:**
- DNS/DHCP services running on DB-only node
- Unnecessary resource usage

**Solutions:**

1. **Verify Node Role Configuration:**
   ```bash
   grep "role" /etc/nnoe/agent.toml
   # Should be: role = "db-only"
   ```

2. **Check Agent Logs:**
   ```bash
   journalctl -u nnoe-agent | grep "DB-only"
   # Should show: "DB-only node: Skipping service registration"
   ```

3. **Verify No Services Started:**
   ```bash
   systemctl status knot
   systemctl status kea-dhcp4
   # Should show "not found" or "inactive"
   ```

4. **Check Orchestrator Logic:**
   - Orchestrator should skip `register_services()` for DB-only nodes
   - Only etcd replication and cache should be active

### DB-Only Node Not Maintaining etcd Replication

**Symptoms:**
- etcd quorum lost
- DB-only node not syncing

**Solutions:**

1. **Verify etcd Client Connection:**
   ```bash
   etcdctl endpoint health --endpoints=<db-only-node>:2379
   ```

2. **Check etcd Member Status:**
   ```bash
   etcdctl member list
   # DB-only node should be in member list
   ```

3. **Check Agent Logs:**
   ```bash
   journalctl -u nnoe-agent | grep -i etcd
   ```

4. **Verify etcd Configuration:**
   ```toml
   [etcd]
   endpoints = ["http://etcd-leader:2379"]
   # DB-only node should connect to etcd cluster
   ```

5. **Test etcd Replication:**
   ```bash
   # Write to etcd from leader
   etcdctl put /test/key "value"
   # Read from DB-only node
   etcdctl --endpoints=<db-only-node>:2379 get /test/key
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

