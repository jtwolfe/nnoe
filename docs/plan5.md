# Documentation Update Plan

## Objective

Update all documentation to accurately reflect the current state of the NNOE codebase, ensuring consistency, completeness, and accuracy across all documentation files.

## Files to Update

### 1. Root README.md

**Location**: `/home/jim/Workspace/nnoe/README.md`

**Changes needed**:

- Fix typo: Change reference from `docs/architecture.md` to `docs/architecure.md` (actual filename) OR rename file
- Update project status section with actual completion state from completion-status.md
- Add information about new features:
- Role mappings for DNS policy
- IPv6 lease support in Kea hooks
- Metrics collection system
- MISP multiple instance support
- DB-only agent role
- Update quick start with actual environment variables from docker-compose files

### 2. Architecture Documentation

**Location**: `/home/jim/Workspace/nnoe/docs/architecure.md`

**Changes needed**:

- Fix filename: Rename `architecure.md` → `architecture.md` OR update all references
- Document role mappings feature:
- etcd path: `/nnoe/role-mappings/<ip_or_subnet>`
- Usage in dnsdist for Cerbos policy evaluation
- IP/subnet to role mapping structure
- Document metrics system:
- AgentMetrics struct and counters
- Prometheus exporter integration
- Metrics exposed on port 9090
- Document IPv6 support:
- Kea hooks IPv6 lease callouts (lease6_offer, lease6_renew, lease6_release, lease6_expire)
- IPv6 lease data structure with type, iaid, duid, preferred_lft
- expires_at timestamp in both IPv4 and IPv6 leases
- Document DB-only agent role:
- Implementation in orchestrator (skip service registration)
- Configuration via `node.role = "db-only"`
- Use case: etcd replication without DNS/DHCP services
- Document HA coordination implementation:
- VIP detection via `ip addr` command
- etcd status keys: `/nnoe/dhcp/ha-pairs/{pair_id}/nodes/{node}/status`
- State machine: Primary/Standby
- Document MISP sync enhancements:
- Multiple instance support (MISP_URL_2, MISP_API_KEY_2, etc.)
- Tag filtering (MISP_FILTER_TAGS)
- Deduplication logic
- Update code examples to match actual implementation:
- Orchestrator with metrics field
- Watch prefixes including `/nnoe/role-mappings`
- DB-only role handling

### 3. API Documentation

#### 3.1 Agent API (`docs/api/agent-api.md`)

**Changes needed**:

- Expand Internal APIs section:
- Document `AgentMetrics` struct with all fields:
- config_updates_total
- service_reloads_total
- dns_queries_total
- blocked_queries_total
- dhcp_leases_total
- dhcp_leases_active
- Document methods: increment_*, get_*
- Add Orchestrator methods:
- `nebula_manager()`: Access to NebulaManager
- DB-only role handling logic
- Add service-specific APIs:
- **KnotService**: DNSSEC key generation, zone transfer, dynamic updates
- **KeaService**: HA coordination (check_vip, check_peer_status, update_ha_status_in_etcd)
- **DnsdistService**: Role mapping management, RPZ generation, Cerbos rule conversion
- **LynisService**: Report parsing structure, audit scheduling
- **CerbosService**: Policy checking interface
- Document CacheManager API:
- `get_stats()`: Returns CacheStats with size_bytes, entry_count, max_size_bytes, ttl_secs
- TTL and LRU eviction background task
- Document EtcdClient TLS support:
- TLS configuration via TlsConfig
- rustls integration
- Client certificate authentication

#### 3.2 etcd Schema (`docs/api/etcd-schema.md`)

**Changes needed**:

- Add new key patterns:
- `/nnoe/role-mappings/<ip_or_subnet>` - IP/subnet to role mappings (JSON array of roles)
- `/nnoe/dhcp/ha-pairs/<pair_id>/nodes/<node>/status` - HA pair node status
- `/nnoe/dhcp/leases/<lease_id>` - DHCP lease data (IPv4 and IPv6)
- `/nnoe/agents/health/<node_name>` - Agent health status
- `/nnoe/agents/metrics/<node_name>` - Agent metrics (if stored)
- Update DHCP lease structure:
- Add IPv6 lease fields: `type`, `iaid`, `duid`, `preferred_lft`
- Document `expires_at` field for both IPv4 and IPv6
- Document base64 encoding in Kea hooks
- Add HA coordination schema:
- HA pair configuration structure
- Node status values: "primary", "standby", "unknown"
- TTL on status keys
- Update Watch Patterns:
- Add `/nnoe/role-mappings` to watched prefixes
- Document watch notification flow to plugins
- Add examples for new structures:
- Role mapping JSON example
- IPv6 lease JSON example
- HA status JSON example

#### 3.3 phpIPAM Extensions (`docs/api/phpipam-extensions.md`)

**Status**: Already documented, verify completeness
**Verify**:

- TLS configuration methods
- Retry logic implementation
- Grafana embed URL generation
- Watch hooks documentation

### 4. Operations Documentation

#### 4.1 Monitoring (`docs/operations/monitoring.md`)

**Changes needed**:

- Update Agent Metrics section with actual metric names from metrics.rs:
- `nnoe_agent_config_updates_total` (Counter)
- `nnoe_agent_service_reloads_total` (Counter)
- `nnoe_agent_dns_queries_total` (Counter)
- `nnoe_agent_blocked_queries_total` (Counter)
- `nnoe_agent_dhcp_leases_total` (Counter)
- `nnoe_agent_dhcp_leases_active` (Gauge)
- `nnoe_agent_etcd_connected` (Gauge)
- `nnoe_agent_ha_state` (Gauge)
- `nnoe_agent_cache_size_bytes` (Gauge)
- `nnoe_agent_cache_entries` (Gauge)
- `nnoe_agent_dns_query_rate` (Gauge, calculated)
- `nnoe_agent_uptime_seconds` (Gauge)
- Document Prometheus exporter:
- Direct etcd connection for metrics
- Environment variables: ETCD_ENDPOINTS, ETCD_PREFIX
- Metrics collection cycle
- Document Grafana dashboard:
- Location: `management/monitoring/grafana/dashboards/nnoe-dashboard.json`
- Alerting rules: `management/monitoring/grafana/alerts/nnoe-alerts.yml`
- Update service metrics to reflect actual implementations

