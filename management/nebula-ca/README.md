# Nebula CA Management Tools

Tools for managing Nebula Certificate Authority and node certificates.

## Tools

### ca-init.sh

Initialize a new Nebula CA.

```bash
./ca-init.sh [ca-name] [duration] [output-dir]
```

Example:
```bash
./ca-init.sh "NNOE CA" 87600h /etc/nebula/ca
```

### sign-cert.sh

Sign a certificate for a Nebula node.

```bash
./sign-cert.sh <node-name> <node-ip> [ca-crt] [ca-key] [output-dir] [duration] [groups]
```

Example:
```bash
./sign-cert.sh node-1 192.168.100.1
./sign-cert.sh lighthouse-1 192.168.100.1 "" "" "" "" "lighthouse"
```

### distribute-certs.sh

Distribute certificates to etcd for agent retrieval.

```bash
./distribute-certs.sh <node-name> <cert-file> <key-file> [etcd-endpoint] [etcd-prefix]
```

Example:
```bash
./distribute-certs.sh node-1 \
    /etc/nebula/certs/node-1.crt \
    /etc/nebula/certs/node-1.key \
    http://127.0.0.1:2379
```

### revoke-cert.sh

Revoke a certificate.

```bash
./revoke-cert.sh <node-name> [ca-crt] [ca-key] [crl-file] [etcd-endpoint] [etcd-prefix]
```

Example:
```bash
./revoke-cert.sh node-1
```

## Workflow

1. **Initialize CA** (one-time setup):
   ```bash
   ./ca-init.sh "NNOE CA"
   ```

2. **Sign certificates for nodes**:
   ```bash
   # Management node
   ./sign-cert.sh mgmt-1 192.168.100.1
   
   # Lighthouse
   ./sign-cert.sh lighthouse-1 192.168.100.2 "" "" "" "" "lighthouse"
   
   # Agent nodes
   ./sign-cert.sh agent-1 192.168.100.10
   ./sign-cert.sh agent-2 192.168.100.11
   ```

3. **Distribute to etcd**:
   ```bash
   ./distribute-certs.sh mgmt-1 \
       /etc/nebula/certs/mgmt-1.crt \
       /etc/nebula/certs/mgmt-1.key
   ```

4. **Agents retrieve from etcd** on startup

## Security Notes

- **CA Key**: Keep `ca.key` secure, never distribute
- **Node Keys**: Can be distributed via etcd (use TLS encryption in production)
- **Revocation**: Update CRL and mark revoked in etcd
- **Rotation**: Regenerate certificates before expiration

## Requirements

- Nebula (`nebula-cert` command)
- etcdctl (for certificate distribution)

