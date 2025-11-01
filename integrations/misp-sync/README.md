# MISP Sync Service

Service that periodically synchronizes threat intelligence from MISP (Malware Information Sharing Platform) to NNOE's etcd backend for DNS filtering.

## Features

- Periodic synchronization from MISP feeds
- Automatic threat domain extraction
- etcd KV storage for dnsdist RPZ
- Configurable sync intervals
- Severity classification

## Configuration

Set environment variables:

```bash
export MISP_URL="https://misp.example.com"
export MISP_API_KEY="your-api-key"
export ETCD_ENDPOINTS="http://127.0.0.1:2379"
export ETCD_PREFIX="/nnoe"
export SYNC_INTERVAL_SECS=3600
```

## Building

```bash
cd integrations/misp-sync
cargo build --release
```

## Running

```bash
./target/release/misp-sync
```

## Output

Threat domains are stored in etcd at:
- Key: `/nnoe/threats/domains/<domain>`
- Value: JSON with domain, source, severity, timestamp

These are automatically picked up by dnsdist service for RPZ blocking.

## Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY integrations/misp-sync .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/misp-sync /usr/local/bin/
ENTRYPOINT ["misp-sync"]
```

## Status

This service synchronizes MISP threat intelligence with NNOE's etcd backend for use by dnsdist.

