# NNOE Modular Implementation Plan

## Project Structure

Create a modular monorepo structure with clear separation between core agent, integrations, management components, deployment configs, and testing:

```
nnoe/
├── agent/                    # Rust agent binary (core orchestrator)
│   ├── src/
│   │   ├── main.rs           # Entry point, CLI parsing
│   │   ├── core/             # Core orchestration logic
│   │   ├── etcd/             # etcd client wrapper with watch logic
│   │   ├── sled_cache/       # Local cache management
│   │   ├── nebula/           # Nebula process management
│   │   ├── services/         # Service integration modules
│   │   │   ├── knot.rs       # Knot DNS config generation/control
│   │   │   ├── kea.rs        # Kea DHCP config generation/control
│   │   │   ├── dnsdist.rs    # dnsdist config/control
│   │   │   ├── cerbos.rs     # Cerbos policy client
│   │   │   └── lynis.rs      # Lynis audit execution
│   │   ├── config/           # Configuration management
│   │   └── plugin/           # Plugin system for extensibility
│   ├── Cargo.toml
│   └── tests/
├── integrations/             # External service integrations
│   ├── phpipam-plugin/       # phpIPAM custom plugin (PHP)
│   │   ├── src/              # PHP plugin files
│   │   └── etc/              # Config templates
│   ├── misp-sync/            # MISP feed sync service (Python/Rust)
│   ├── kea-hooks/            # Custom Kea hooks for etcd sync
│   └── cerbos-policies/      # Default Cerbos policy templates
├── management/               # Management node components
│   ├── etcd-orchestrator/    # etcd cluster setup/mgmt scripts
│   ├── nebula-ca/            # Nebula CA management
│   └── monitoring/           # Prometheus exporters, Grafana dashboards
├── deployments/              # Deployment configurations
│   ├── docker/               # Docker Compose files
│   │   ├── docker-compose.dev.yml
│   │   ├── docker-compose.prod.yml
│   │   └── Dockerfile.*
│   ├── kubernetes/           # K8s manifests
│   │   ├── management/         # Management node deployments
│   │   ├── agent/            # Agent DaemonSets/Deployments
│   │   └── operator/         # Optional K8s operator
│   └── ansible/              # Ansible playbooks
│       ├── roles/
│       └── playbooks/
├── testing/                  # Testing infrastructure
│   ├── unit/                 # Unit test fixtures
│   ├── integration/          # Integration test suite
│   │   ├── fixtures/         # Mock services, test containers
│   │   ├── scenarios/        # Test scenarios
│   │   └── e2e/              # End-to-end tests
│   ├── performance/          # Load/perf tests
│   │   ├── benchmarks/
│   │   └── scripts/
│   └── mocks/                # Mock services for isolated testing
│       ├── etcd-mock/
│       ├── knot-mock/
│       └── kea-mock/
├── docs/                     # Documentation
│   ├── architecture.md       # (existing)
│   ├── intention.md          # (existing)
│   ├── api/                  # API documentation
│   │   ├── agent-api.md
│   │   ├── etcd-schema.md
│   │   └── phpipam-extensions.md
│   ├── deployment/           # Deployment guides
│   │   ├── docker.md
│   │   ├── kubernetes.md
│   │   ├── ansible.md
│   │   └── manual.md
│   ├── development/          # Developer guides
│   │   ├── getting-started.md
│   │   ├── contributing.md
│   │   ├── plugin-development.md
│   │   └── testing.md
│   ├── operations/           # Operational docs
│   │   ├── troubleshooting.md
│   │   ├── monitoring.md
│   │   └── security.md
│   └── examples/             # Usage examples
│       ├── zone-management/
│       ├── dhcp-scopes/
│       └── threat-integration/
├── scripts/                  # Utility scripts
│   ├── build.sh
│   ├── test.sh
│   ├── deploy.sh
│   └── generate-certs.sh
├── .github/
│   └── workflows/            # CI/CD pipelines
│       ├── test.yml          # Unit + integration tests
│       ├── e2e.yml           # E2E test suite
│       ├── build.yml         # Multi-arch builds
│       └── release.yml       # Release automation
├── Cargo.toml                # Workspace Cargo.toml
├── LICENSE                   # Apache 2.0
└── README.md                 # Project overview
```

## Implementation Phases

### Phase 1: Foundation & Core Agent (Weeks 1-4)

**1.1 Project Setup**

- Initialize Rust workspace with `Cargo.toml`
- Set up Git structure, `.gitignore`, `.editorconfig`
- Create basic README with quick start
- Set up CI/CD skeleton (GitHub Actions)

