// src/client/retry.rs

use crate::error::LinearMcpError;
use std::time::Duration;
use tracing::{instrument, warn};

/// Retry configuration for transient failures.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not counting the initial attempt).
    pub max_retries: u32,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Maximum delay cap so backoff doesn't grow unbounded.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(16),
        }
    }
}

/// Executes `op` with exponential backoff, retrying only on errors
/// where `LinearMcpError::is_retryable()` returns `true`.
///
/// Uses full-jitter backoff:
///   delay = random(0, min(max_delay, base_delay * 2^attempt))
#[instrument(skip(op), fields(retry.max = config.max_retries))]
pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    op_name: &str,
    mut op: F,
) -> Result<T, LinearMcpError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, LinearMcpError>>,
{
    let mut attempt = 0u32;

    loop {
        match op().await {
            Ok(val) => return Ok(val),

            Err(e) if !e.is_retryable() || attempt >= config.max_retries => {
                if attempt > 0 {
                    warn!(
                        op     = op_name,
                        attempt,
                        error  = %e,
                        "Operation failed after retries"
                    );
                }
                return Err(e);
            }

            Err(e) => {
                // Honour Retry-After if the server gave us one.
                let server_hint = if let LinearMcpError::RateLimited {
                    retry_after_secs: Some(secs),
                } = &e
                {
                    Some(Duration::from_secs(*secs))
                } else {
                    None
                };

                let backoff = server_hint.unwrap_or_else(|| {
                    let exp = config.base_delay * 2u32.saturating_pow(attempt);
                    let capped = exp.min(config.max_delay);
                    // Full-jitter: uniform sample in [0, capped]
                    let jitter_ms = rand::random::<u64>() % (capped.as_millis() as u64 + 1);
                    Duration::from_millis(jitter_ms)
                });

                warn!(
                    op      = op_name,
                    attempt,
                    delay_ms = backoff.as_millis(),
                    error    = %e,
                    "Retryable error — backing off"
                );

                tokio::time::sleep(backoff).await;
                attempt += 1;
            }
        }
    }
}
