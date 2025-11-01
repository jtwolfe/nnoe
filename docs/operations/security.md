# Security Guide

Security best practices for NNOE deployments.

## etcd Security

### TLS Encryption

Enable TLS for etcd:

```toml
[etcd]
endpoints = ["https://etcd-server:2379"]
tls = { ca_cert = "/etc/nnoe/certs/ca.crt",
        cert = "/etc/nnoe/certs/client.crt",
        key = "/etc/nnoe/certs/client.key" }
```

### Authentication

Use etcd authentication:

```bash
# Create root user
etcdctl user add root

# Enable authentication
etcdctl auth enable

# Create agent user
etcdctl user add agent
etcdctl role add agent-role
etcdctl role grant-permission agent-role readwrite --prefix /nnoe
etcdctl user grant-role agent agent-role
```

### Network Security

- Restrict etcd ports (2379, 2380) to internal networks
- Use firewall rules
- Enable network policies in Kubernetes

## Nebula Overlay

### Certificate Management

- Use strong CA key
- Rotate certificates regularly
- Revoke compromised certificates immediately

### Network Isolation

- Isolate control plane traffic via Nebula
- Use separate networks for data plane
- Implement firewall rules

## Service Security

### DNS Security

**DNSSEC:**
- Enable DNSSEC signing in Knot
- Rotate keys regularly
- Monitor key expiration

**Rate Limiting:**
- Implement query rate limits
- Use dnsdist for DDoS protection

### DHCP Security

**Lease Validation:**
- Validate MAC addresses
- Implement lease duration limits
- Monitor for rogue DHCP servers

### Access Control

- Use Cerbos for policy enforcement
- Implement role-based access
- Audit policy changes

## Data Protection

### Encryption at Rest

- Encrypt etcd data volumes
- Use encrypted filesystems
- Protect backup files

### Encryption in Transit

- TLS for all etcd communication
- TLS for API endpoints
- Secure certificate distribution

## Authentication & Authorization

### API Authentication

- Use API keys for phpIPAM API
- Implement rate limiting
- Validate all inputs

### etcd ACLs

```bash
# Grant read-only access
etcdctl role grant-permission read-only read --prefix /nnoe

# Grant write access to specific paths
etcdctl role grant-permission write-role write /nnoe/dns/zones
```

## Audit Logging

### Enable Audit Logs

Log all configuration changes:

```bash
# Audit etcd operations
etcdctl watch /nnoe --prefix

# Log to file
journalctl -u nnoe-agent > /var/log/nnoe/audit.log
```

### Log Retention

- Retain logs for compliance period
- Rotate logs regularly
- Secure log storage

## Hardening

### System Hardening

- Run as non-root user (nnoe)
- Limit file permissions
- Use systemd security settings

### Container Security

- Use non-root containers
- Scan images for vulnerabilities
- Limit container capabilities

### Network Hardening

- Use network policies
- Implement segmentation
- Restrict unnecessary ports

## Compliance

### Regular Updates

- Keep software updated
- Monitor security advisories
- Apply patches promptly

### Vulnerability Scanning

- Scan containers regularly
- Check dependencies
- Review security reports

### Access Reviews

- Review access regularly
- Remove unnecessary access
- Document access changes

## Incident Response

### Detection

- Monitor for anomalies
- Set up alerts
- Review logs regularly

### Response Plan

1. Identify affected systems
2. Isolate compromised components
3. Preserve evidence
4. Remediate issues
5. Review and improve

### Recovery

- Test backup/restore procedures
- Document recovery steps
- Keep recovery tools ready

## Best Practices

1. **Least Privilege:** Grant minimum necessary access
2. **Defense in Depth:** Multiple security layers
3. **Regular Updates:** Keep software current
4. **Monitor Continuously:** Watch for anomalies
5. **Encrypt Sensitive Data:** Both at rest and in transit
6. **Audit Regularly:** Review logs and access
7. **Test Security:** Regular security testing
8. **Document Procedures:** Security runbooks

## Security Checklist

- [ ] etcd TLS enabled
- [ ] etcd authentication configured
- [ ] Nebula certificates secure
- [ ] Firewall rules configured
- [ ] Non-root user for agent
- [ ] Logging enabled
- [ ] Audit trails active
- [ ] Backups encrypted
- [ ] Updates applied
- [ ] Monitoring configured

