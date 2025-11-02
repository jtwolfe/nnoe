# Getting Started with NNOE Development

## Prerequisites

- **Rust**: 1.82 or later / stable ([Install Rust](https://www.rust-lang.org/tools/install))
- **Protocol Buffers compiler (protoc)**: Required for building protobuf files used by the agent
  - Ubuntu/Debian: `sudo apt-get install protobuf-compiler libprotobuf-dev`
  - macOS: `brew install protobuf`
  - Other platforms: See [protobuf installation guide](https://grpc.io/docs/protoc-installation/)
- **Docker** and **Docker Compose** (for integration testing)
- **etcd**: v3.5+ (for local development)

## Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/nnoe/nnoe.git
cd nnoe
```

### 2. Install Dependencies

```bash
# Install Rust toolchain
rustup toolchain install stable

# Install development tools
rustup component add rustfmt clippy

# Install Protocol Buffers compiler (if not already installed)
# Ubuntu/Debian:
sudo apt-get install protobuf-compiler libprotobuf-dev
# macOS:
# brew install protobuf
```

### 3. Build the Project

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### 5. Local etcd Setup

For local development, you can run etcd in Docker:

```bash
docker run -d \
  --name etcd \
  -p 2379:2379 \
  -p 2380:2380 \
  quay.io/coreos/etcd:v3.5.9 \
  etcd \
  --name etcd-server \
  --data-dir /etcd-data \
  --listen-client-urls http://0.0.0.0:2379 \
  --advertise-client-urls http://localhost:2379 \
  --listen-peer-urls http://0.0.0.0:2380 \
  --initial-advertise-peer-urls http://localhost:2380 \
  --initial-cluster etcd-server=http://localhost:2380 \
  --initial-cluster-token etcd-cluster-1 \
  --initial-cluster-state new
```

### 6. Run the Agent

Create a configuration file (see `agent/examples/agent.toml.example`):

```bash
mkdir -p /etc/nnoe
cp agent/examples/agent.toml.example /etc/nnoe/agent.toml
# Edit the configuration as needed
```

Run the agent:

```bash
cargo run --bin nnoe-agent -- run -c /etc/nnoe/agent.toml
```

## Project Structure

```
nnoe/
├── agent/              # Rust agent binary
│   ├── src/
│   │   ├── core/      # Orchestration logic
│   │   ├── etcd/      # etcd client
│   │   ├── sled_cache/ # Local cache
│   │   ├── nebula/    # Nebula integration
│   │   ├── plugin/    # Plugin system
│   │   └── services/  # Service integrations (Phase 2)
│   └── Cargo.toml
├── integrations/      # External service integrations
├── management/         # Management node components
├── deployments/        # Deployment configs
├── testing/           # Test infrastructure
└── docs/              # Documentation
```

## Development Workflow

1. **Create a feature branch**: `git checkout -b feature/my-feature`
2. **Make changes** and ensure tests pass
3. **Run linters**: `cargo clippy` and `cargo fmt`
4. **Commit changes**: Follow the project's commit message conventions
5. **Push and create PR**: Open a pull request for review

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Fix clippy warnings: `cargo clippy -- -D warnings`
- Document public APIs with doc comments

## Testing

- **Unit tests**: Co-located with source code using `#[cfg(test)]`
- **Integration tests**: In `testing/integration/`
- **E2E tests**: In `testing/integration/e2e/` (Phase 4)

## Debugging

Enable debug logging:

```bash
RUST_LOG=nnoe_agent=debug cargo run --bin nnoe-agent
```

Or use the `--debug` flag:

```bash
cargo run --bin nnoe-agent -- --debug run
```

## Next Steps

- Read [Contributing Guide](contributing.md)
- Check [Architecture Documentation](../architecture.md)
- Review [Plugin Development Guide](plugin-development.md)

