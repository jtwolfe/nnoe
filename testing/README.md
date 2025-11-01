# NNOE Testing Infrastructure

Comprehensive testing infrastructure for NNOE project.

## Test Structure

```
testing/
├── unit/              # Unit tests
├── integration/       # Integration tests
│   ├── fixtures/     # Test fixtures and data
│   ├── scenarios/    # Test scenarios
│   └── e2e/          # End-to-end tests
├── performance/      # Performance tests
│   ├── benchmarks/   # Rust benchmarks
│   └── scripts/      # Load testing scripts
└── mocks/            # Mock services
```

## Running Tests

### Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run specific test module
cargo test --lib config_test

# Run with output
cargo test --lib -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'

# Run specific scenario
cargo test --test zone_management_test
```

### E2E Tests

```bash
cd testing/integration/e2e
docker-compose up --build
```

### Performance Tests

```bash
# Run benchmarks
cd testing/performance/benchmarks
cargo bench

# Run load tests
./testing/performance/scripts/load_test.sh
```

## Test Coverage

Target: >80% code coverage

Generate coverage report:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

## Mock Services

Mock services are provided for isolated testing:

- **etcd-mock**: In-memory etcd server for testing
- **knot-mock**: (Placeholder for Knot DNS mock)
- **kea-mock**: (Placeholder for Kea DHCP mock)

## Continuous Integration

Tests run automatically on:
- Pull requests
- Pushes to main/develop
- Pre-commit hooks (optional)

See `.github/workflows/test.yml` for CI configuration.

## Writing Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_integration() {
    // Setup
    let client = create_test_client().await;
    
    // Test
    let result = client.operation().await;
    
    // Verify
    assert!(result.is_ok());
}
```

## Test Data

Test fixtures are in `testing/integration/fixtures/`:
- `test_config.toml`: Example agent configuration
- `misp-data/`: Mock MISP API responses

## Performance Targets

- DNS query latency: <10ms (p99)
- Config propagation: <100ms
- etcd watch latency: <50ms
- Throughput: >1M QPS (DNS queries)

