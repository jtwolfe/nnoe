use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub node: NodeConfig,
    pub etcd: EtcdConfig,
    pub cache: CacheConfig,
    pub nebula: NebulaConfig,
    pub services: ServicesConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub role: NodeRole,
    #[serde(default)]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    Management,
    #[serde(rename = "db-only")]
    DbOnly,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtcdConfig {
    pub endpoints: Vec<String>,
    pub prefix: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
}

fn default_timeout() -> u64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: String,
    pub cert: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub path: String,
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_secs: u64,
    #[serde(default = "default_cache_max_size")]
    pub max_size_mb: u64,
}

fn default_cache_ttl() -> u64 {
    300
}

fn default_cache_max_size() -> u64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NebulaConfig {
    pub enabled: bool,
    pub config_path: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    #[serde(default)]
    pub lighthouse_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    #[serde(default)]
    pub dns: Option<DnsServiceConfig>,
    #[serde(default)]
    pub dhcp: Option<DhcpServiceConfig>,
    #[serde(default)]
    pub dnsdist: Option<DnsdistServiceConfig>,
    #[serde(default)]
    pub cerbos: Option<CerbosServiceConfig>,
    #[serde(default)]
    pub lynis: Option<LynisServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServiceConfig {
    pub enabled: bool,
    #[serde(default = "default_dns_engine")]
    pub engine: String,
    pub config_path: String,
    pub zone_dir: String,
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
}

fn default_dns_engine() -> String {
    "knot".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpServiceConfig {
    pub enabled: bool,
    #[serde(default = "default_dhcp_engine")]
    pub engine: String,
    pub config_path: String,
    #[serde(default)]
    pub ha_pair_id: Option<String>,
    #[serde(default = "default_interface")]
    pub interface: String,
    #[serde(default = "default_kea_control_port")]
    pub control_port: u16,
}

fn default_dhcp_engine() -> String {
    "kea".to_string()
}

fn default_listen_address() -> String {
    "0.0.0.0".to_string()
}

fn default_listen_port() -> u16 {
    53
}

fn default_interface() -> String {
    "eth0".to_string()
}

fn default_kea_control_port() -> u16 {
    8000
}

fn default_dnsdist_control_port() -> u16 {
    5199
}

fn default_prometheus_port() -> u16 {
    9090
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsdistServiceConfig {
    pub enabled: bool,
    pub config_path: String,
    pub lua_script_path: String,
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default = "default_dnsdist_control_port")]
    pub control_port: u16,
    #[serde(default)]
    pub upstream_resolvers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CerbosServiceConfig {
    pub enabled: bool,
    pub endpoint: String,
    #[serde(default = "default_cerbos_timeout")]
    pub timeout_secs: u64,
}

fn default_cerbos_timeout() -> u64 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LynisServiceConfig {
    pub enabled: bool,
    #[serde(default = "default_lynis_interval")]
    pub audit_interval_secs: u64,
    pub report_path: String,
}

fn default_lynis_interval() -> u64 {
    86400 // 24 hours
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
    #[serde(default)]
    pub file: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl AgentConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: AgentConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn default_config() -> Self {
        Self {
            node: NodeConfig {
                name: "nnoe-node-1".to_string(),
                role: NodeRole::Active,
                node_id: None,
            },
            etcd: EtcdConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                prefix: "/nnoe".to_string(),
                timeout_secs: 5,
                tls: None,
            },
            cache: CacheConfig {
                path: "/var/nnoe/cache".to_string(),
                default_ttl_secs: 300,
                max_size_mb: 100,
            },
            nebula: NebulaConfig {
                enabled: false,
                config_path: Some("/etc/nebula/config.yml".to_string()),
                cert_path: None,
                key_path: None,
                lighthouse_hosts: vec![],
            },
            services: ServicesConfig {
                dns: None,
                dhcp: None,
                dnsdist: None,
                cerbos: None,
                lynis: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                json: false,
                file: None,
            },
        }
    }
}
