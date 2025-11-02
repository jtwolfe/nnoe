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
- **DB-Only Agents**: Replica nodes running etcd followers + sled (for local reads) + Nebula client. No services (DNS/DHCP); focus on replication/quorum (survive 50% loss). Configure via `node.role = "db-only"` in agent.toml. Orchestrator skips service registration on DB-only nodes, maintaining only etcd replication and cache.
- **Active Agents**: Rust binaries on service nodes; embed sled (with TTL/eviction), etcd client (TLS support via rustls), Nebula (with process monitoring/auto-restart). Pull etcd configs, generate/run Knot/Kea/dnsdist. HA pairs (e.g., 4 agents → 2 pairs with Keepalived VIP failover, state coordination via etcd). Run Lynis audits periodically. Support configurable ports/addresses for all services. Track metrics (config updates, service reloads, DNS queries, DHCP leases) via AgentMetrics.
- **Network**: Nebula overlay for control (etcd gossip, cert exchanges; virtual IPs e.g., 192.168.100.0/24). Data plane on host interfaces (eth0). Firewall isolation (nftables DROP non-Nebula for control ports).

#### Communication Specs
- **Protocols**: 
  - etcd gRPC (ports 2379-2380, TLS-encrypted with rustls) for KV watches/pushes
  - Nebula UDP (4194) for NAT traversal/P2P tunnels
  - phpIPAM REST API (HTTPS) for UI/etcd sync
  - Cerbos gRPC (8222, via tonic) for policy queries
  - MISP REST (5000) for feeds
  - Knot/Kea JSON over Unix sockets from agents
  - Prometheus HTTP (9090) for metrics export
- **Flows**:
  - Management → etcd: Push configs (zones/subnets/policies/role-mappings) via gRPC Put/Watch (TLS-encrypted).
  - Agents → etcd: Watch prefixes (/dns/zones, /dhcp/scopes, /policies, /threats, /role-mappings), pull on changes (<100ms latency). TLS client cert auth.
  - Agents → Cerbos: gRPC CheckResources (tonic client) for query decisions (e.g., allow IoT access?). Policies converted to dnsdist Lua rules. Role extraction from IP/subnet mappings stored in etcd.
  - MISP → etcd: Sync service supports multiple MISP instances (MISP_URL, MISP_URL_2, etc.), tag filtering (MISP_FILTER_TAGS), and deduplication. Pushes feeds to KV (/threats/domains) for dnsdist RPZ, with retry/backoff.
  - Kea Hooks → etcd: Lease events (offer/renew/release/expire) via libdhcp_etcd.so hook. Supports IPv4 (lease4_*) and IPv6 (lease6_*) callouts. Lease data includes `expires_at` timestamp. Keys/values base64 encoded for etcd v3 API.
  - HA Coordination: Kea HA pairs coordinate via etcd (/dhcp/ha-pairs/{pair_id}/nodes/{node}/status). VIP detection via `ip addr show` command. State machine: Primary (VIP present) / Standby (VIP absent). Status updates include timestamp, TTL of 60 seconds.
  - Role Mappings: IP/subnet to role mappings stored at /role-mappings/{ip_or_subnet}. Used by dnsdist for client role extraction in Cerbos policy evaluation.
  - Lynis → etcd: Audit reports parsed into structured JSON (score, warnings, suggestions, sections). Pushed to /audit/lynis/{node} with periodic scheduling.
  - Monitoring: Prometheus exporter connects directly to etcd for metrics collection. Exposes metrics on port 9090 (config_updates_total, service_reloads_total, dns_queries_total, blocked_queries_total, dhcp_leases_total, dhcp_leases_active, etcd_connected, ha_state, cache_size_bytes, cache_entries, dns_query_rate, uptime_seconds). Grafana dashboards for visualization.
  - Nebula: Cert exchanges via etcd (/nebula/certs); lighthouses advertise for hole-punching. Auto-restart on failure with exponential backoff. Certificate rotation and expiry checking via management scripts.
  - Metrics: AgentMetrics tracks operational statistics using atomic counters. Incremented on config updates, service reloads, DNS queries, DHCP leases. Exposed via Prometheus exporter HTTP endpoint.
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
**Rust Agent Binary** (embeds sled with TTL/eviction, etcd client with TLS, Nebula with monitoring; actual implementation uses plugin architecture):
```rust
use nnoe_agent::core::Orchestrator;
use nnoe_agent::config::AgentConfig;
use nnoe_agent::sled_cache::CacheManager;
use nnoe_agent::etcd::EtcdClient;
use nnoe_agent::nebula::NebulaManager;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AgentConfig::load("/etc/nnoe/agent.toml")?;
    
    // Initialize orchestrator (handles etcd client, cache manager, Nebula)
    let orchestrator = Orchestrator::new(config).await?;
    
    // Start Nebula if enabled (with automatic restart on failure)
    if let Some(ref nebula_manager) = orchestrator.nebula_manager() {
        nebula_manager.start().await?;
    }
    
    // Start watching etcd for configuration changes
    orchestrator.watch_config_changes().await?;
    
    // Run orchestrator (registers services, watches etcd)
    orchestrator.run().await?;
    
    Ok(())
}

// Actual orchestrator watch implementation (simplified):
// The orchestrator spawns separate tasks for each watch prefix:
// - /nnoe/dns/zones
// - /nnoe/dhcp/scopes  
// - /nnoe/policies
// - /nnoe/threats
// - /nnoe/role-mappings
//
// Each watch task notifies registered plugins via on_config_change()
// when etcd events (Put/Delete) are received.
//
// DB-Only Role Handling:
// If node.role = "db-only", orchestrator skips register_services()
// and only maintains etcd replication and cache, improving quorum resilience.
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
**Kea DHCP Config Example** (JSON from agent, with IPv4 and IPv6):
```json
{
  "Dhcp4": {
    "interfaces-config": { "interfaces": [ "eth0" ] },
    "lease-database": { "type": "memfile" },
    "subnet4": [ { "subnet": "192.168.1.0/24", "pools": [ { "pool": "192.168.1.10 - 192.168.1.200" } ] } ],
    "hooks-libraries": [ { "library": "/usr/lib/kea/hooks/libdhcp_etcd.so" } ]
  },
  "Dhcp6": {
    "interfaces-config": { "interfaces": [ "eth0" ] },
    "lease-database": { "type": "memfile" },
    "subnet6": [ { "subnet": "2001:db8::/64", "pools": [ { "pool": "2001:db8::100 - 2001:db8::200" } ] } ],
    "hooks-libraries": [ { "library": "/usr/lib/kea/hooks/libdhcp_etcd.so" } ]
  }
}
// Kea hooks sync IPv4 (lease4_offer, lease4_renew, lease4_release, lease4_expire)
// and IPv6 (lease6_offer, lease6_renew, lease6_release, lease6_expire) leases to etcd
// with expires_at timestamps and IPv6-specific fields (type, iaid, duid, preferred_lft)
```
**dnsdist Lua Rule Example** (with role extraction and Cerbos policy):
```lua
-- Role lookup based on client IP from role-mappings in etcd
local client_ip = dq.remoteaddr:toString()
local role = "user"  -- Default role
local role_map = {
  ["192.168.1.10"] = "iot",
  ["192.168.1.0/24"] = "guest",
}
if role_map[client_ip] then
  role = role_map[client_ip]
