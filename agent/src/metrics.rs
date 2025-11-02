use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics tracking for the agent
pub struct AgentMetrics {
    /// Total number of config updates received
    pub config_updates_total: Arc<AtomicU64>,
    /// Total number of service reloads performed
    pub service_reloads_total: Arc<AtomicU64>,
    /// Total number of DNS queries processed (if available from dnsdist)
    pub dns_queries_total: Arc<AtomicU64>,
    /// Total number of blocked DNS queries
    pub blocked_queries_total: Arc<AtomicU64>,
    /// Total number of DHCP leases allocated
    pub dhcp_leases_total: Arc<AtomicU64>,
    /// Current number of active DHCP leases
    pub dhcp_leases_active: Arc<AtomicU64>,
}

impl AgentMetrics {
    pub fn new() -> Self {
        Self {
            config_updates_total: Arc::new(AtomicU64::new(0)),
            service_reloads_total: Arc::new(AtomicU64::new(0)),
            dns_queries_total: Arc::new(AtomicU64::new(0)),
            blocked_queries_total: Arc::new(AtomicU64::new(0)),
            dhcp_leases_total: Arc::new(AtomicU64::new(0)),
            dhcp_leases_active: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_config_updates(&self) {
        self.config_updates_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_service_reloads(&self) {
        self.service_reloads_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dns_queries(&self) {
        self.dns_queries_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_blocked_queries(&self) {
        self.blocked_queries_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_dhcp_leases(&self) {
        self.dhcp_leases_total.fetch_add(1, Ordering::Relaxed);
        self.dhcp_leases_active.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_dhcp_leases_active(&self) {
        self.dhcp_leases_active.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get_config_updates_total(&self) -> u64 {
        self.config_updates_total.load(Ordering::Relaxed)
    }

    pub fn get_service_reloads_total(&self) -> u64 {
        self.service_reloads_total.load(Ordering::Relaxed)
    }

    pub fn get_dns_queries_total(&self) -> u64 {
        self.dns_queries_total.load(Ordering::Relaxed)
    }

    pub fn get_blocked_queries_total(&self) -> u64 {
        self.blocked_queries_total.load(Ordering::Relaxed)
    }

    pub fn get_dhcp_leases_total(&self) -> u64 {
        self.dhcp_leases_total.load(Ordering::Relaxed)
    }

    pub fn get_dhcp_leases_active(&self) -> u64 {
        self.dhcp_leases_active.load(Ordering::Relaxed)
    }
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self::new()
    }
}

