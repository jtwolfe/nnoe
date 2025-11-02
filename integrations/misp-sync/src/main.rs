use anyhow::{Context, Result};
use etcd_client::{Client, PutOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

// Retry configuration
const RETRY_CONFIG: nnoe_agent::util::retry::RetryConfig = nnoe_agent::util::retry::RetryConfig {
    max_retries: 3,
    initial_delay_ms: 500,
    max_delay_ms: 5000,
    multiplier: 2.0,
};

#[derive(Debug, Clone)]
struct Config {
    misp_instances: Vec<MispInstance>,
    etcd_endpoints: Vec<String>,
    etcd_prefix: String,
    sync_interval_secs: u64,
    feed_types: Vec<String>,
    filter_tags: Vec<String>, // Optional tag filtering
    enable_deduplication: bool,
}

#[derive(Debug, Clone)]
struct MispInstance {
    url: String,
    api_key: String,
    name: String,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MispEvent {
    id: String,
    #[serde(rename = "Event")]
    event: MispEventData,
}

#[derive(Debug, Serialize, Deserialize)]
struct MispEventData {
    id: String,
    info: String,
    uuid: String,
    #[serde(rename = "Attribute")]
    attributes: Vec<MispAttribute>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MispAttribute {
    id: String,
    #[serde(rename = "type")]
    attr_type: String,
    value: String,
    category: String,
    #[serde(default)]
    to_ids: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreatData {
    domain: String,
    source: String,
    severity: String,
    timestamp: String,
    misp_event_id: Option<String>,
    category: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("misp_sync=info")
        .init();

    info!("Starting MISP sync service");

    // Load configuration from environment or defaults
    let config = load_config()?;
    info!("MISP instances: {} configured", config.misp_instances.len());
    for instance in &config.misp_instances {
        info!("  - {}: {}", instance.name, instance.url);
    }
    info!("etcd endpoints: {:?}", config.etcd_endpoints);
    info!("Sync interval: {} seconds", config.sync_interval_secs);
    if !config.filter_tags.is_empty() {
        info!("Tag filter: {:?}", config.filter_tags);
    }
    info!("Deduplication: {}", config.enable_deduplication);

    // Connect to etcd
    let mut etcd_client = Client::connect(&config.etcd_endpoints, None)
        .await
        .context("Failed to connect to etcd")?;

    info!("Connected to etcd");

    // Start periodic sync
    let mut sync_interval = interval(Duration::from_secs(config.sync_interval_secs));

    loop {
        sync_interval.tick().await;

        match sync_misp_to_etcd(&config, &mut etcd_client).await {
            Ok(count) => {
                info!("Synced {} threat domains from MISP", count);
            }
            Err(e) => {
                error!("MISP sync failed after retries: {}", e);
                // Continue running - will retry on next interval
            }
        }
    }
}

fn load_config() -> Result<Config> {
    // Support multiple MISP instances
    let mut misp_instances = Vec::new();
    
    // Primary instance (backward compatibility)
    let misp_url = std::env::var("MISP_URL").unwrap_or_else(|_| "http://localhost".to_string());
    let misp_api_key = std::env::var("MISP_API_KEY").unwrap_or_else(|_| {
        warn!("MISP_API_KEY not set, using empty key");
        String::new()
    });
    
    if !misp_url.is_empty() && !misp_api_key.is_empty() {
        misp_instances.push(MispInstance {
            url: misp_url,
            api_key: misp_api_key,
            name: "primary".to_string(),
            enabled: true,
        });
    }
    
    // Additional instances (MISP_URL_2, MISP_URL_3, etc.)
    let mut instance_num = 2;
    loop {
        let url_var = format!("MISP_URL_{}", instance_num);
        let key_var = format!("MISP_API_KEY_{}", instance_num);
        let name_var = format!("MISP_NAME_{}", instance_num);
        
        let url = match std::env::var(&url_var) {
            Ok(v) if !v.is_empty() => v,
            _ => break, // No more instances
        };
        
        let api_key = std::env::var(&key_var).unwrap_or_else(|_| String::new());
        let name = std::env::var(&name_var).unwrap_or_else(|_| format!("instance-{}", instance_num));
        
        if !api_key.is_empty() {
            misp_instances.push(MispInstance {
                url,
                api_key,
                name,
                enabled: true,
            });
        }
        
        instance_num += 1;
    }
    
    if misp_instances.is_empty() {
        return Err(anyhow::anyhow!("No MISP instances configured"));
    }
    
    let etcd_endpoints_str =
        std::env::var("ETCD_ENDPOINTS").unwrap_or_else(|_| "http://127.0.0.1:2379".to_string());
    let etcd_endpoints: Vec<String> = etcd_endpoints_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let etcd_prefix = std::env::var("ETCD_PREFIX").unwrap_or_else(|_| "/nnoe".to_string());
    let sync_interval_secs: u64 = std::env::var("SYNC_INTERVAL_SECS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);
    
    // Tag filtering
    let filter_tags_str = std::env::var("MISP_FILTER_TAGS").unwrap_or_else(|_| String::new());
    let filter_tags: Vec<String> = if !filter_tags_str.is_empty() {
        filter_tags_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };
    
    // Deduplication
    let enable_deduplication = std::env::var("MISP_DEDUP")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    Ok(Config {
        misp_instances,
        etcd_endpoints,
        etcd_prefix,
        sync_interval_secs,
        feed_types: vec!["domain".to_string(), "hostname".to_string()],
        filter_tags,
        enable_deduplication,
    })
}

async fn sync_misp_to_etcd(config: &Config, etcd_client: &mut Client) -> Result<usize> {
    info!("Fetching threat feeds from {} MISP instance(s)", config.misp_instances.len());

    let mut all_events = Vec::new();
    let mut seen_domains = std::collections::HashSet::new(); // For deduplication

    // Fetch events from all configured MISP instances
    for instance in &config.misp_instances {
        if !instance.enabled {
            continue;
        }
        
        info!("Fetching from MISP instance: {}", instance.name);
        match fetch_misp_events_from_instance(instance, config).await {
            Ok(mut events) => {
                info!("Fetched {} events from {}", events.len(), instance.name);
                all_events.append(&mut events);
            }
            Err(e) => {
                error!("Failed to fetch from {}: {}", instance.name, e);
                // Continue with other instances
            }
        }
    }

    info!("Total events collected: {}", all_events.len());

    let mut threat_count = 0;
    let mut kv_client = etcd_client.kv_client();

    for event in all_events {
        // Check tag filtering if configured
        if !config.filter_tags.is_empty() {
            // Check if event has any of the required tags
            let event_tags: Vec<String> = event.event
                .attributes
                .iter()
                .flat_map(|attr| attr.category.clone())
                .collect();
            
            let has_tag = config.filter_tags.iter().any(|tag| {
                event_tags.iter().any(|et| et.to_lowercase().contains(&tag.to_lowercase()))
            });
            
            if !has_tag {
                continue; // Skip events without matching tags
            }
        }
        
        for attr in &event.event.attributes {
            // Only process domain/hostname attributes that should be blocked
            if !config.feed_types.contains(&attr.attr_type) || !attr.to_ids {
                continue;
            }

            let domain = attr.value.clone();
            
            // Deduplication: skip if we've already seen this domain
            if config.enable_deduplication {
                if seen_domains.contains(&domain) {
                    continue;
                }
                seen_domains.insert(domain.clone());
            }

            let threat_data = ThreatData {
                domain: domain.clone(),
                source: format!("MISP:{}", event.event.id.clone()),
                severity: determine_severity(&attr.category),
                timestamp: chrono::Utc::now().to_rfc3339(),
                misp_event_id: Some(event.id.clone()),
                category: Some(attr.category.clone()),
            };

            let key = format!(
                "{}/threats/domains/{}",
                config.etcd_prefix, threat_data.domain
            );
            let value = serde_json::to_string(&threat_data)?;

            // Retry etcd put operations
            let result = nnoe_agent::util::retry::retry_with_backoff(
                &RETRY_CONFIG,
                || async {
                    let mut kv = etcd_client.kv_client();
                    kv.put(
                        key.clone(),
                        value.clone(),
                        Some(PutOptions::new().with_prev_kv()),
                    )
                    .await
                    .context(format!(
                        "Failed to put threat to etcd: {}",
                        threat_data.domain
                    ))
                },
                &format!("put_threat_{}", threat_data.domain),
            )
            .await?;

            // Check if result indicates success (etcd put doesn't return a value on success, just Ok(()))
            let _ = result;

            threat_count += 1;
        }
    }

    Ok(threat_count)
}

async fn fetch_misp_events_from_instance(instance: &MispInstance, config: &Config) -> Result<Vec<MispEvent>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // MISP API: search for events with domain/hostname attributes
    let url = format!("{}/events/index", instance.url);

    let mut query_params = serde_json::json!({
        "returnFormat": "json",
        "limit": 1000,
        "published": true,
        "type": config.feed_types
    });
    
    // Add tag filtering if configured
    if !config.filter_tags.is_empty() {
        query_params["tags"] = serde_json::json!(config.filter_tags);
    }

    let response = client
        .post(&url)
        .header("Authorization", &instance.api_key)
        .header("Content-Type", "application/json")
        .json(&query_params)
        .send()
        .await
        .context(format!("Failed to fetch MISP events from {}", instance.name))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "MISP API ({}) returned error: {}",
            instance.name,
            response.status()
        ));
    }

    let events: Vec<MispEvent> = response
        .json()
        .await
        .context(format!("Failed to parse MISP response from {}", instance.name))?;

    Ok(events)
}

// Legacy function for backward compatibility
#[allow(dead_code)]
async fn fetch_misp_events(config: &Config) -> Result<Vec<MispEvent>> {
    if let Some(instance) = config.misp_instances.first() {
        fetch_misp_events_from_instance(instance, config).await
    } else {
        Err(anyhow::anyhow!("No MISP instances configured"))
    }
}

fn determine_severity(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "malware" | "attack-pattern" => "high".to_string(),
        "payload-delivery" | "network-activity" => "medium".to_string(),
        _ => "low".to_string(),
    }
}
