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

#### HA Pairs

- **Path**: `/nnoe/dhcp/ha-pairs/<pair_id>/nodes/<node_name>/status`
- **Format**: JSON
- **Description**: High Availability pair node status tracking
- **TTL**: 60 seconds (auto-expires if not refreshed)
- **Example**:
```json
{
  "state": "Primary",
  "timestamp": "2025-01-15T10:30:00Z"
}
```
- **State Values**: `"Primary"`, `"Standby"`, `"Unknown"`

#### DHCP Leases

- **Path**: `/nnoe/dhcp/leases/<lease_id>` (IPv4) or `/nnoe/dhcp/leases/<lease_id>` (IPv6)
- **Format**: JSON (base64 encoded in etcd v3 API)
- **Note**: Keys and values are base64 encoded when stored via Kea hooks using etcd v3 API
- **IPv4 Lease Example**:
```json
{
  "ip": "192.168.1.100",
  "hwaddr": "aa:bb:cc:dd:ee:ff",
  "hostname": "client.example.com",
  "state": 0,
  "cltt": 1705315200,
  "valid_lft": 86400,
  "operation": "offer",
  "timestamp": 1705315200,
  "expires_at": 1705401600
}
```
- **IPv6 Lease Example**:
```json
{
  "ip": "2001:db8::1",
  "type": 1,
  "iaid": 12345,
  "duid": "00:01:00:01:1a:2b:3c:4d:5e:6f",
  "state": 0,
  "cltt": 1705315200,
  "valid_lft": 86400,
  "preferred_lft": 3600,
  "operation": "offer",
  "timestamp": 1705315200,
  "expires_at": 1705401600
}
```
- **Fields**:
  - `ip`: IP address (IPv4 or IPv6)
  - `operation`: Lease event type (`"offer"`, `"renew"`, `"release"`, `"expire"`)
  - `expires_at`: Unix timestamp when lease expires (calculated from `cltt` + `valid_lft`)
  - IPv6-specific: `type` (IA_NA, IA_PD), `iaid`, `duid`, `preferred_lft`

### Policies

- **Path**: `/nnoe/policies/<policy-id>`
- **Format**: JSON (Cerbos-compatible policy definition)

### Role Mappings

- **Path**: `/nnoe/role-mappings/<ip_or_subnet>`
- **Format**: JSON
- **Description**: IP address or subnet CIDR to role mappings for DNS policy evaluation
- **Example**:
```json
{
  "roles": ["iot", "guest", "untrusted"]
}
```
- **Usage**: Used by dnsdist service to extract client roles for Cerbos policy evaluation
- **Note**: IP addresses can be individual IPs (e.g., `192.168.1.10`) or subnets (e.g., `192.168.1.0/24`)

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
- `/nnoe/role-mappings` - Role mapping updates (IP/subnet to roles)

When a change is detected (Put or Delete event), agents:
1. Parse the etcd event
2. Extract the key and value
3. Notify registered service plugins via `on_config_change()` method
4. Plugins handle the update (e.g., regenerate configs, reload services)

## Versioning

The schema may evolve. Future versions will include:

- Schema version in metadata
- Migration scripts
- Backward compatibility guarantees

## Storage Encoding

### Base64 Encoding (etcd v3 API)

The Kea DHCP hooks use base64 encoding for etcd v3 API storage:

- **Keys**: Base64 encoded before storage (e.g., `/nnoe/dhcp/leases/192.168.1.100`)
- **Values**: Base64 encoded JSON payload
- **Implementation**: Uses OpenSSL BIO functions for robust base64 encoding
- **Purpose**: Required by etcd v3 API for binary-safe storage

When reading from etcd via agent:
- Keys and values are automatically decoded by etcd-client library
- Direct etcd client usage returns decoded data

## Agent Health and Metrics

- **Path**: `/nnoe/agents/health/<node_name>` (if implemented)
- **Path**: `/nnoe/agents/metrics/<node_name>` (if stored in etcd)
- **Format**: JSON
- **Note**: Metrics are primarily exposed via Prometheus exporter on port 9090

## Status

This schema documentation reflects the current implementation. All key patterns and data structures documented above are actively used in production code.

