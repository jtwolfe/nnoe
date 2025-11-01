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

## Quick Start

### Prerequisites

- Rust 1.70+ (for building the agent)
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
docker-compose -f docker-compose.dev.yml up -d
```

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

This project is under active development. Current status:
- ✅ Phase 1: Foundation & Core Agent (In Progress)
- ⏳ Phase 2: Service Integrations
- ⏳ Phase 3: Management Components
- ⏳ Phase 4: Testing Infrastructure
- ⏳ Phase 5: Deployment Configurations
- ⏳ Phase 6: Documentation

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](docs/development/contributing.md) for guidelines.

