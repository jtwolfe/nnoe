# NNOE Agent API Documentation

## Overview

The NNOE agent is a Rust-based binary that orchestrates DNS, DHCP, and IPAM services. This document describes the agent's command-line interface and internal APIs.

## Command-Line Interface

### Basic Usage

```bash
nnoe-agent [OPTIONS] [COMMAND]
```

### Options

- `-c, --config <PATH>`: Configuration file path (default: `/etc/nnoe/agent.toml`)
- `-d, --debug`: Enable debug logging
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### Commands

#### `run`

Run the agent (default command if none specified).

```bash
nnoe-agent run -c /etc/nnoe/agent.toml
```

#### `validate`

Validate configuration file without running the agent.

```bash
nnoe-agent validate -c /etc/nnoe/agent.toml
```

#### `version`

Show version information.

```bash
nnoe-agent version
```

## Configuration

See `agent/examples/agent.toml.example` for a complete configuration example.

## Internal APIs

### Orchestrator

The `Orchestrator` is the main component that coordinates all agent activities.

- **Location**: `agent/src/core/orchestrator.rs`
- **Key Methods**:
  - `new(config) -> Result<Self>`: Initialize orchestrator with configuration
    - Creates etcd client, cache manager, Nebula manager (if enabled), plugin registry, and metrics
  - `run() -> Result<()>`: Start the main event loop
    - Starts Nebula if enabled
    - Registers service plugins (skipped for DB-only nodes)
    - Starts etcd watch loops
    - Handles shutdown signals
  - `register_services() -> Result<()>`: Register service plugins based on configuration
    - Skips registration if node role is `db-only`
    - Registers: Knot DNS, Kea DHCP, dnsdist, Cerbos, Lynis (if enabled)
    - Injects dependencies (etcd client, node name) into services
  - `watch_config_changes() -> Result<()>`: Start etcd watch loops for all watched prefixes
    - Watches: `/nnoe/dns/zones`, `/nnoe/dhcp/scopes`, `/nnoe/policies`, `/nnoe/threats`, `/nnoe/role-mappings`
    - Each watch spawns a task that notifies plugins via `on_config_change()`
  - `nebula_manager() -> Option<&Arc<NebulaManager>>`: Access to Nebula manager (if enabled)
- **DB-Only Role Handling**: 
  - If `node.role = "db-only"`, orchestrator skips service registration
  - DB-only nodes only maintain etcd replication and cache
  - No DNS/DHCP services are started on DB-only nodes

### Etcd Client

The etcd client provides a wrapper around etcd operations.

- **Location**: `agent/src/etcd/client.rs`
- **Key Methods**:
  - `new(config: &EtcdConfig) -> Result<Self>`: Create etcd client with TLS support
    - Configures TLS if `config.tls` is provided
    - Uses rustls for TLS implementation
    - Supports client certificate authentication
  - `get(key: &str) -> Result<Option<Vec<u8>>>`: Retrieve value by key
  - `put(key: &str, value: &[u8]) -> Result<()>`: Store key-value pair
  - `delete(key: &str) -> Result<()>`: Delete key
  - `list_prefix(prefix: &str) -> Result<Vec<(String, Vec<u8>)>>`: List all keys with prefix
  - `watch(prefix: &str) -> Result<WatchStream>`: Watch for changes to keys with prefix
    - Returns async stream of etcd watch events (Put/Delete)
- **TLS Configuration**:
  - Configure via `EtcdConfig.tls` (optional `TlsConfig`)
  - Required fields: `ca_cert`, `cert`, `key` (file paths)
  - Optional: `verify` (default: true) for certificate verification
  - Uses `rustls::ClientConfig` and `etcd_client::ConnectOptions::with_tls_config`

### Cache Manager

The cache manager provides local caching using sled with TTL and LRU eviction.

