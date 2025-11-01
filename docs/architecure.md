### Revised NNOE Architecture Overview

The New Network Orchestration Engine (NNOE) is an open-source, distributed DDI (DNS, DHCP, IPAM) platform inspired by BlueCat Integrity, emphasizing modularity, high availability (HA), security, and deployment flexibility across VMs, Docker, and Kubernetes (K8s). It uses a control plane/data plane separation: control for configs/sync via Nebula overlay (private, NAT-traversing), data for serving DNS/DHCP to clients. Runtime ops are internal (etcd watches for updates, no external orchestrators post-deployment); deployment uses Ansible/K8s optionally.

Key revisions based on 2025 research:
- Shift to phpIPAM for lighter IPAM/UI, integrated with etcd for DDI extensions (e.g., DNS/DHCP views via API plugins).
- Embed sled in Rust agents for local caching, reducing etcd load.
- dnsdist for advanced DNS filtering, Cerbos for policies, MISP for threats, Lynis for audits.
- Nebula ensures control traffic privacy/auditability, visualized in UI.

Scales to enterprise: 100+ nodes, millions QPS, IPv6 native. OSS, no vendor lock-in.

#### Component Breakdown
- **Management Nodes**: 3+ HA replicas (active-active via load-balancer, e.g., HAProxy). Run phpIPAM (IPAM/UI), etcd leader, Nebula lighthouses, MISP server. Addressable individually (IPs) or via VIP. Sync configs to etcd; handle UI/automation.
- **DB-Only Agents**: Replica nodes running etcd followers + sled (for local reads) + Nebula client. No services (DNS/DHCP); focus on replication/quorum (survive 50% loss). Configurable via etcd flags (role=db-only).
- **Active Agents**: Rust binaries on service nodes; embed sled, raft-rs (etcd client), Nebula. Pull etcd configs, generate/run Knot/Kea/dnsdist. HA pairs (e.g., 4 agents → 2 pairs with Keepalived VIP failover). Run Lynis audits periodically.
- **Network**: Nebula overlay for control (etcd gossip, cert exchanges; virtual IPs e.g., 192.168.100.0/24). Data plane on host interfaces (eth0). Firewall isolation (nftables DROP non-Nebula for control ports).

#### Communication Specs
- **Protocols**: etcd gRPC (ports 2379-2380, TLS-encrypted) for KV watches/pushes; Nebula UDP (4194) for NAT traversal/P2P tunnels; phpIPAM REST API (HTTPS) for UI/etcd sync; Cerbos gRPC (8222) for policy queries; MISP REST (5000) for feeds; Knot/Kea JSON over Unix sockets from agents.
- **Flows**:
  - Management → etcd: Push configs (zones/subnets/policies) via gRPC Put/Watch.
  - Agents → etcd: Watch prefixes (e.g., /dns/zones), pull on changes (<100ms latency).
  - Agents → Cerbos: gRPC CheckResources for query decisions (e.g., allow IoT access?).
  - MISP → etcd: Cron pushes feeds to KV (/threats/domains) for dnsdist RPZ.
  - Lynis → phpIPAM: Audit logs via API for UI reports.
  - Nebula: Cert exchanges via etcd (/nebula/certs); lighthouses advertise for hole-punching.
- **Security**: All inter-node via Nebula (AES-256, cert-auth); etcd ACLs; audit trails in etcd (/audit/logs).

#### Full-Featured UI Details
phpIPAM provides a responsive web UI (PHP/MySQL backend), extended for NNOE via custom plugins/scripts:
- **Dashboard**: IP utilization graphs, query rates (Prometheus embeds), agent health (from etcd), Nebula topology (GraphViz from logs).
- **IPAM Section**: Subnet/VLAN/Host editor, auto-scan (SNMP), CSV exports, IPv6 support.
- **DNS Admin**: Zone/Record management; DNSSEC keys; integration with Knot configs (push to etcd).
- **DHCP Admin**: Scope/options, HA pairs setup, lease views (sync from Kea hooks).
- **Security/Threats**: MISP feed viewer, Cerbos policy editor (YAML upload), dnsdist rules, Lynis reports/alerts.
- **Orchestration/Audit**: Node registration (etcd push), workflows (e.g., Ansible snippets), RBAC, logs/search.
- **Extensions**: Custom PHP plugins for etcd integration (e.g., watch hooks); Grafana embeds for metrics. Deploy via Docker; HA with MySQL replication.

