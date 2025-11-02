# NNOE Issues Resolution Plan

## Priority 1: Critical Reliability Issues (Weeks 1-2)

### 1.1 Complete Metrics Collection Implementation

**Files**: `management/monitoring/prometheus-exporter/src/metrics.rs`, `management/monitoring/prometheus-exporter/src/main.rs`

**Current State**: Metrics collector has TODOs and placeholder values (line 98 in metrics.rs). Only uptime is collected; all other metrics are static.

**Changes Needed**:

- Add metrics collection API/interface for agent to report statistics
- Implement cache size collection from sled database (query CacheManager)
- Add etcd connection status tracking (from EtcdClient)
- Collect service reload counts from plugin registry
- Integrate with agent via shared metrics registry or HTTP endpoint
- Replace placeholder values with actual data collection

**Dependencies**: Requires agent to expose metrics (either shared registry or HTTP endpoint).

### 1.2 Fix Ignored and Incomplete Tests

**Files**: `testing/integration/scenarios/ha_failover_test.rs`, `testing/integration/scenarios/*.rs`, `agent/tests/*.rs`

**Current State**: Many tests are `#[ignore]` and contain placeholder assertions (`assert!(true)`). Tests require external services (etcd, Keepalived) that aren't mocked.

**Changes Needed**:

- Replace `#[ignore]` with testcontainers-rs for etcd integration tests
- Implement actual HA failover test logic in `ha_failover_test.rs`:
- Test VIP acquisition/detection
- Test state transitions (Primary/Standby)
- Mock Keepalived VIP status for unit tests
- Complete other scenario tests (dhcp_scope_test.rs, zone_management_test.rs, threat_blocking_test.rs)
- Remove placeholder assertions and add real test logic
- Update test fixtures to include IPv6 examples
- Achieve >70% test coverage target

**Dependencies**: testcontainers-rs crate already in workspace dependencies.

## Priority 2: High Priority Functional Gaps (Weeks 3-4)

### 2.1 Replace Systemctl with Direct Kea Process Management

**Files**: `agent/src/services/kea.rs` (lines 306-348)

**Current State**: `start_kea()` and `stop_kea()` use `systemctl` commands, limiting deployment flexibility (e.g., Docker containers without systemd).

**Changes Needed**:

- Replace `Command::new("systemctl")` with direct `kea-dhcp4` binary execution
- Add process management with PID tracking (store Child handle or PID file)
- Implement signal-based graceful shutdown (SIGTERM) instead of systemctl stop
- Add configurable Kea binary path in `DhcpServiceConfig`
- Update error handling for process spawn failures
- Maintain HA coordination logic (handle_ha_coordination remains unchanged)

**Testing**: Verify Kea starts/stops correctly in Docker and systemd environments.

### 2.2 Implement Lynis Report Parsing

**Files**: `agent/src/services/lynis.rs` (lines 64-97)

**Current State**: `run_audit()` has placeholder parsing - report fields are empty (score: None, warnings: Vec::new(), etc.). Comment says "simplified - real parsing would be more complex".

**Changes Needed**:

- Parse Lynis report file (typically `/var/log/lynis-report.dat` or JSON format)
- Extract security score from report
- Parse warnings and suggestions sections
- Parse section-by-section data (create `LynisSection` from report file)
- Handle multiple Lynis output formats (JSON, DAT, plain text)
- Update `LynisReport` struct population with actual parsed data
- Add error handling for malformed reports

**Dependencies**: May need additional parsing crate (e.g., `regex` for DAT format or `serde_json` for JSON).

### 2.3 Refine DB-Only Agent Role Logic

**Files**: `agent/src/core/orchestrator.rs` (lines 39-50, 64-71)

**Current State**: Nebula is initialized for all nodes including DB-only, even though it may not be needed for replication-only nodes.

**Changes Needed**:

- Move Nebula initialization check inside `run()` method after role check
- Skip Nebula start for DB-only nodes (unless explicitly enabled for control plane access)
- Add config flag `nebula.required_for_db_only` to allow optional Nebula on DB nodes
- Update documentation to clarify DB-only node behavior

## Priority 3: Medium Priority Improvements (Weeks 5-6)

### 3.1 Add TLS and Retry Logic to phpIPAM Plugin

**Files**: `integrations/phpipam-plugin/src/NNOE.php` (lines 49-55, 178-224)

**Current State**: etcd client uses HTTP without TLS support. No retry/backoff logic as recommended in plan2.md Priority 2.5.

**Changes Needed**:

- Add TLS configuration to Guzzle client (verify SSL certificates)
- Implement retry logic with exponential backoff for etcd operations (etcdPut, etcdGet)
- Add connection timeout configuration
- Improve error handling with context (include endpoint URLs in errors)
- Update `nnoe-config.php.example` to include TLS cert paths

### 3.2 Add IPv6 Examples and Test Fixtures

**Files**: `testing/integration/fixtures/dhcp-scope-example.json`, `docs/examples/*.md`

**Current State**: Architecture claims "IPv6 native" but examples/tests only show IPv4.

**Changes Needed**:

- Add IPv6 subnet examples to `dhcp-scope-example.json`
- Create IPv6 zone example in `zone-example.json`
- Update documentation examples to show IPv6 configuration
- Add IPv6 test cases to integration tests
- Verify Knot/Kea config generation handles IPv6 addresses

### 3.3 Fix Minor TODOs in Codebase

**Files**:

- `integrations/kea-hooks/src/libdhcp_etcd.cpp` (lines 64-65, 166): Base64 encoding and DELETE operation
- `agent/src/services/dnsdist.rs` (line 295): Extract role from request instead of hardcoded

**Changes Needed**:

- Implement base64 encoding in Kea hook for etcd key/value
- Implement DELETE operation for lease releases
- Extract actual role/principal from DNS request context in dnsdist Lua rule generation

## Priority 4: Documentation and Code Quality (Weeks 7-8)

### 4.1 Update Architecture Documentation

**Files**: `docs/architecture.md`

**Current State**: Code examples don't match actual implementation (orchestrator.rs uses different watch pattern).

**Changes Needed**:

- Update Rust agent example to match actual `orchestrator.rs` implementation
- Fix watch loop example to show `watch_prefix` with plugin notification
- Remove any references to old patterns (if any)
- Verify all code snippets compile and match current codebase

### 4.2 Enhance Plugin Development Documentation

**Files**: `docs/development/plugin-development.md`

**Current State**: File exists but content completeness not verified.

**Changes Needed**:

- Add complete ServicePlugin trait examples
- Include code samples for creating custom plugins
- Document plugin registration and lifecycle
- Add troubleshooting section

### 4.3 Complete Backup/Restore Documentation

**Files**: `docs/operations/backup-restore.md`

**Current State**: File exists but may lack detail on point-in-time recovery.

**Changes Needed**:

- Add detailed etcd snapshot procedures
- Document point-in-time recovery steps
- Include sled cache backup/restore procedures
- Add Nebula certificate backup guidance

## Success Criteria

- All tests pass without `#[ignore]` (except optional performance tests)
- Test coverage >70% for core components
- Metrics exporter shows real data from agent
- Kea works in both systemd and containerized environments
- Lynis reports contain parsed security data
- phpIPAM plugin supports TLS connections
- IPv6 examples exist in fixtures and docs
- All TODOs in critical paths are resolved
- Documentation matches actual implementation

## Testing Strategy

- Run `cargo test --all` after each priority phase
- Use testcontainers for integration tests requiring etcd
- Verify metrics collection with actual agent instance
- Test Kea process management in Docker container
- Validate Lynis parsing with sample report files
- Test TLS connections with self-signed certificates