#### 4.2 Security (`docs/operations/security.md`)

**Changes needed**:

- Document TLS setup for etcd:
- Configuration in agent.toml
- Certificate paths and verification
- rustls configuration
- Document Nebula certificate management:
- Rotation automation (rotate-cert.sh)
- Expiry checking (check-expiry.sh)
- CRL management (revoke-cert.sh)
- Verify all security practices match current implementation

#### 4.3 Backup/Restore (`docs/operations/backup-restore.md`)

**Status**: Already enhanced, verify completeness
**Verify**: All backup/restore procedures match actual scripts

#### 4.4 Troubleshooting (`docs/operations/troubleshooting.md`)

**Changes needed**:

- Add troubleshooting for:
- Role mappings not working
- HA coordination issues
- IPv6 lease problems
- Metrics collection failures
- DB-only node configuration

### 5. Deployment Documentation

#### 5.1 Docker (`docs/deployment/docker.md`)

**Changes needed**:

- Verify environment variables match docker-compose.dev.yml and docker-compose.prod.yml
- Document new variables:
- MISP_URL_2, MISP_API_KEY_2 (multiple instances)
- MISP_FILTER_TAGS, MISP_DEDUP
- NNOE_GRAFANA_URL, NNOE_GRAFANA_DASHBOARD
- NNOE_ETCD_TLS_* variables
- PHPIPAM_PORT
- Document health checks:
- Agent health endpoint (port 8080)
- etcd health checks
- Service health check implementation
- Update with actual port numbers and service names

#### 5.2 Kubernetes (`docs/deployment/kubernetes.md`)

**Changes needed**:

- Verify manifests match actual files in deployments/kubernetes/
- Document:
- ServiceMonitor for Prometheus
- NetworkPolicy implementation
- Resource limits and requests
- Health check probes (httpGet on port 8080)
- Update examples to match actual ConfigMap structure

#### 5.3 Ansible (`docs/deployment/ansible.md`)

**Changes needed**:

- Verify playbook structure matches deployments/ansible/
- Document:
- Idempotency checks
- Validation tasks
- OS compatibility requirements
- Resource validation (memory, CPU)

#### 5.4 Manual (`docs/deployment/manual.md`)

**Changes needed**:

- Document uninstall script: `deployments/manual/uninstall.sh`
- Document upgrade script: `deployments/manual/upgrade.sh`
- Verify install script matches actual implementation

### 6. Development Documentation

#### 6.1 Getting Started (`docs/development/getting-started.md`)

**Changes needed**:

- Verify build process matches current Cargo.toml
- Update with actual test execution commands
- Document testcontainers usage for integration tests
- Verify etcd setup commands are current

#### 6.2 Plugin Development (`docs/development/plugin-development.md`)

**Status**: Already enhanced, verify completeness
**Verify**: Troubleshooting section matches current implementation

#### 6.3 Contributing (`docs/development/contributing.md`)

**Changes needed**:

- Verify project structure matches actual layout
- Update with CI/CD requirements (pre-commit hooks, cargo-audit, etc.)
- Verify contribution guidelines match current process

### 7. Documentation Infrastructure

#### 7.1 Docs README (`docs/README.md`)

**Changes needed**:

- Fix typo: Update architecture link to match actual filename
- Verify all links are correct
- Update documentation status section
- Add completion-status.md to documentation index

### 8. Example Documentation

#### 8.1 Zone Management (`docs/examples/zone-management/README.md`)

**Verify**: Examples match actual zone data structure and DNSSEC support

#### 8.2 DHCP Scopes (`docs/examples/dhcp-scopes/README.md`)

**Changes needed**:

- Add IPv6 scope examples
- Document HA pair configuration
- Add lease expiration examples

#### 8.3 Threat Integration (`docs/examples/threat-integration/README.md`)

**Changes needed**:

- Document multiple MISP instance configuration
- Add tag filtering examples
- Document deduplication behavior

## Implementation Order

1. **Critical Updates** (High Priority):

- Fix architecture filename typo (architecure.md → architecture.md) OR update all references
- Update README.md with current status
- Expand etcd schema with role-mappings and HA paths
- Update monitoring.md with actual metric names

2. **API Documentation** (High Priority):

- Expand agent-api.md with service-specific APIs
- Document metrics system
- Document HA coordination methods
- Add CacheManager and AgentMetrics APIs

3. **Architecture Updates** (Medium Priority):

- Document role mappings feature
- Document IPv6 support
- Document DB-only agent role
- Update code examples

4. **Deployment Documentation** (Medium Priority):

- Verify and update Docker deployment guide
- Verify Kubernetes manifests match docs
- Update with new environment variables

5. **Operations Documentation** (Medium Priority):

- Update monitoring guide with actual metrics
- Add troubleshooting for new features

6. **Development Documentation** (Low Priority):

- Verify accuracy of getting started guide
- Update examples with IPv6 and new features

## Success Criteria

- All documentation references match actual filenames
- All code examples compile and run
- All API documentation matches actual method signatures
- All etcd schema paths match actual implementation
- All environment variables documented match actual deployment files
- All metrics documented match actual metric names
- All new features are documented
- No broken internal links
- Consistent terminology across all docs
