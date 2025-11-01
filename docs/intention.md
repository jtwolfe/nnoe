# Intention for the Final Product: New Network Orchestration Engine (NNOE)

## Executive Summary

The New Network Orchestration Engine (NNOE) is envisioned as a comprehensive, open-source Distributed DNS, DHCP, and IP Address Management (DDI) platform designed to provide enterprise-grade network services with a focus on security, scalability, and flexibility. Drawing inspiration from proprietary solutions like BlueCat DNS Integrity, NNOE aims to democratize advanced DDI capabilities by leveraging modular, lightweight open-source components. The project's core intention is to empower organizations—ranging from small networks to large-scale enterprises—to manage their core network infrastructure without vendor lock-in, high costs, or proprietary dependencies. By November 2025, with evolving open-source trends emphasizing zero-trust security and hybrid deployments, NNOE positions itself as a resilient alternative that integrates seamlessly across VMs, Docker containers, and Kubernetes environments.

The final product will deliver a unified system for automating DNS resolution, DHCP leasing, IP allocation, and threat protection, while ensuring high availability (HA), real-time synchronization, and auditable operations. Unlike commercial tools, NNOE prioritizes community-driven extensibility, minimal resource footprints, and integration with modern orchestration practices, ultimately reducing operational complexity and enhancing network resilience.

## Project Background and Motivation

In an era where networks are increasingly hybrid (on-premises, cloud, edge), traditional DDI solutions like BlueCat Integrity have set benchmarks for centralized management, adaptive security, and multi-cloud integrations. However, these come with challenges: high licensing fees, proprietary ecosystems, and limited customization. Open-source alternatives, such as NetBox or phpIPAM for IPAM, and tools like Knot DNS or Kea for services, have emerged to address these gaps, but often lack unified orchestration and advanced features like pre-cache DNS visibility or automated threat aggregation.

NNOE's intention stems from this landscape: to create a "BlueCat-like" experience through a composable stack that combines best-of-breed OSS tools. The project evolved from initial explorations of BlueCat's features (e.g., distributed databases, HA pools) to a refined architecture incorporating user feedback for lightweight agents, NAT-traversing overlays, and internal runtime operations. The goal is not mere replication but innovation—enabling features like embedded Raft consensus for fault tolerance and zero-touch policy updates, all while aligning with 2025 trends in edge computing and zero-trust networking.

## Core Goals and Objectives

The final product intends to achieve the following:

1. **Accessibility and Cost-Effectiveness**: As a fully open-source solution (licensed under Apache 2.0 or similar), NNOE eliminates barriers for adoption. Organizations can deploy it without subscriptions, fostering community contributions for ongoing enhancements.

2. **Security and Resilience**: Incorporate BlueCat-inspired features like DNS query anomaly detection, granular policies, and threat intelligence integration to mitigate risks such as DDoS, exfiltration, or phishing. Ensure HA through distributed replication and failover, surviving node losses without downtime.

3. **Scalability and Flexibility**: Support massive scales (e.g., millions of queries per second) across deployment models. Modular components allow mixing environments, with agents handling local operations independently.

4. **Ease of Use and Automation**: Provide a intuitive UI for management, while automating runtime tasks internally (e.g., config propagation via watches). Reduce manual intervention, aligning with DevOps practices.

5. **Interoperability and Extensibility**: Integrate with existing tools (e.g., cloud providers via APIs) and allow custom extensions, such as plugins for emerging protocols.

By focusing on these, NNOE aims to outperform fragmented OSS setups (e.g., BIND + ISC DHCP + phpIPAM) in cohesion, while rivaling commercial alternatives like Infoblox NIOS or EfficientIP in functionality.

## Key Features of the Final Product

- **Unified DDI Management**: Centralized oversight via phpIPAM UI for IPAM, DNS zones, DHCP scopes, and policies. Supports IPv6 natively, with auto-discovery and reporting.

- **Distributed Architecture**: etcd for consensus-based config sync across management nodes, DB-only replicas, and active agents. sled embedded in agents for fast local reads/caching.

- **Service Delivery**: Knot DNS for high-performance authoritative/recursive queries (DNSSEC-enabled); Kea for dynamic DHCPv4/v6 leasing with HA pairs; dnsdist for proxying, filtering, and anomaly detection.

- **Security Enhancements**: Cerbos for context-aware policies (e.g., time/device-based access); MISP for automated threat feed aggregation and RPZ blocklists; Lynis for compliance auditing across Unix-like systems (containerized for Windows).

- **Overlay Networking**: Nebula for secure, NAT-traversing control plane communications, ensuring privacy and auditability without exposing internal traffic.

- **Runtime Autonomy**: Post-deployment, all operations (e.g., policy updates, failover) handled internally via etcd watches and agent logic—no reliance on external orchestrators like Ansible/K8s during runtime.

These features mirror BlueCat's strengths (e.g., adaptive DNS, edge visibility) but extend them with OSS modularity, such as Rust-based agents for minimal footprints (~2-5MB binaries).

## Target Users and Use Cases

- **Enterprises**: Large organizations managing hybrid networks, seeking cost savings over BlueCat while maintaining features like multi-site HA and threat protection.

- **Service Providers**: ISPs or cloud operators needing scalable DDI for tenants, with customizable policies.

- **DevOps Teams**: Teams in agile environments, integrating NNOE with CI/CD for automated provisioning.

Use cases include: Secure DNS resolution in zero-trust setups; Dynamic IP allocation in containerized apps; Real-time threat blocking in edge networks.

## Roadmap and Future Vision

The final product, targeted for release by mid-2026, will include beta deployments with community testing. Future iterations may add ML-based anomaly detection, deeper cloud integrations (e.g., AWS Route 53 sync), and support for emerging standards like DNS over QUIC. The intention is to build a thriving ecosystem, encouraging forks and contributions to evolve NNOE beyond BlueCat's proprietary limits.

In summary, NNOE's intention is to redefine open-source DDI as a secure, efficient, and accessible foundation for modern networks, bridging the gap between proprietary excellence and community-driven innovation.