end

-- Cerbos policy evaluation (converted from etcd policy)
addLuaAction(AllRule(), function(dq)
  if checkCerbosPolicy(role, dq.qname:toString()) then
    return DNSAction.Allow
  else
    -- Block or redirect based on policy
    return DNSAction.Drop
  end
end)

-- RPZ blocking for threat domains (generated from /nnoe/threats/domains)
-- RPZ zone file generated at /var/lib/dnsdist/rpz/threats.rpz
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
**MISP Sync Service** (Rust binary with multiple instance support):
```rust
// Supports multiple MISP instances via environment variables:
// MISP_URL, MISP_API_KEY (primary instance)
// MISP_URL_2, MISP_API_KEY_2 (secondary instance)
// MISP_FILTER_TAGS (comma-separated tags for filtering)
// MISP_DEDUP (enable deduplication)

// Syncs events from all enabled instances, applies tag filtering,
// performs deduplication using HashSet, and pushes to etcd at
// /nnoe/threats/domains/{domain} with source, severity, timestamp.
```
**Lynis Integration** (Rust service with structured parsing):
```rust
// LynisService runs periodic audits, parses report file using regex,
// extracts structured data (score, warnings, suggestions, sections),
// and uploads JSON to etcd at /nnoe/audit/lynis/{node_id}
// Report includes detailed section items for comprehensive security analysis.
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
:Push to etcd KV (/dns/zones or /role-mappings);
note right: gRPC Put (TLS-encrypted)
:etcd broadcasts via Watch;
:Active Agent pulls change;
note right: Metrics: increment_config_updates_total()
:Update sled cache (with TTL);
:Generate configs (Knot/Kea/dnsdist);
:Apply Cerbos policy check (with role extraction);
:Reload/Restart services;
note right: Metrics: increment_service_reloads_total()
:Log audit to Lynis/phpIPAM;
stop
@enduml
```

**HA Coordination Flow (Kea Failover)**:
```
@startuml
start
:Agent checks VIP via "ip addr show";
alt VIP present
  :State = Primary;
  :Update etcd: /dhcp/ha-pairs/{id}/nodes/{node}/status = Primary;
  :Start Kea service;
else VIP absent
  :State = Standby;
  :Update etcd: /dhcp/ha-pairs/{id}/nodes/{node}/status = Standby;
  :Stop Kea service;
end
:Periodically refresh status (TTL 60s);
stop
@enduml
```
