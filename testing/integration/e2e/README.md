# End-to-End Testing

End-to-end tests verify the complete NNOE system working together.

## Prerequisites

- Docker and Docker Compose
- etcd running (or use docker-compose)
- Required binaries built (`cargo build --release`)

## Running E2E Tests

### Using Docker Compose

```bash
cd testing/integration/e2e
docker-compose up --build
```

This will:
1. Start etcd container
2. Start mock MISP server
3. Run agent tests in container
4. Clean up on completion

### Manual Testing

1. Start etcd:
   ```bash
   docker run -d -p 2379:2379 quay.io/coreos/etcd:v3.5.9 etcd --listen-client-urls http://0.0.0.0:2379 --advertise-client-urls http://localhost:2379
   ```

2. Run tests:
   ```bash
   cargo test --test e2e_tests -- --nocapture
   ```

## Test Scenarios

### Zone Propagation Test
- Creates DNS zone in etcd
- Verifies agent receives change
- Verifies Knot zone file generation

### DHCP Scope Test
- Creates DHCP scope in etcd
- Verifies Kea config generation
- Verifies scope propagation

### Threat Blocking Test
- Adds threat domain via MISP sync
- Verifies dnsdist Lua script update
- Verifies DNS blocking

## Debugging

Enable debug logging:
```bash
RUST_LOG=nnoe_agent=debug cargo test --test e2e_tests
```

View etcd contents:
```bash
etcdctl --endpoints=http://127.0.0.1:2379 get --prefix /nnoe/test
```

