# NNOE Complete Implementation Plan

## Executive Summary

This plan addresses completion of the New Network Orchestration Engine (NNOE) project. The project is currently in Phase 1 (Foundation & Core Agent) with approximately 60-70% implementation. Critical gaps include incomplete HA functionality, missing TLS support, incomplete service integrations, documentation inconsistencies, and incomplete test coverage.

**Current State Assessment:**

- Core orchestrator and plugin system: ✅ Implemented
- Service plugins (Knot, Kea, dnsdist, Cerbos, Lynis): ✅ Partially implemented with TODOs
- etcd client: ✅ Implemented, TLS incomplete
- Cache manager: ✅ Implemented
- Nebula manager: ✅ Implemented
- MISP sync: ✅ Implemented
- Prometheus exporter: ⚠️ TODOs for actual metrics
- Kea hooks: ⚠️ TODOs for base64 encoding
- Documentation: ⚠️ Partially out of date
- Tests: ⚠️ Basic coverage, missing comprehensive integration tests

## Stage 1: Critical Security and Reliability Fixes

### 1.1 Implement etcd TLS Support

**Priority: CRITICAL**
**Files:** `agent/src/etcd/client.rs`

- Research etcd-client 0.11 TLS API documentation
- Implement TLS configuration application via `ConnectOptions`
- Add integration tests for TLS connections
- Update configuration examples with TLS settings
- **Acceptance Criteria:** Agent can connect to etcd with TLS, tests pass

### 1.2 Complete Kea HA Coordination

**Priority: HIGH**
**Files:** `agent/src/services/kea.rs`

- Implement `check_vip()` using `nix` crate or `ip` command parsing to detect VIP presence
- Implement `check_peer_status()` by querying etcd keys at `/nnoe/dhcp/ha-pairs/{pair_id}/nodes/{peer_node}/status`
- Implement `update_ha_status_in_etcd()` to write status to etcd with TTL
- Add split-brain prevention logic (use etcd leases/transactions)
- Add unit tests for HA state machine
- **Acceptance Criteria:** HA pairs correctly coordinate primary/standby, no split-brain scenarios

### 1.3 Fix Kea Hooks Base64 Encoding

**Priority: MEDIUM**
**Files:** `integrations/kea-hooks/src/libdhcp_etcd.cpp`

- Implement proper base64 encoding for etcd v3 API key/value
- Implement DELETE operation for `lease4_release`
- Add error handling for curl operations
- Update build script to ensure dependencies (libcurl, jsoncpp) are available
- **Acceptance Criteria:** Kea hooks successfully sync leases to etcd

## Stage 2: Complete Service Implementations

### 2.1 Enhance dnsdist Service

**Priority: MEDIUM**
**Files:** `agent/src/services/dnsdist.rs`

- Implement role extraction from DNS query context (store roles in dnsdist metadata or custom headers)
- Improve Cerbos expression to Lua conversion with proper parser (consider using `nom` or similar)
- Add anomaly detection integration (stub for ML-based detection)
- Add RPZ zone file generation support
- **Acceptance Criteria:** dnsdist correctly applies policies with role-based access control

### 2.2 Complete Lynis Report Parsing

**Priority: LOW**
**Files:** `agent/src/services/lynis.rs`

- Implement proper Lynis report parsing (parse `/tmp/lynis.report` format)
- Extract score, warnings, suggestions, and section data
- Store parsed data in structured format
- Add validation for report file format
- **Acceptance Criteria:** Lynis reports are fully parsed and uploaded to etcd

### 2.3 Complete Prometheus Metrics Collection

**Priority: MEDIUM**
**Files:** `management/monitoring/prometheus-exporter/src/metrics.rs`

- Implement metrics collection from agent via:
- etcd client connection status polling
- Cache manager statistics API
- Service health check aggregation
- DNS/DHCP query rates (via dnsdist/Kea APIs)
- Add metrics for HA state, config updates, service reloads
- Expose metrics via HTTP endpoint (already implemented in main.rs)
- **Acceptance Criteria:** All metrics are collected from actual agent state

### 2.4 Enhance Knot DNS Service

**Priority: LOW**
**Files:** `agent/src/services/knot.rs`

- Add DNSSEC key management
- Add zone transfer support
- Add dynamic update support
- Improve error handling for service restarts
- **Acceptance Criteria:** Knot service supports full DNS feature set

## Stage 3: Testing Infrastructure

### 3.1 Expand Unit Tests

**Priority: HIGH**
**Files:** `agent/tests/*.rs`, new test files

- Add comprehensive tests for:
- etcd client with TLS
- HA coordination logic
- Service plugin lifecycle
- Cache TTL and eviction
- Nebula manager restart logic
- Achieve >80% code coverage
- **Acceptance Criteria:** All critical paths have unit tests, coverage report shows >80%

