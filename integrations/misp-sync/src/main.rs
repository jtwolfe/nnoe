use anyhow::{Context, Result};
use etcd_client::{Client, PutOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct Config {
    misp_url: String,
    misp_api_key: String,
    etcd_endpoints: Vec<String>,
    etcd_prefix: String,
    sync_interval_secs: u64,
    feed_types: Vec<String>,
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
    info!("MISP URL: {}", config.misp_url);
    info!("etcd endpoints: {:?}", config.etcd_endpoints);
    info!("Sync interval: {} seconds", config.sync_interval_secs);

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
                error!("MISP sync failed: {}", e);
            }
        }
    }
}

fn load_config() -> Result<Config> {
    let misp_url = std::env::var("MISP_URL")
        .unwrap_or_else(|_| "http://localhost".to_string());
    let misp_api_key = std::env::var("MISP_API_KEY")
        .unwrap_or_else(|_| {
            warn!("MISP_API_KEY not set, using empty key");
            String::new()
        });
    let etcd_endpoints_str = std::env::var("ETCD_ENDPOINTS")
        .unwrap_or_else(|_| "http://127.0.0.1:2379".to_string());
    let etcd_endpoints: Vec<String> = etcd_endpoints_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let etcd_prefix = std::env::var("ETCD_PREFIX")
        .unwrap_or_else(|_| "/nnoe".to_string());
    let sync_interval_secs: u64 = std::env::var("SYNC_INTERVAL_SECS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600);

    Ok(Config {
        misp_url,
        misp_api_key,
        etcd_endpoints,
        etcd_prefix,
        sync_interval_secs,
        feed_types: vec!["domain".to_string(), "hostname".to_string()],
    })
}

async fn sync_misp_to_etcd(config: &Config, etcd_client: &mut Client) -> Result<usize> {
    info!("Fetching threat feeds from MISP");

    // Fetch MISP events
    let events = fetch_misp_events(config).await?;
    info!("Fetched {} events from MISP", events.len());

    let mut threat_count = 0;
    let mut kv_client = etcd_client.kv_client();

    for event in events {
        for attr in &event.event.attributes {
            // Only process domain/hostname attributes that should be blocked
            if !config.feed_types.contains(&attr.attr_type) || !attr.to_ids {
                continue;
            }

            let threat_data = ThreatData {
                domain: attr.value.clone(),
                source: "MISP".to_string(),
                severity: determine_severity(&attr.category),
                timestamp: chrono::Utc::now().to_rfc3339(),
                misp_event_id: Some(event.id.clone()),
                category: Some(attr.category.clone()),
            };

            let key = format!("{}/threats/domains/{}", config.etcd_prefix, threat_data.domain);
            let value = serde_json::to_string(&threat_data)?;

            kv_client
                .put(
                    key,
                    value,
                    Some(PutOptions::new().with_prev_kv()),
                )
                .await
                .context(format!("Failed to put threat to etcd: {}", threat_data.domain))?;

            threat_count += 1;
        }
    }

    Ok(threat_count)
}

async fn fetch_misp_events(config: &Config) -> Result<Vec<MispEvent>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // MISP API: search for events with domain/hostname attributes
    let url = format!("{}/events/index", config.misp_url);
    
    let response = client
        .post(&url)
        .header("Authorization", &config.misp_api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "returnFormat": "json",
            "limit": 1000,
            "published": true,
            "type": config.feed_types
        }))
        .send()
        .await
        .context("Failed to fetch MISP events")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "MISP API returned error: {}",
            response.status()
        ));
    }

    let events: Vec<MispEvent> = response
        .json()
        .await
        .context("Failed to parse MISP response")?;

    Ok(events)
}

fn determine_severity(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "malware" | "attack-pattern" => "high".to_string(),
        "payload-delivery" | "network-activity" => "medium".to_string(),
        _ => "low".to_string(),
    }
}