- **Location**: `agent/src/sled_cache/cache.rs`
- **Key Methods**:
  - `new(config: &CacheConfig) -> Result<Self>`: Create cache manager
    - Opens or creates sled database at `config.path`
    - Starts background sweep task for TTL and LRU eviction
  - `get(key: &str) -> Result<Option<Vec<u8>>>`: Retrieve cached value
    - Returns `None` if key not found or expired
  - `put(key: &str, value: &[u8]) -> Result<()>`: Cache value with default TTL
  - `put_with_ttl(key: &str, value: &[u8], ttl_secs: u64) -> Result<()>`: Cache value with custom TTL
  - `delete(key: &str) -> Result<()>`: Remove from cache
  - `list_prefix(prefix: &str) -> Result<Vec<(String, Vec<u8>)>>`: List cached keys with prefix
  - `clear() -> Result<()>`: Clear all cached data
  - `get_stats() -> CacheStats`: Get cache statistics
    - Returns: `size_bytes`, `entry_count`, `max_size_bytes`, `ttl_secs`
  - `flush() -> Result<()>`: Flush cache to disk
- **Background Tasks**:
  - Automatic TTL expiration: Sweeps expired entries periodically
  - LRU eviction: Evicts oldest entries when cache size exceeds `max_size_mb`
  - Sweep interval: Configurable (default: every 60 seconds)

### Plugin System

The plugin system allows extensible service integration.

- **Trait**: `agent/src/plugin/trait_def.rs`
- **Registry**: `agent/src/plugin/registry.rs`
- **Service plugins** must implement the `ServicePlugin` trait

### Agent Metrics

The metrics system tracks agent operational statistics using atomic counters.

- **Location**: `agent/src/metrics.rs`
- **Struct**: `AgentMetrics`
- **Fields** (all `Arc<AtomicU64>`):
  - `config_updates_total`: Total number of config updates received from etcd
  - `service_reloads_total`: Total number of service reloads performed
  - `dns_queries_total`: Total DNS queries processed (from dnsdist)
  - `blocked_queries_total`: Total DNS queries blocked (policy/threat)
  - `dhcp_leases_total`: Total DHCP leases allocated
  - `dhcp_leases_active`: Current number of active DHCP leases
- **Methods**:
  - `new() -> Self`: Create new metrics instance with all counters at 0
  - `increment_config_updates()`, `increment_service_reloads()`, etc.: Increment counters
  - `decrement_dhcp_leases_active()`: Decrement active lease count (on release/expire)
  - `get_*()`: Get current counter values (for Prometheus exporter)

## Service-Specific APIs

### KnotService

DNS zone management using Knot DNS.

- **Location**: `agent/src/services/knot.rs`
- **Key Methods**:
  - `new(config: DnsServiceConfig) -> Self`: Create Knot service instance
  - `generate_dnssec_keys(zone_name, zone_domain) -> Result<DnssecKeyInfo>`: Generate DNSSEC keys
    - Creates KSK (Key Signing Key) and ZSK (Zone Signing Key) using `keymgr`
    - Stores keys in Knot key directory
  - `rotate_dnssec_keys(zone_name) -> Result<()>`: Rotate DNSSEC keys (key rollover)
  - `initiate_zone_transfer(zone_name, target) -> Result<()>`: Initiate zone transfer to secondary
    - Uses `knotc zone-transfer` command
  - `apply_dynamic_update(zone_name, updates) -> Result<()>`: Apply RFC 2136 dynamic updates
    - Uses `knotc zone-reload` command
  - `reload_knot() -> Result<()>`: Reload Knot configuration (handles errors gracefully)
  - `restart_knot() -> Result<()>`: Restart Knot service (handles errors gracefully)

### KeaService

DHCP service management using Kea DHCP with HA support.

