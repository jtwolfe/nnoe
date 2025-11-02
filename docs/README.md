# NNOE Documentation

Comprehensive documentation for the New Network Orchestration Engine (NNOE).

## Quick Links

- [Architecture Overview](architecture.md) - System design and component breakdown
- [Project Intention](intention.md) - Project goals and vision
- [Getting Started](development/getting-started.md) - Development setup guide

## Documentation Structure

### API Documentation

- [Agent API](api/agent-api.md) - Agent command-line and internal APIs
- [etcd Schema](api/etcd-schema.md) - etcd key-value schema and data formats
- [phpIPAM Extensions](api/phpipam-extensions.md) - phpIPAM plugin API

### Deployment Guides

- [Docker Deployment](deployment/docker.md) - Docker Compose deployment
- [Kubernetes Deployment](deployment/kubernetes.md) - Kubernetes manifests and deployment
- [Ansible Deployment](deployment/ansible.md) - Automated deployment with Ansible
- [Manual Installation](deployment/manual.md) - Manual installation on bare metal/VMs

### Development Guides

- [Getting Started](development/getting-started.md) - Setup development environment
- [Contributing](development/contributing.md) - Contribution guidelines
- [Plugin Development](development/plugin-development.md) - Creating custom plugins

### Operational Documentation

- [Troubleshooting](operations/troubleshooting.md) - Common issues and solutions
- [Monitoring](operations/monitoring.md) - Monitoring and observability
- [Security](operations/security.md) - Security best practices
- [Backup and Restore](operations/backup-restore.md) - Backup and restore procedures

### Examples

- [Zone Management](examples/zone-management/README.md) - DNS zone management examples (includes DNSSEC)
- [DHCP Scopes](examples/dhcp-scopes/README.md) - DHCP scope management examples (includes IPv6 and HA pairs)
- [Threat Integration](examples/threat-integration/README.md) - Threat intelligence integration (includes multiple MISP instances and tag filtering)

## Documentation by Use Case

### I want to...

**Deploy NNOE:**
- Start with [Docker Deployment](deployment/docker.md) for quick setup
- Use [Kubernetes Deployment](deployment/kubernetes.md) for production
- Follow [Manual Installation](deployment/manual.md) for bare metal

**Develop for NNOE:**
- Read [Getting Started](development/getting-started.md)
- Learn [Plugin Development](development/plugin-development.md)
- Review [Contributing Guidelines](development/contributing.md)

**Operate NNOE:**
- Check [Troubleshooting Guide](operations/troubleshooting.md) for issues
- Set up [Monitoring](operations/monitoring.md)
- Implement [Security Best Practices](operations/security.md)

**Understand NNOE:**
- Read [Architecture Overview](architecture.md)
- Review [Project Intention](intention.md)
- Explore [API Documentation](api/)

## Documentation Standards

- **Markdown Format**: All docs use Markdown
- **Code Examples**: Include working examples
- **Diagrams**: Architecture diagrams in PlantUML/Mermaid
- **Versioning**: Docs updated with code changes

## Contributing to Documentation

1. Follow existing documentation style
2. Include examples where possible
3. Keep documentation current with code
4. Test all commands and examples
5. Add diagrams for complex concepts

## Documentation Status

- ✅ Architecture documentation
- ✅ API documentation
- ✅ Deployment guides
- ✅ Development guides
- ✅ Operational documentation
- ✅ Examples

See [Completion Status](completion-status.md) for detailed project implementation status and remaining work.

## Additional Resources

- [GitHub Repository](https://github.com/nnoe/nnoe)
- [Issue Tracker](https://github.com/nnoe/nnoe/issues)
- [Discussions](https://github.com/nnoe/nnoe/discussions)

