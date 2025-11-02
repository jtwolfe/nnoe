# NNOE Issues Resolution Plan

## Priority 1: Critical Security & Functionality (Weeks 1-2)

### 1.1 Implement TLS Support for etcd Client

**Files**: `agent/src/etcd/client.rs`, `agent/Cargo.toml`

- Add `rustls` or `native-tls` dependency to workspace Cargo.toml
- Update `EtcdClient::new()` in `agent/src/etcd/client.rs` (lines 17-34):
- Use `etcd_client::ConnectOptions` to configure TLS
- Load CA cert, client cert, and key from `TlsConfig` paths
- Replace placeholder warning with actual TLS initialization
- Test TLS connection with self-signed certificates
- Update `agent/examples/agent.toml.example` with TLS config examples

### 1.2 Implement Cerbos gRPC Client

**Files**: `agent/src/services/cerbos.rs`, `agent/Cargo.toml`

- Add `tonic` and `tonic-build` dependencies for gRPC
- Generate Cerbos proto files or use `cerbos-sdk` crate if available
- Fix `CerbosService` struct: replace `etcd_client::Client` with `tonic::Channel` or Cerbos client
- Implement `check_policy()` method (lines 25-38):
- Create gRPC request to Cerbos `/api.cerbos.dev/CheckResources`
- Parse Cerbos response
- Return actual policy decision
- Add error handling and retry logic for gRPC calls
- Add connection pooling and timeout handling

### 1.3 Fix Lynis Periodic Audit Execution

**Files**: `agent/src/services/lynis.rs`, `agent/src/core/orchestrator.rs`

- In `LynisService::init()` (lines 136-167), spawn periodic audit task:
- Get etcd client from orchestrator context
- Call `self.start_periodic_audits(Some(etcd_client))` in background task
- Update orchestrator to pass etcd client to Lynis service during initialization
- Ensure audit task runs independently and doesn't block service startup
- Add graceful shutdown handling for audit task

### 1.4 Add CI/CD Pipeline

**Files**: `.github/workflows/test.yml`, `.github/workflows/build.yml`, `.github/workflows/e2e.yml`, `.github/workflows/release.yml`

- Create `.github/workflows/test.yml`:
- Rust toolchain setup
- Run `cargo test --all`
- Run `cargo clippy --all-targets -- -D warnings`
- Run `cargo fmt --check`
- Create `.github/workflows/build.yml`:
- Multi-arch builds (linux/amd64, linux/arm64)
- Build agent and misp-sync binaries
- Upload artifacts
- Create `.github/workflows/e2e.yml`:
- Docker Compose test environment
- Full stack integration tests
- Run `testing/integration/e2e/e2e_tests.rs`
- Create `.github/workflows/release.yml`:
- Tag-based releases
- Build and publish Docker images
- Create GitHub releases

### 1.5 Add PHP Dependencies for phpIPAM Plugin

**Files**: `integrations/phpipam-plugin/composer.json`

- Create `composer.json` with:
- `guzzlehttp/guzzle` dependency
- Autoload configuration for `src/` directory
- Minimum PHP version requirement
- Add `composer.lock` to `.gitignore` or commit it
- Update plugin README with installation instructions

## Priority 2: High Priority Features (Weeks 3-5)

### 2.1 Implement Cache TTL and Eviction

**Files**: `agent/src/sled_cache/cache.rs`, `agent/src/config.rs`

- Add timestamp metadata to cache entries (key + `_timestamp` suffix)
- Implement TTL check in `get()` method:
- Read timestamp from metadata
- Compare with current time against `default_ttl_secs`
- Delete expired entries
- Add background task in `CacheManager::new()`:
- Periodic sweep for expired entries (every 60 seconds)
- Remove entries older than TTL
- Track cache size and enforce `max_size_mb` limit
- Implement LRU eviction when size limit exceeded

### 2.2 Implement Kea Hooks (C++)

**Files**: `integrations/kea-hooks/src/libdhcp_etcd.cpp`, `integrations/kea-hooks/CMakeLists.txt`, `integrations/kea-hooks/README.md`

- Create CMake build system for Kea hook
- Implement `libdhcp_etcd.cpp`:
- Implement Kea hook API callouts (`lease4_offer`, `lease4_renew`, `lease4_release`)
- etcd client integration (use etcd-cpp or HTTP client)
- Error handling and retry logic
- Configuration loading from Kea hook parameters
- Add build instructions and Dockerfile for hook compilation
- Update `agent/src/services/kea.rs` (line 136) to reference actual hook library

### 2.3 Create Monitoring Infrastructure

**Files**: `management/monitoring/prometheus-exporter/src/main.rs`, `management/monitoring/grafana/dashboards/nnoe-dashboard.json`

- Create Prometheus metrics exporter:
- HTTP endpoint on port 9090
- Metrics: agent uptime, etcd connections, cache size, config updates, service reloads
- Service-specific metrics (DNS query rate, DHCP leases, etc.)
- Create Grafana dashboard JSON:
- Agent status panels
- Service health panels
- Performance metrics (latency, QPS)
- Resource usage (CPU, memory, disk)
- Update orchestrator to expose metrics endpoint
- Add metrics collection in service plugins

### 2.4 Implement DB-Only Agent Role Logic

**Files**: `agent/src/core/orchestrator.rs`, `agent/src/config.rs`