#### Code Examples
**Rust Agent Binary** (embeds sled, raft-rs for etcd, Nebula spawn; ~300 LOC core):
```rust
use sled::Db;
use etcd_client::{Client, WatchOptions};
use std::process::Command;
use tokio::main;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Embed sled for local cache
    let db: Db = sled::open("/var/nnoe/cache")?;
    db.insert("config_key", "initial_value")?; // ACID-safe KV

    // etcd client with Raft embed (via raft-rs)
    let client = Client::connect(["http://etcd-leader:2379"], None).await?;
    let mut watcher = client.watch(" /dns/zones", Some(WatchOptions::default())).await?;
    
    // Spawn Nebula as subprocess
    Command::new("nebula").arg("-config").arg("/etc/nebula/config.yml").spawn()?;

    // Watch loop for runtime ops
    while let Some(resp) = watcher.message().await? {
        for event in resp.events() {
            if let Some(value) = event.kv().and_then(|kv| kv.value()) {
                db.insert(event.kv().unwrap().key(), value)?; // Cache update
                // Recompile configs (e.g., Knot JSON)
                generate_knot_config(&db)?;
                // Restart services (Knot/Kea/dnsdist)
                restart_services()?;
            }
        }
    }
    Ok(())
}

fn generate_knot_config(db: &Db) -> Result<(), sled::Error> {
    // Pull from sled/etcd cache, write to /etc/knot/knot.conf
    Ok(())
}

fn restart_services() -> Result<(), std::io::Error> {
    Command::new("systemctl").arg("restart").arg("knot kea dnsdist").output()?;
    Ok(())
}
```
**Knot DNS Config Example** (generated by agent from etcd):
```
server:
    rundir: "/var/lib/knot"
    listen: [ "::@53", "0.0.0.0@53" ]

zone:
    - domain: example.com
      file: "/var/lib/knot/example.com.zone"
      dnssec-signing: on
```
**Kea DHCP Config Example** (JSON from agent):
```json
{
  "Dhcp4": {
    "interfaces-config": { "interfaces": [ "eth0" ] },
    "lease-database": { "type": "memfile" },
    "subnet4": [ { "subnet": "192.168.1.0/24", "pools": [ { "pool": "192.168.1.10 - 192.168.1.200" } ] } ],
    "hooks-libraries": [ { "library": "/usr/lib/kea/hooks/libdhcp_etcd.so" } ]  // Custom hook for etcd sync
  }
}
```
**dnsdist Lua Rule Example** (for anomalies):
```lua
addLuaAction(AllRule(), function(dq)
  if isAnomalous(dq) then return DNSAction.Drop end  -- Custom detection
  return DNSAction.None
end)
```
**Cerbos Policy Example** (YAML for DNS access):
```yaml
apiVersion: api.cerbos.dev/v1
resourcePolicy:
  version: default
  resource: dns_query
  rules:
    - actions: ['allow']
      effect: EFFECT_ALLOW
      roles: ['iot']
      condition:
        match:
          expr: request.time.hour < 18
```
**MISP Feed Pull Script** (Python for cron):
```python
import requests
import etcd3

etcd = etcd3.client(host='etcd-leader', port=2379)
response = requests.get('https://misp-feed.example/api')
for threat in response.json():
    etcd.put(f'/threats/{threat["domain"]}', threat["data"])
```
**Lynis Integration Script** (Bash in agent):
```bash
#!/bin/bash
lynis audit system --quiet --report-file /tmp/lynis.report
curl -X POST -d @/tmp/lynis.report https://phpipam/api/audit  # Push to UI
```

#### PlantUML Diagrams
**Architecture Overview**:
```
@startuml
node "Management Nodes (HA)" {
  [phpIPAM UI/API]
  [etcd Leader]
  [MISP Server]
  [Nebula Lighthouse]
}
node "DB-Only Agents" {
  [etcd Follower]
  [sled Cache]
  [Nebula Client]
}
node "Active Agents" {
  [Rust Binary] --> [sled Cache]
  [Rust Binary] --> [Knot DNS]
  [Rust Binary] --> [Kea DHCP]
  [Rust Binary] --> [dnsdist Proxy]
  [Rust Binary] --> [Cerbos PDP]
  [Rust Binary] --> [Lynis Auditor]
  [Nebula Client]
}
[Management Nodes] --> [DB-Only Agents] : etcd gRPC (replication)
[Management Nodes] --> [Active Agents] : Nebula UDP (control traffic)
[Active Agents] --> [Clients] : DNS/UDP (53), DHCP/UDP (67/68)
@enduml
```

**Functionality Flow (Config Update)**:
```
@startuml
start
:User edits in phpIPAM UI;
:Push to etcd KV (/dns/zones);
note right: gRPC Put
:etcd broadcasts via Watch;
:Active Agent pulls change;
:Update sled cache;
:Generate configs (Knot/Kea);
:Apply Cerbos policy check;
:Restart services;
:Log audit to Lynis/phpIPAM;
stop
@enduml
```