**1.2 Core Agent Architecture**

- `agent/src/core/orchestrator.rs`: Main orchestration loop
- `agent/src/core/config.rs`: Configuration loading (YAML/TOML)
- `agent/src/etcd/client.rs`: etcd client wrapper with:
  - Connection management (TLS support)
  - Watch subscription for prefixes (`/dns/zones`, `/dhcp/scopes`, `/policies`)
  - KV operations (get/put with retry logic)
- `agent/src/sled_cache/cache.rs`: Local sled cache manager with:
  - Cache synchronization from etcd
  - TTL/eviction policies
  - Metrics export

**1.3 Nebula Integration**

- `agent/src/nebula/manager.rs`: Nebula process lifecycle (spawn/monitor/restart)
- Certificate management via etcd (`/nebula/certs`)
- Network configuration generation

**1.4 Plugin System**

- `agent/src/plugin/trait.rs`: `ServicePlugin` trait for extensibility
- `agent/src/plugin/registry.rs`: Plugin discovery and registration
- Interface for service-specific config generators

### Phase 2: Service Integrations (Weeks 5-8)

**2.1 Knot DNS Integration**

- `agent/src/services/knot.rs`: 
  - Config generator (Knot JSON from etcd zones)
  - Zone file management
  - DNSSEC key generation/rotation
  - Process control (reload via signal/API)
- Templates for Knot config templates

**2.2 Kea DHCP Integration**

- `agent/src/services/kea.rs`:
  - JSON config generation from etcd (`/dhcp/scopes`)
  - HA pair coordination
  - Lease tracking hooks integration
- `integrations/kea-hooks/`: Custom Kea hook for etcd lease sync

**2.3 dnsdist Integration**

- `agent/src/services/dnsdist.rs`:
  - Lua rule generation from Cerbos policies
  - RPZ feed management (from MISP)
  - Anomaly detection rules
  - Config hot-reload

**2.4 Cerbos Integration**

- `agent/src/services/cerbos.rs`: gRPC client for policy checks
- Policy caching layer
- Error handling and fallback logic

**2.5 Lynis Integration**

- `agent/src/services/lynis.rs`: Scheduled audit execution
- Report parsing and etcd upload (`/audit/lynis/<node>`)

### Phase 3: Management Components (Weeks 9-12)

**3.1 phpIPAM Plugin Development**

- `integrations/phpipam-plugin/src/`:
  - Custom API endpoints for etcd sync
  - Dashboard widgets (IP utilization, query rates)
  - DNS/DHCP admin UI extensions
  - Security/Threats viewer
  - Nebula topology visualization
- Database schema extensions
- API integration layer for etcd operations

**3.2 MISP Sync Service**

- `integrations/misp-sync/`: Periodic feed fetcher
  - MISP API client
  - Feed parsing and normalization
  - etcd KV writes (`/threats/domains`)
  - Configurable intervals and filters

**3.3 etcd Orchestration**

- `management/etcd-orchestrator/`:
  - Cluster bootstrap scripts
  - Member management
  - Backup/restore utilities
  - Health check endpoints

**3.4 Nebula CA Management**

- `management/nebula-ca/`: Certificate authority tools
  - CA generation
  - Node certificate signing
  - Revocation management
  - Distribution via etcd

### Phase 4: Testing Infrastructure (Weeks 13-16)

**4.1 Unit Testing Framework**

- Agent unit tests with mocks (`testing/mocks/`)
- Service integration unit tests
- Config generation validation tests
- Coverage targets (80%+)

**4.2 Integration Test Suite**

- `testing/integration/`:
  - Docker Compose test environment
  - Test scenarios for:
    - Zone creation/update propagation
    - DHCP scope management
    - Policy enforcement
    - Failover scenarios
    - MISP threat blocking
  - Test fixtures (sample zones, scopes, policies)
  - Assertion helpers

**4.3 Mock Services**

- `testing/mocks/etcd-mock/`: Mock etcd server (use `etcd-test-server` or embedded)
- `testing/mocks/knot-mock/`: Knot API mock
- `testing/mocks/kea-mock/`: Kea control API mock
- `testing/mocks/cerbos-mock/`: Cerbos gRPC mock

**4.4 End-to-End Tests**

- `testing/integration/e2e/`:
  - Full stack deployment (Docker Compose)
  - Multi-node scenarios
  - HA failover tests
  - Performance benchmarks (QPS, latency)
  - Network partition tests

**4.5 Performance Testing**

