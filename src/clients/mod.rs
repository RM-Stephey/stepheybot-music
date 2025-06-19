//! API clients for external services
//!
//! This module contains clients for interacting with various external music services
//! including Navidrome, ListenBrainz, Lidarr, and MusicBrainz.

pub mod navidrome;
pub mod listenbrainz;
pub mod lidarr;
pub mod musicbrainz;

// Re-export for convenience
pub use navidrome::NavidromeClient;
pub use listenbrainz::ListenBrainzClient;
pub use lidarr::LidarrClient;
pub use musicbrainz::MusicBrainzClient;

use anyhow::Result;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use tracing::{debug, warn};

/// Rate limiter for API clients
#[derive(Debug, Clone)]
pub struct RateLimiter {
    last_request: Option<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_second: f64) -> Self {
        let min_interval = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            last_request: None,
            min_interval,
        }
    }

    /// Wait if necessary to respect rate limits
    pub async fn wait(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                let wait_time = self.min_interval - elapsed;
                debug!("Rate limiting: waiting {:?}", wait_time);
                sleep(wait_time).await;
            }
        }
        self.last_request = Some(Instant::now());
    }
}

/// Common HTTP client configuration
pub fn create_http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("StepheyBot-Music/1.0 (https://stepheybot.dev)")
        .build()?)
}

/// Health check trait for all clients
#[async_trait::async_trait]
pub trait HealthCheck {
    async fn health_check(&self) -> Result<()>;
}

/// Common error types for API clients
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("Rate limit exceeded")]
    RateLimitError,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("API error: {code} - {message}")]
    ApiError { code: u32, message: String },
}

/// Retry configuration for API requests
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

/// Retry logic for API requests
pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    config: &RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> futures::future::BoxFuture<'_, Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    let mut delay = config.base_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempt += 1;
                if attempt > config.max_retries {
                    return Err(error);
                }

                warn!(
                    "API request failed (attempt {}/{}): {:?}. Retrying in {:?}",
                    attempt, config.max_retries, error, delay
                );

                sleep(delay).await;

                // Exponential backoff
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.backoff_factor),
                    config.max_delay,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2.0); // 2 requests per second

        let start = Instant::now();
        limiter.wait().await; // First request - no wait
        limiter.wait().await; // Second request - should wait ~500ms
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(450)); // Allow some variance
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(500));
        assert_eq!(config.backoff_factor, 2.0);
    }

    #[test]
    fn test_http_client_creation() {
        let client = create_http_client().unwrap();
        assert!(client.timeout().is_some());
    }
}