- **Location**: `agent/src/services/kea.rs`
- **Key Methods**:
  - `new(config: DhcpServiceConfig) -> Self`: Create Kea service instance
  - `set_etcd_client(client: Arc<EtcdClient>)`: Inject etcd client for HA coordination
  - `set_node_name(name: String)`: Set node name for HA status tracking
  - `check_vip() -> Result<bool>`: Check if VIP (Virtual IP) is present on this node
    - Uses `ip addr show` command to detect VIP
    - Returns `true` if VIP is configured, `false` otherwise
  - `check_peer_status(pair_id, peer_node) -> Result<Option<HaState>>`: Query peer HA status from etcd
    - Reads from `/nnoe/dhcp/ha-pairs/{pair_id}/nodes/{peer_node}/status`
    - Returns: `Primary`, `Standby`, `Unknown`, or `None` if not found
  - `update_ha_status_in_etcd() -> Result<()>`: Write this node's HA status to etcd
    - Writes to `/nnoe/dhcp/ha-pairs/{pair_id}/nodes/{node}/status`
    - Includes timestamp and state (`Primary`, `Standby`, `Unknown`)
    - Should be called periodically to maintain TTL
  - `coordinate_ha_pair() -> Result<()>`: Coordinate HA pair state machine
    - Checks VIP presence to determine state
    - Starts Kea if Primary, stops if Standby
    - Updates status in etcd

### DnsdistService

DNS filtering, load balancing, and policy enforcement.

- **Location**: `agent/src/services/dnsdist.rs`
- **Key Methods**:
  - `new(config: DnsdistServiceConfig) -> Self`: Create dnsdist service instance
  - `generate_role_lookup_lua(role_mappings) -> String`: Generate Lua code for role extraction
    - Creates IP/subnet to role mapping table
    - Used by Cerbos policy evaluation in dnsdist
  - `generate_rpz_zone_file() -> Result<()>`: Generate RPZ (Response Policy Zone) zone file
    - Creates standard RPZ zone file for threat domains
    - Stored in `rpz_zone_dir` (default: `/var/lib/dnsdist/rpz`)
    - Can be consumed by downstream DNS servers
  - `cerbos_rule_to_lua(policy) -> Result<String>`: Convert Cerbos policy to dnsdist Lua rule
    - Parses Cerbos expressions using regex
    - Extracts roles from role mappings
    - Generates Lua code for policy enforcement
  - `generate_lua_script() -> Result<()>`: Generate complete dnsdist Lua script
    - Combines role lookup, Cerbos rules, and RPZ blocking
  - `reload_dnsdist() -> Result<()>`: Reload dnsdist configuration

### LynisService

Security auditing with structured report parsing.

- **Location**: `agent/src/services/lynis.rs`
- **Key Methods**:
  - `new(config: LynisServiceConfig, node_id: Option<String>) -> Self`: Create Lynis service
  - `set_etcd_client(client: Arc<EtcdClient>)`: Inject etcd client for report storage
  - `run_audit() -> Result<LynisReport>`: Execute Lynis system audit
    - Runs `lynis audit system --quiet`
    - Parses report file into structured `LynisReport`
  - `parse_lynis_report() -> Result<LynisReport>`: Parse Lynis report file
    - Extracts: score, warnings, suggestions, detailed section items
    - Uses regex for parsing report format
    - Returns structured `LynisReport` with sections
  - `upload_report_to_etcd(report) -> Result<()>`: Upload parsed report to etcd
    - Stores at `/nnoe/audit/lynis/{node_id}`
    - JSON format with all parsed data

### CerbosService

Policy decision point for DNS query authorization.

- **Location**: `agent/src/services/cerbos.rs`
- **Key Methods**:
  - `new(config: CerbosServiceConfig) -> Result<Self>`: Create Cerbos service
    - Connects to Cerbos gRPC endpoint (via tonic)
  - `check_policy(resource, action, principal) -> Result<bool>`: Check policy decision
    - Sends gRPC request to Cerbos `/api.cerbos.dev/CheckResources`
    - Returns `true` if allowed, `false` if denied

## Status

This documentation reflects the current implementation. All APIs documented above are actively used in production code.

