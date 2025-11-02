use anyhow::{Context, Result};
use etcd_client::Client;
use prometheus::{Gauge, Opts, Registry, Counter};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

pub struct MetricsCollector {
    pub registry: Registry,
    agent_uptime: Gauge,
    etcd_connected: Gauge, // Changed from connections to connected (boolean)
    cache_size_bytes: Gauge,
    cache_size_entries: Gauge,
    config_updates_total: Counter, // Changed to Counter for cumulative updates
    service_reloads_total: Counter, // Changed to Counter
    dns_queries_total: Counter,
    dns_query_rate: Gauge,
    dhcp_leases_active: Gauge,
    dhcp_leases_total: Counter,
    blocked_queries_total: Counter,
    ha_state: Gauge, // 0=Unknown, 1=Primary, 2=Standby
    // etcd endpoint configuration (optional, for direct connection)
    etcd_endpoints: Option<Vec<String>>,
    etcd_prefix: Option<String>,
}

impl MetricsCollector {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let agent_uptime = Gauge::with_opts(Opts::new(
            "nnoe_agent_uptime_seconds",
            "Agent uptime in seconds",
        ))?;
        registry.register(Box::new(agent_uptime.clone()))?;

        let etcd_connected = Gauge::with_opts(Opts::new(
            "nnoe_agent_etcd_connected",
            "etcd connection status (1=connected, 0=disconnected)",
        ))?;
        registry.register(Box::new(etcd_connected.clone()))?;

        let cache_size_bytes = Gauge::with_opts(Opts::new(
            "nnoe_agent_cache_size_bytes",
            "Cache size in bytes",
        ))?;
        registry.register(Box::new(cache_size_bytes.clone()))?;

        let cache_size_entries = Gauge::with_opts(Opts::new(
            "nnoe_agent_cache_size_entries",
            "Number of cache entries",
        ))?;
        registry.register(Box::new(cache_size_entries.clone()))?;

        let config_updates_total = Counter::with_opts(Opts::new(
            "nnoe_agent_config_updates_total",
            "Total config updates",
        ))?;
        registry.register(Box::new(config_updates_total.clone()))?;

        let service_reloads_total = Counter::with_opts(Opts::new(
            "nnoe_agent_service_reloads_total",
            "Total service reloads",
        ))?;
        registry.register(Box::new(service_reloads_total.clone()))?;

        let dns_queries_total = Counter::with_opts(Opts::new(
            "nnoe_dns_queries_total",
            "Total DNS queries processed",
        ))?;
        registry.register(Box::new(dns_queries_total.clone()))?;

        let dns_query_rate =
            Gauge::with_opts(Opts::new("nnoe_dns_query_rate", "DNS queries per second"))?;
        registry.register(Box::new(dns_query_rate.clone()))?;

        let dhcp_leases_active = Gauge::with_opts(Opts::new(
            "nnoe_dhcp_leases_active",
            "Current active DHCP lease count",
        ))?;
        registry.register(Box::new(dhcp_leases_active.clone()))?;

        let dhcp_leases_total = Counter::with_opts(Opts::new(
            "nnoe_dhcp_leases_total",
            "Total DHCP leases allocated",
        ))?;
        registry.register(Box::new(dhcp_leases_total.clone()))?;

        let blocked_queries_total = Counter::with_opts(Opts::new(
            "nnoe_blocked_queries_total",
            "Total blocked DNS queries",
        ))?;
        registry.register(Box::new(blocked_queries_total.clone()))?;

        let ha_state = Gauge::with_opts(Opts::new(
            "nnoe_agent_ha_state",
            "HA state (0=Unknown, 1=Primary, 2=Standby)",
        ))?;
        registry.register(Box::new(ha_state.clone()))?;