- In `Orchestrator::register_services()`, check `config.node.role`:
- If `NodeRole::DbOnly`, skip service registration (DNS/DHCP/dnsdist)
- Only initialize etcd client and cache manager
- Log role-specific startup message
- Ensure DB-only agents don't attempt to generate service configs
- Update deployment docs to explain DB-only node configuration

### 2.5 Add Comprehensive Error Handling

**Files**: `integrations/misp-sync/src/main.rs`, `agent/src/services/*.rs`

- Implement retry logic with exponential backoff:
- Create `RetryConfig` struct with max retries, initial delay, max delay
- Add `retry_with_backoff()` utility function
- Apply to all network operations (MISP API, etcd operations)
- Add circuit breaker pattern for external services
- Improve error messages with context:
- Include endpoint URLs in TLS errors
- Include service names in health check failures
- Add structured error types using `thiserror` crate

## Priority 3: Medium Priority Improvements (Weeks 6-8)

### 3.1 Implement Cerbos Policy to dnsdist Rule Conversion

**Files**: `agent/src/services/dnsdist.rs`, `agent/src/services/cerbos.rs`

- Create policy parser in `DnsdistService::on_config_change()` (lines 232-236):
- Parse Cerbos policy YAML from etcd
- Extract DNS-related rules (resource: `dns_query`, actions: `allow`, `deny`)
- Convert conditions to Lua expressions
- Generate Lua rules:
- Time-based conditions → Lua time checks
- Principal/role checks → Lua variable lookups
- Apply rules via `addLuaAction()` in generated script
- Test with example Cerbos policies from `integrations/cerbos-policies/`

### 3.2 Complete Nebula Process Monitoring

**Files**: `agent/src/nebula/manager.rs`

- Fix `is_running()` method (line 91):
- Use atomic boolean flag set on process start/stop
- Check process PID file existence
- Send signal 0 to verify process responsiveness
- Fix Drop trait issue (lines 124-130):
- Use `tokio::runtime::Handle::block_on()` for async cleanup
- Or store `tokio::task::JoinHandle` and cancel on drop
- Implement automatic restart on failure:
- Track restart count and max retries
- Exponential backoff between restarts
- Alert if restart fails repeatedly

### 3.3 Implement Kea HA Coordination

**Files**: `agent/src/services/kea.rs`, `agent/src/config.rs`

- Add Keepalived integration:
- Detect VIP assignment via system calls
- Monitor peer status via etcd (`/nnoe/dhcp/ha-pairs/{pair_id}/status`)
- Implement HA state machine:
- Primary: Active DHCP service
- Secondary: Standby, ready to take over
- Transition logic based on VIP and health checks
- Add HA coordination to `handle_ha_coordination()` (lines 237-244)
- Test failover scenarios

### 3.4 Add Comprehensive Test Suite

**Files**: `testing/integration/scenarios/*.rs`, `testing/mocks/*/src/*.rs`

- Complete integration test scenarios:
- Zone management: create, update, delete zones
- DHCP scope management: add/remove scopes
- Threat blocking: MISP feed → dnsdist RPZ
- Policy enforcement: Cerbos → dnsdist rules
- Failover scenarios: HA pair transitions
- Expand mock services:
- Complete etcd-mock implementation
- Create knot-mock for DNS API testing
- Create kea-mock for DHCP control testing
- Create cerbos-mock for policy testing
- Add performance benchmarks:
- Config propagation latency
- Cache operation throughput
- etcd watch performance

### 3.5 Update Architecture Documentation

**Files**: `docs/architecture.md`

- Remove references to `raft-rs` (lines 16, 54) - replace with `etcd-client`
- Update code examples to match actual implementation:
- Fix `orchestrator.rs` example (lines 41-85)
- Update service examples to match actual plugin structure
- Update LOC estimates if needed
- Add notes about actual vs planned differences

### 3.6 Remove Hardcoded Values

**Files**: `agent/src/services/kea.rs`, `agent/src/services/dnsdist.rs`, `agent/src/services/knot.rs`

- Move hardcoded interface names to config:
- `DnsServiceConfig::interface` field
- `DhcpServiceConfig::interface` field
- Move hardcoded ports to config:
- Kea control port (8000)
- dnsdist control port (5199)
- Prometheus metrics port (9090)
- Move hardcoded upstream resolvers to `DnsdistServiceConfig`

### 3.7 Create Backup/Restore Documentation

**Files**: `docs/operations/backup-restore.md`

- Document etcd backup procedures:
- Using `etcdctl snapshot save`
- Automated backup scripts
- Backup retention policies
- Document restore procedures:
- Restore from snapshot
- Point-in-time recovery
- Document cache backup (sled database files)
- Document Nebula certificate backup

### 3.8 Fix Missing Workspace Dependency

**Files**: `Cargo.toml`, `integrations/misp-sync/Cargo.toml`

- Add `reqwest` to workspace dependencies in root `Cargo.toml`
- Update `integrations/misp-sync/Cargo.toml` to use `reqwest.workspace = true`
- Ensure version consistency across workspace

## Testing & Validation

- Run full test suite after each priority phase
- Validate TLS connections with test certificates
- Test Cerbos integration with local Cerbos server
- Verify Lynis audits run periodically
- Validate CI/CD pipelines pass
- Test DB-only agent role behavior
- Verify cache eviction and TTL enforcement

## Success Criteria

- All critical security issues resolved (TLS, Cerbos working)
- All high-priority features implemented
- CI/CD pipeline running and passing
- Test coverage >70% for core components
- Documentation matches implementation
- No critical code quality issues remaining