### 3.2 Implement Integration Tests

**Priority: HIGH**
**Files:** `testing/integration/scenarios/*.rs`

- Complete HA failover scenario tests
- Complete threat blocking scenario tests  
- Complete DHCP scope management tests
- Complete zone management tests
- Use testcontainers for etcd, MISP, Cerbos
- **Acceptance Criteria:** All scenarios pass in CI environment

### 3.3 Enhance E2E Tests

**Priority: MEDIUM**
**Files:** `testing/integration/e2e/e2e_tests.rs`

- Test full agent lifecycle with all services
- Test etcd watch propagation across multiple agents
- Test HA failover end-to-end
- Test MISP sync integration
- **Acceptance Criteria:** E2E test suite runs successfully in Docker Compose environment

### 3.4 Performance Benchmarks

**Priority: LOW**
**Files:** `testing/performance/benchmarks/*.rs`

- Complete DNS query benchmark
- Add etcd watch latency benchmarks
- Add cache performance benchmarks
- Add load test scripts validation
- **Acceptance Criteria:** Benchmarks run and produce meaningful results

## Stage 4: Documentation Updates

### 4.1 Update Architecture Documentation

**Priority: MEDIUM**
**Files:** `docs/architecture.md`

- Fix filename: `architecure.md` → `architecture.md` (completed)
- Update component descriptions to match current implementation
- Add diagrams for HA coordination flow
- Document etcd schema fully
- Update code examples to match actual implementation
- **Acceptance Criteria:** Architecture doc matches codebase exactly

### 4.2 Update API Documentation

**Priority: MEDIUM**
**Files:** `docs/api/*.md`

- Complete etcd schema documentation with all key patterns
- Document agent API with actual method signatures
- Add phpIPAM extension API documentation
- Document service plugin APIs
- **Acceptance Criteria:** API docs are complete and accurate

### 4.3 Update Deployment Guides

**Priority: HIGH**
**Files:** `docs/deployment/*.md`

- Verify all Docker Compose commands work
- Update Kubernetes manifests to match current structure
- Update Ansible playbooks documentation
- Add troubleshooting sections based on actual deployment experience
- **Acceptance Criteria:** Deployment guides are tested and accurate

### 4.4 Update Development Documentation

**Priority: MEDIUM**
**Files:** `docs/development/*.md`

- Update getting started guide with current build process
- Verify plugin development guide matches actual code
- Update contributing guide with current project structure
- **Acceptance Criteria:** New developers can follow guides successfully

### 4.5 Update Operational Documentation

**Priority: MEDIUM**
**Files:** `docs/operations/*.md`

- Complete monitoring guide with actual Prometheus metrics
- Update troubleshooting guide with known issues and solutions
- Complete security guide with TLS setup instructions
- **Acceptance Criteria:** Operators can use docs to run NNOE in production

## Stage 5: Integration Components

### 5.1 Complete phpIPAM Plugin

**Priority: MEDIUM**
**Files:** `integrations/phpipam-plugin/src/NNOE.php`

- Implement etcd watch hooks for DNS/DHCP views
- Add Grafana embed support
- Add custom dashboard components
- Complete API extensions
- Test with actual phpIPAM instance
- **Acceptance Criteria:** phpIPAM plugin integrates with etcd and provides NNOE extensions

### 5.2 Enhance MISP Sync

**Priority: LOW**
**Files:** `integrations/misp-sync/src/main.rs`

- Add support for multiple MISP instances
- Add filtering by event tags
- Add deduplication logic
- Improve error recovery
- **Acceptance Criteria:** MISP sync handles multiple sources and edge cases

### 5.3 Complete Kea Hooks Integration

**Priority: MEDIUM**
**Files:** `integrations/kea-hooks/src/libdhcp_etcd.cpp`

- Complete base64 encoding implementation (Stage 1.3)
- Add IPv6 lease support (lease6_* callouts)
- Add lease expiration handling
- Test with actual Kea instance
- **Acceptance Criteria:** Kea hooks work in production Kea deployments

## Stage 6: Management Components

### 6.1 etcd Orchestrator Scripts

**Priority: MEDIUM**
**Files:** `management/etcd-orchestrator/*.sh`

- Verify bootstrap script works for etcd cluster
- Complete member management scripts
- Add backup/restore validation tests
- **Acceptance Criteria:** etcd cluster can be fully managed via scripts

### 6.2 Nebula CA Management

**Priority: LOW**
**Files:** `management/nebula-ca/*.sh`