        Ok(Self {
            registry,
            agent_uptime,
            etcd_connected,
            cache_size_bytes,
            cache_size_entries,
            config_updates_total,
            service_reloads_total,
            dns_queries_total,
            dns_query_rate,
            dhcp_leases_active,
            dhcp_leases_total,
            blocked_queries_total,
            ha_state,
            etcd_endpoints: None,
            etcd_prefix: None,
        })
    }

    /// Configure etcd connection for metrics collection
    pub fn set_etcd_config(&mut self, endpoints: Vec<String>, prefix: String) {
        self.etcd_endpoints = Some(endpoints);
        self.etcd_prefix = Some(prefix);
    }

    pub async fn collect_metrics_loop(&self) {
        let mut interval_timer = interval(Duration::from_secs(10));
        let start_time = std::time::SystemTime::now();
        let mut last_dns_queries = 0u64;
        let mut last_collection = std::time::SystemTime::now();

        loop {
            interval_timer.tick().await;

            // Update uptime
            if let Ok(elapsed) = start_time.elapsed() {
                self.agent_uptime.set(elapsed.as_secs() as f64);
            }

            // Collect metrics from etcd if configured
            if let (Some(ref endpoints), Some(ref prefix)) = (&self.etcd_endpoints, &self.etcd_prefix) {
                if let Err(e) = self.collect_from_etcd(endpoints, prefix).await {
                    warn!("Failed to collect metrics from etcd: {}", e);
                    self.etcd_connected.set(0.0);
                } else {
                    self.etcd_connected.set(1.0);
                }
            } else {
                // No etcd configured, mark as disconnected
                self.etcd_connected.set(0.0);
            }

            // Calculate DNS query rate
            let current_dns_queries = self.dns_queries_total.get() as u64;
            let now = std::time::SystemTime::now();
            if let Ok(elapsed) = now.duration_since(last_collection) {
                if elapsed.as_secs() > 0 {
                    let queries_diff = current_dns_queries.saturating_sub(last_dns_queries);
                    let rate = queries_diff as f64 / elapsed.as_secs() as f64;
                    self.dns_query_rate.set(rate);
                }
            }
            last_dns_queries = current_dns_queries;
            last_collection = now;

            debug!("Metrics collection cycle completed");
        }
    }

    /// Collect metrics from etcd
    async fn collect_from_etcd(&self, endpoints: &[String], prefix: &str) -> Result<()> {
        // Connect to etcd
        let client = etcd_client::Client::connect(endpoints, None)
            .await
            .context("Failed to connect to etcd")?;

        let mut kv_client = client.kv_client();

        // Collect DHCP lease count from etcd
        let lease_prefix = format!("{}/dhcp/leases", prefix);
        let lease_resp = kv_client
            .get(&lease_prefix, Some(etcd_client::GetOptions::new().with_prefix()))
            .await?;
        let lease_count = lease_resp.kvs().len();
        self.dhcp_leases_active.set(lease_count as f64);

        // Try to read config updates count (stored in etcd as metric)
        // Format: /nnoe/metrics/config_updates_total
        let metrics_key = format!("{}/metrics/config_updates_total", prefix);
        if let Ok(resp) = kv_client.get(&metrics_key, None).await {
            if let Some(kv) = resp.kvs().first() {
                if let Ok(value_str) = std::str::from_utf8(kv.value()) {
                    if let Ok(value) = value_str.parse::<u64>() {
                        // Note: Counter doesn't have set(), but we can track incrementally
                        // For now, we'll track this differently or use a different approach
                        debug!("Config updates from etcd: {}", value);
                    }
                }
            }
        }

        // Collect HA state from etcd (if available)
        // Format: /nnoe/dhcp/ha-pairs/{pair_id}/nodes/{node_name}/status
        let ha_prefix = format!("{}/dhcp/ha-pairs", prefix);
        if let Ok(ha_resp) = kv_client
            .get(&ha_prefix, Some(etcd_client::GetOptions::new().with_prefix()))
            .await
        {
            // Try to find primary state
            let mut found_primary = false;
            for kv in ha_resp.kvs() {
                if let Ok(value_str) = std::str::from_utf8(kv.value()) {
                    if value_str.contains("\"state\":\"Primary\"") {
                        found_primary = true;
                        self.ha_state.set(1.0); // Primary
                        break;
                    } else if value_str.contains("\"state\":\"Standby\"") {
                        self.ha_state.set(2.0); // Standby
                        break;
                    }
                }
            }
            if !found_primary {
                // Check if we have any HA state at all
                if ha_resp.kvs().is_empty() {
                    self.ha_state.set(0.0); // Unknown
                }
            }
        }

        Ok(())
    }
}
