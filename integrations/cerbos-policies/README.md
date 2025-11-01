# Cerbos Policy Templates for NNOE

This directory contains example Cerbos policies for DNS query filtering and access control.

## Example Policies

### dns-query-policy.yaml

Basic policy for DNS query access control based on time and device type.

```yaml
apiVersion: api.cerbos.dev/v1
resourcePolicy:
  version: default
  resource: dns_query
  rules:
    - actions: ['allow']
      effect: EFFECT_ALLOW
      roles: ['iot']
      condition:
        match:
          expr: request.time.hour < 18
    - actions: ['allow']
      effect: EFFECT_ALLOW
      roles: ['admin']
      condition:
        match:
          expr: true
```

### network-segment-policy.yaml

Policy for network segment-based DNS filtering.

```yaml
apiVersion: api.cerbos.dev/v1
resourcePolicy:
  version: default
  resource: dns_query
  rules:
    - actions: ['allow']
      effect: EFFECT_ALLOW
      roles: ['internal']
      condition:
        match:
          expr: request.network_segment == "internal"
    - actions: ['deny']
      effect: EFFECT_DENY
      roles: ['guest']
      condition:
        match:
          expr: request.domain.contains("internal")
```

## Deployment

1. Upload policies to etcd at `/nnoe/policies/`
2. Agent will sync policies to Cerbos via gRPC
3. dnsdist will query Cerbos for policy decisions

## Status

Full Cerbos integration will be implemented in Phase 2 service integrations.