- Verify certificate generation scripts
- Add certificate rotation automation
- Add revocation list management
- **Acceptance Criteria:** Nebula certificates can be fully managed

### 6.3 Monitoring Dashboard

**Priority: MEDIUM**
**Files:** `management/monitoring/grafana/dashboards/nnoe-dashboard.json`

- Complete Grafana dashboard with all metrics
- Add alerting rules
- Test with actual Prometheus data
- **Acceptance Criteria:** Dashboard displays all NNOE metrics correctly

## Stage 7: Deployment Configurations

### 7.1 Docker Compose Updates

**Priority: HIGH**
**Files:** `deployments/docker/*.yml`

- Verify all services start correctly
- Add health checks for all containers
- Update environment variable documentation
- Test production compose file
- **Acceptance Criteria:** Docker Compose deployments work out of the box

### 7.2 Kubernetes Manifests

**Priority: HIGH**
**Files:** `deployments/kubernetes/**/*.yaml`

- Complete ConfigMap examples
- Add resource limits and requests
- Add network policies
- Add service monitors for Prometheus
- Test on actual Kubernetes cluster
- **Acceptance Criteria:** Kubernetes deployment works with standard K8s distribution

### 7.3 Ansible Playbooks

**Priority: MEDIUM**
**Files:** `deployments/ansible/**/*.yml`

- Complete all role implementations
- Add idempotency checks
- Add validation tasks
- Test on clean VMs
- **Acceptance Criteria:** Ansible playbooks can deploy NNOE to bare metal

### 7.4 Manual Installation Scripts

**Priority: LOW**
**Files:** `deployments/manual/*.sh`, `scripts/*.sh`

- Verify install script works on major Linux distributions
- Add uninstall script
- Add upgrade script
- Test on Ubuntu, Debian, CentOS, RHEL
- **Acceptance Criteria:** Manual installation works on supported platforms

## Stage 8: CI/CD and Quality Assurance

### 8.1 Enhance CI Pipelines

**Priority: HIGH**
**Files:** `.github/workflows/*.yml`

- Add multi-arch builds (amd64, arm64)
- Add test coverage reporting
- Add security scanning (cargo-audit, clippy)
- Add release automation
- **Acceptance Criteria:** CI runs all tests and produces artifacts

### 8.2 Code Quality Checks

**Priority: MEDIUM**

- Fix all clippy warnings
- Ensure all code follows Rust formatting
- Add pre-commit hooks
- **Acceptance Criteria:** Codebase passes all quality checks

### 8.3 Dependency Updates

**Priority: LOW**

- Update all dependencies to latest stable versions
- Check for security vulnerabilities
- Update Cargo.lock
- **Acceptance Criteria:** All dependencies are up to date and secure

## Stage 9: Final Integration and Validation

### 9.1 End-to-End System Testing

**Priority: CRITICAL**

- Deploy full NNOE system with all components
- Test DNS zone management workflow
- Test DHCP scope management workflow
- Test threat blocking workflow
- Test HA failover scenarios
- **Acceptance Criteria:** Full system works in integrated environment

### 9.2 Performance Validation

**Priority: MEDIUM**

- Validate DNS query rates meet requirements (millions QPS)
- Validate etcd watch latency (<100ms)
- Validate cache performance
- **Acceptance Criteria:** System meets performance requirements

### 9.3 Security Audit

**Priority: HIGH**

- Review TLS configuration
- Review etcd ACL setup
- Review Nebula certificate handling
- Run security scanning tools
- **Acceptance Criteria:** No critical security issues found

## Implementation Order and Dependencies

**Critical Path:**

1. Stage 1 (Security/Reliability) → Must be done first
2. Stage 2 (Service Completion) → Can partially overlap with Stage 1
3. Stage 3 (Testing) → Should run in parallel with Stages 1-2
4. Stage 4 (Documentation) → Should be updated as code changes
5. Stages 5-7 (Integrations/Management/Deployment) → Can run in parallel
6. Stage 8 (CI/CD) → Should be set up early and improved iteratively
7. Stage 9 (Final Validation) → Must be last

## Success Criteria

**Project Complete When:**

- All TODOs resolved
- All tests pass (>80% coverage)
- Documentation matches code
- All deployment methods work
- System passes security audit
- Performance requirements met
- E2E scenarios pass

## Estimated Effort

- Stage 1: 2-3 weeks
- Stage 2: 3-4 weeks  
- Stage 3: 2-3 weeks
- Stage 4: 2 weeks
- Stage 5: 2-3 weeks
- Stage 6: 1-2 weeks
- Stage 7: 2-3 weeks
- Stage 8: 1-2 weeks
- Stage 9: 1-2 weeks

**Total: 16-23 weeks** (4-6 months with one developer)
