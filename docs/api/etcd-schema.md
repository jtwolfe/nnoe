# etcd KV Schema Documentation

## Overview

NNOE uses etcd as the distributed configuration store. This document describes the key-value schema and data formats.

## Key Prefixes

All NNOE keys are prefixed with `/nnoe` by default (configurable).

### DNS Configuration

#### Zones

- **Path**: `/nnoe/dns/zones/<zone-name>`
- **Format**: JSON
- **Example**:
```json
{
  "domain": "example.com",
  "ttl": 3600,
  "records": [
    {
      "name": "@",
      "type": "A",
      "value": "192.168.1.1"
    }
  ]
}
```

#### Zone Files

- **Path**: `/nnoe/dns/zones/<zone-name>/zonefile`
- **Format**: Plain text (standard DNS zone file format)

### DHCP Configuration

#### Scopes

- **Path**: `/nnoe/dhcp/scopes/<scope-id>`
- **Format**: JSON
- **Example**:
```json
{
  "subnet": "192.168.1.0/24",
  "pool": {
    "start": "192.168.1.100",
    "end": "192.168.1.200"
  },
  "gateway": "192.168.1.1",
  "options": {
    "router": "192.168.1.1",
    "dns-servers": ["192.168.1.1"]
  }
}
```

### Policies

- **Path**: `/nnoe/policies/<policy-id>`
- **Format**: JSON (Cerbos-compatible policy definition)

### Threat Intelligence

#### Domain Threats

- **Path**: `/nnoe/threats/domains/<domain>`
- **Format**: JSON
- **Example**:
```json
{
  "domain": "malicious.example.com",
  "source": "MISP",
  "severity": "high",
  "timestamp": "2025-01-01T00:00:00Z"
}
```

### Nebula Certificates

- **Path**: `/nnoe/nebula/certs/<node-name>`
- **Format**: PEM certificate

### Audit Logs

- **Path**: `/nnoe/audit/logs/<timestamp>-<node-id>`
- **Format**: JSON
- **Example**:
```json
{
  "timestamp": "2025-01-01T00:00:00Z",
  "node": "nnoe-node-1",
  "action": "config_updated",
  "resource": "/nnoe/dns/zones/example.com",
  "result": "success"
}
```

### Lynis Reports

- **Path**: `/nnoe/audit/lynis/<node-id>`
- **Format**: JSON (parsed Lynis report)

## Watch Patterns

Agents watch the following prefixes for real-time updates:

- `/nnoe/dns/zones` - DNS zone changes
- `/nnoe/dhcp/scopes` - DHCP scope changes
- `/nnoe/policies` - Policy updates
- `/nnoe/threats` - Threat intelligence updates

## Versioning

The schema may evolve. Future versions will include:

- Schema version in metadata
- Migration scripts
- Backward compatibility guarantees

## Status

This schema documentation will be expanded as the project evolves. Detailed schemas for each resource type will be documented in future phases.

