# NNOE - New Network Orchestration Engine

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

The New Network Orchestration Engine (NNOE) is an open-source, distributed DDI (DNS, DHCP, IPAM) platform inspired by BlueCat Integrity, emphasizing modularity, high availability (HA), security, and deployment flexibility across VMs, Docker, and Kubernetes (K8s).

## Features

- **Unified DDI Management**: Centralized DNS, DHCP, and IPAM through phpIPAM UI
- **Distributed Architecture**: etcd-based consensus for config sync with sled-embedded local caching
- **High Performance**: Knot DNS, Kea DHCP, and dnsdist for millions of QPS
- **Security First**: Cerbos policies, MISP threat feeds, Lynis auditing, Nebula overlay networking
- **Modular & Extensible**: Plugin-based architecture for custom service integrations
- **Multi-Platform**: Deploy on VMs, Docker, or Kubernetes
- **IPv6 Native**: Full IPv6 support for DNS zones and DHCP leases
- **Role-Based DNS Policies**: IP/subnet to role mappings for context-aware DNS filtering
- **Metrics & Monitoring**: Prometheus metrics, Grafana dashboards, and comprehensive health checks
- **High Availability**: Kea HA pairs with VIP failover and etcd-based state coordination
- **DB-Only Nodes**: Dedicated etcd replication nodes for improved quorum resilience
- **Multi-Source Threat Intelligence**: Support for multiple MISP instances with tag filtering and deduplication

## Quick Start

### Prerequisites

- Rust 1.70+ (for building the agent)
- Protocol Buffers compiler (protoc) - required for building protobuf files
  - Ubuntu/Debian: `sudo apt-get install protobuf-compiler libprotobuf-dev`
  - macOS: `brew install protobuf`
  - Other platforms: See [protobuf installation guide](https://grpc.io/docs/protoc-installation/)
- Docker & Docker Compose (for containerized deployments)
- etcd cluster (for production) or standalone (for development)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/nnoe/nnoe.git
cd nnoe

# Build the agent
cargo build --release

# Run the agent
./target/release/nnoe-agent --help
```

### Docker Compose (Development)

```bash
cd deployments/docker

# Create .env file with your configuration (see .env.example)
cp .env.example .env

# Start services
docker-compose -f docker-compose.dev.yml up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f agent
```

Key environment variables (see `deployments/docker/README.md` for full list):
- `ETCD_ENDPOINTS`: etcd cluster endpoints (default: `http://etcd:2379`)
- `NODE_NAME`: Unique agent node name
- `NODE_ROLE`: Agent role (`agent` or `db-only`)
- `LOG_LEVEL`: Logging level (`debug`, `info`, `warn`, `error`)

## Architecture

NNOE uses a control plane/data plane separation:
- **Control Plane**: etcd for config synchronization via Nebula overlay network
- **Data Plane**: DNS/DHCP services (Knot, Kea, dnsdist) serving clients on host interfaces

### Components

- **Management Nodes**: phpIPAM UI, etcd leader, Nebula lighthouses, MISP server
- **DB-Only Agents**: etcd followers with sled cache for replication/quorum
- **Active Agents**: Rust binaries managing Knot DNS, Kea DHCP, dnsdist, Cerbos, Lynis

See [docs/architecture.md](docs/architecture.md) for detailed architecture documentation.

## Documentation

- [Architecture](docs/architecture.md) - System design and component breakdown
- [Getting Started](docs/development/getting-started.md) - Development setup guide
- [Deployment Guides](docs/deployment/) - Docker, Kubernetes, Ansible, and manual installation
- [API Documentation](docs/api/) - Agent APIs and etcd schema
- [Contributing](docs/development/contributing.md) - How to contribute

## Project Status

**Overall Progress: ~85-90% Complete**

This project is under active development. Current status:
- ✅ Stage 1: Critical Security and Reliability Fixes (100%) - TLS support, HA coordination, Kea hooks
- ✅ Stage 2: Complete Service Implementations (100%) - All service plugins implemented
- ⚠️ Stage 3: Testing Infrastructure (~60%) - Integration tests in progress
- ✅ Stage 4: Documentation Updates (100%) - Documentation comprehensive and current
- ✅ Stage 5: Integration Components (100%) - phpIPAM, MISP sync, Kea hooks complete
- ✅ Stage 6: Management Components (100%) - etcd orchestrator, Nebula CA, monitoring
- ✅ Stage 7: Deployment Configurations (100%) - Docker, Kubernetes, Ansible, Manual
- ✅ Stage 8: CI/CD and Quality Assurance (66%) - Pipelines complete, dependency updates pending
- ⏳ Stage 9: Final Integration and Validation - E2E testing and security audit pending

See [docs/completion-status.md](docs/completion-status.md) for detailed completion status.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](docs/development/contributing.md) for guidelines.

