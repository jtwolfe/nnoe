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
  - `new(config)`: Initialize orchestrator with configuration
  - `run()`: Start the main event loop
  - `register_services()`: Register service plugins
  - `watch_config_changes()`: Start etcd watch loops

### Etcd Client

The etcd client provides a wrapper around etcd operations.

- **Location**: `agent/src/etcd/client.rs`
- **Key Methods**:
  - `get(key)`: Retrieve value by key
  - `put(key, value)`: Store key-value pair
  - `delete(key)`: Delete key
  - `list_prefix(prefix)`: List all keys with prefix
  - `watch(prefix)`: Watch for changes to keys with prefix

### Cache Manager

The cache manager provides local caching using sled.

- **Location**: `agent/src/sled_cache/cache.rs`
- **Key Methods**:
  - `get(key)`: Retrieve cached value
  - `put(key, value)`: Cache value
  - `delete(key)`: Remove from cache
  - `list_prefix(prefix)`: List cached keys with prefix
  - `clear()`: Clear all cached data

### Plugin System

The plugin system allows extensible service integration.

- **Trait**: `agent/src/plugin/trait_def.rs`
- **Registry**: `agent/src/plugin/registry.rs`
- **Service plugins** must implement the `ServicePlugin` trait

## Status

This documentation will be expanded as the project evolves. Service-specific APIs will be documented in Phase 2.

