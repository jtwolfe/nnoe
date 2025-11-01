# Kea DHCP Hooks for NNOE

This directory contains custom Kea hooks for integrating with NNOE's etcd backend.

## libdhcp_etcd.so Hook

This hook synchronizes DHCP lease information with etcd for centralized tracking.

### Features

- Lease assignment events → etcd KV store
- Lease renewal events → etcd updates
- Lease expiration events → etcd cleanup
- Integration with Kea's lease database

### Status

This hook will be implemented in Phase 3 as it requires C++ development for Kea hook API.

### Usage

```json
{
  "Dhcp4": {
    "hooks-libraries": [
      {
        "library": "/usr/lib/kea/hooks/libdhcp_etcd.so",
        "parameters": {
          "etcd_endpoints": ["http://127.0.0.1:2379"],
          "prefix": "/nnoe/dhcp/leases",
          "ttl": 3600
        }
      }
    ]
  }
}
```

