use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn new(max_retries: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            initial_delay_ms,
            max_delay_ms,
            multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = Duration::from_millis(config.initial_delay_ms);
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation {} succeeded after {} retries", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);
                
                if attempt < config.max_retries {
                    warn!(
                        "Operation {} failed (attempt {}/{}), retrying in {:?}: {}",
                        operation_name,
                        attempt + 1,
                        config.max_retries + 1,
                        delay,
                        last_error.as_ref().unwrap()
                    );
                    
                    sleep(delay).await;
                    
                    // Exponential backoff
                    let new_delay_ms = (delay.as_millis() as f64 * config.multiplier) as u64;
                    delay = Duration::from_millis(new_delay_ms.min(config.max_delay_ms));
                } else {
                    warn!(
                        "Operation {} failed after {} attempts",
                        operation_name,
                        config.max_retries + 1
                    );
                }
            }
        }
    }

    Err(last_error
        .unwrap()
        .context(format!("Operation {} failed after {} retries", operation_name, config.max_retries + 1)))
}