- `testing/performance/`:
  - DNS query load tests (using `dnsperf`)
  - DHCP lease stress tests
  - etcd watch latency measurements
  - Cache hit rate analysis
  - Resource utilization profiling

### Phase 5: Deployment Configurations (Weeks 17-20)

**5.1 Docker Deployment**

- `deployments/docker/`:
  - `Dockerfile.agent`: Multi-stage Rust build
  - `Dockerfile.phpipam`: phpIPAM with plugins
  - `Dockerfile.misp`: MISP containerization
  - `docker-compose.dev.yml`: Development stack
  - `docker-compose.prod.yml`: Production HA setup
  - Environment variable management
  - Volume mounts for persistence

**5.2 Kubernetes Deployment**

- `deployments/kubernetes/`:
  - `management/`: StatefulSets for etcd, phpIPAM
  - `agent/`: DaemonSet for agents
  - `operator/`: Optional K8s operator for lifecycle management
  - ConfigMaps and Secrets management
  - Service definitions
  - Network policies (for Nebula isolation)
  - Helm charts (optional)

**5.3 Ansible Playbooks**

- `deployments/ansible/`:
  - `roles/etcd/`: etcd cluster setup
  - `roles/agent/`: Agent installation/configuration
  - `roles/nebula/`: Nebula overlay setup
  - `roles/phpipam/`: phpIPAM deployment
  - `playbooks/deploy.yml`: Full deployment
  - `playbooks/upgrade.yml`: Upgrade procedures
  - Inventory templates

**5.4 Manual Deployment**

- `deployments/manual/`: Shell scripts for bare-metal/VM
  - Systemd service files
  - Init scripts
  - Configuration templates
  - Health check scripts

### Phase 6: Documentation (Ongoing, Weeks 21-24)

**6.1 API Documentation**

- `docs/api/agent-api.md`: Agent CLI and internal APIs
- `docs/api/etcd-schema.md`: etcd KV schema (paths, formats)
- `docs/api/phpipam-extensions.md`: phpIPAM plugin APIs

**6.2 Deployment Guides**

- `docs/deployment/docker.md`: Docker setup and operation
- `docs/deployment/kubernetes.md`: K8s deployment guide
- `docs/deployment/ansible.md`: Ansible automation guide
- `docs/deployment/manual.md`: Manual installation

**6.3 Development Documentation**

- `docs/development/getting-started.md`: Local development setup
- `docs/development/contributing.md`: Contribution guidelines
- `docs/development/plugin-development.md`: Plugin API reference
- `docs/development/testing.md`: Testing guide

**6.4 Operational Documentation**

- `docs/operations/troubleshooting.md`: Common issues and fixes
- `docs/operations/monitoring.md`: Metrics, alerts, dashboards
- `docs/operations/security.md`: Security hardening, TLS setup
- `docs/operations/backup-restore.md`: Disaster recovery

**6.5 Examples**

- `docs/examples/`: Step-by-step examples for common scenarios

## Key Technical Decisions

**Modularity:**

- Rust workspace for shared libraries (e.g., `nnoe-common`, `nnoe-etcd-client`)
- Plugin trait system for service integrations (enables custom services)
- Configuration-driven behavior (YAML/TOML configs)

**Extensibility:**

- Plugin system allows adding new services without core changes
- etcd schema is versioned and extensible
- phpIPAM plugin architecture supports custom UI extensions

**Testing Strategy:**

- Unit tests: >80% coverage for core logic
- Integration tests: Docker Compose-based, run in CI
- E2E tests: Full stack, multi-node scenarios
- Performance tests: Automated benchmarks, regression tracking

**Documentation:**

- Markdown-based, rendered via GitHub Pages or similar
- Code examples in all major docs
- Architecture diagrams (PlantUML, Mermaid)
- API docs generated from code (rustdoc for Rust, PHPDoc for PHP)

## Dependencies & Tools

**Rust Agent:**

- `etcd-rs` or `etcd-client` for etcd
- `sled` for embedded cache
- `tokio` for async runtime
- `serde` for serialization
- `clap` for CLI
- `tracing` for logging

**Testing:**

- `cargo test` for unit tests
- `testcontainers-rs` for integration tests
- `docker-compose` for E2E
- `criterion` for benchmarks

**CI/CD:**

- GitHub Actions
- Multi-arch Docker builds
- Automated testing on PR
- Release automation

## Success Criteria

- All components build and deploy successfully
- Integration tests pass (>95% pass rate)
- Documentation covers all major use cases
- Performance meets targets (sub-100ms config propagation, millions QPS)
- Codebase is modular and extensible (plugins work independently)
