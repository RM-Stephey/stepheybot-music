//! ListenBrainz API client for StepheyBot Music
//!
//! This module provides integration with ListenBrainz for music recommendations,
//! listening statistics, and social music features.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// ListenBrainz API client
#[derive(Clone)]
pub struct ListenBrainzClient {
    client: Client,
    base_url: String,
    user_token: Option<String>,
}

/// Listen data structure for submitting to ListenBrainz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listen {
    pub track_metadata: TrackMetadata,
    pub listened_at: Option<i64>,
    pub recording_msid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub artist_name: String,
    pub track_name: String,
    pub release_name: Option<String>,
    pub additional_info: Option<AdditionalInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalInfo {
    pub duration_ms: Option<u64>,
    pub recording_mbid: Option<String>,
    pub artist_mbid: Option<String>,
    pub release_mbid: Option<String>,
}

/// Recommendation from ListenBrainz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recording_mbid: String,
    pub score: f64,
    pub track_metadata: TrackMetadata,
}

/// User listening statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListeningStats {
    pub total_listen_count: u64,
    pub artists: Vec<ArtistStats>,
    pub releases: Vec<ReleaseStats>,
    pub recordings: Vec<RecordingStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStats {
    pub artist_name: String,
    pub artist_mbid: Option<String>,
    pub listen_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseStats {
    pub release_name: String,
    pub release_mbid: Option<String>,
    pub artist_name: String,
    pub listen_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStats {
    pub track_name: String,
    pub recording_mbid: Option<String>,
    pub artist_name: String,
    pub listen_count: u64,
}

impl ListenBrainzClient {
    /// Create a new ListenBrainz client
    pub fn new(base_url: &str, user_token: Option<&str>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("StepheyBot-Music/1.0")
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = if base_url.ends_with('/') {
            base_url.trim_end_matches('/').to_string()
        } else {
            base_url.to_string()
        };

        Ok(Self {
            client,
            base_url,
            user_token: user_token.map(|s| s.to_string()),
        })
    }

    /// Perform health check on ListenBrainz service
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/1/status/get-dump-info", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to ListenBrainz")?;

        if response.status().is_success() {
            debug!("ListenBrainz health check passed");
            Ok(())
        } else {
            anyhow::bail!("ListenBrainz health check failed: {}", response.status());
        }
    }

    /// Submit listening data to ListenBrainz
    pub async fn submit_listens(&self, user_name: &str, listens: Vec<Listen>) -> Result<()> {
        if listens.is_empty() {
            return Ok(());
        }

        let token = self
            .user_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("User token required for submitting listens"))?;

        let url = format!("{}/1/submit-listens", self.base_url);

        let payload = serde_json::json!({
            "listen_type": "import",
            "payload": listens
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to submit listens")?;

        if response.status().is_success() {
            info!(
                "Successfully submitted {} listens for user {}",
                listens.len(),
                user_name
            );
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Failed to submit listens: {} - {}",
                response.status(),
                error_text
            );
        }
    }

    /// Get user's listening statistics
    pub async fn get_user_stats(
        &self,
        user_name: &str,
        range: Option<&str>,
    ) -> Result<ListeningStats> {
        let mut url = format!("{}/1/stats/user/{}/artists", self.base_url, user_name);

        if let Some(range) = range {
            url = format!("{}?range={}", url, range);
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get user stats")?;

        if response.status().is_success() {
            // For now, return empty stats - would need actual API parsing
            Ok(ListeningStats {
                total_listen_count: 0,
                artists: Vec::new(),
                releases: Vec::new(),
                recordings: Vec::new(),
            })
        } else {
            anyhow::bail!("Failed to get user stats: {}", response.status());
        }
    }

    /// Get recommendations for a user
    pub async fn get_recommendations(&self, user_name: &str) -> Result<Vec<Recommendation>> {
        let url = format!(
            "{}/1/cf/recommendation/user/{}/recording",
            self.base_url, user_name
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get recommendations")?;

        if response.status().is_success() {
            // For now, return empty recommendations - would need actual API parsing
            Ok(Vec::new())
        } else {
            warn!(
                "Failed to get recommendations for user {}: {}",
                user_name,
                response.status()
            );
            Ok(Vec::new()) // Return empty instead of error for graceful degradation
        }
    }

    /// Get similar users based on listening history
    pub async fn get_similar_users(&self, user_name: &str) -> Result<Vec<String>> {
        let url = format!("{}/1/user/{}/similar-users", self.base_url, user_name);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get similar users")?;

        if response.status().is_success() {
            // For now, return empty list - would need actual API parsing
            Ok(Vec::new())
        } else {
            warn!(
                "Failed to get similar users for {}: {}",
                user_name,
                response.status()
            );
            Ok(Vec::new())
        }
    }

    /// Submit a "now playing" notification
    pub async fn submit_now_playing(&self, listen: &Listen) -> Result<()> {
        let token = self
            .user_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("User token required for now playing"))?;

        let url = format!("{}/1/submit-listens", self.base_url);

        let payload = serde_json::json!({
            "listen_type": "playing_now",
            "payload": [listen]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to submit now playing")?;

        if response.status().is_success() {
            debug!("Successfully submitted now playing");
            Ok(())
        } else {
            warn!("Failed to submit now playing: {}", response.status());
            Ok(()) // Don't fail hard on now playing errors
        }
    }

    /// Get user's recent listens
    pub async fn get_recent_listens(
        &self,
        user_name: &str,
        count: Option<u32>,
    ) -> Result<Vec<Listen>> {
        let mut url = format!("{}/1/user/{}/listens", self.base_url, user_name);

        if let Some(count) = count {
            url = format!("{}?count={}", url, count);
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get recent listens")?;

        if response.status().is_success() {
            // For now, return empty list - would need actual API parsing
            Ok(Vec::new())
        } else {
            warn!(
                "Failed to get recent listens for {}: {}",
                user_name,
                response.status()
            );
            Ok(Vec::new())
        }
    }

    /// Convert Navidrome play data to ListenBrainz format
    pub fn create_listen_from_play(
        &self,
        artist: &str,
        track: &str,
        album: Option<&str>,
        duration_ms: Option<u64>,
        played_at: DateTime<Utc>,
    ) -> Listen {
        Listen {
            track_metadata: TrackMetadata {
                artist_name: artist.to_string(),
                track_name: track.to_string(),
                release_name: album.map(|s| s.to_string()),
                additional_info: Some(AdditionalInfo {
                    duration_ms,
                    recording_mbid: None,
                    artist_mbid: None,
                    release_mbid: None,
                }),
            },
            listened_at: Some(played_at.timestamp()),
            recording_msid: None,
        }
    }

    /// Batch submit multiple listens efficiently
    pub async fn batch_submit_listens(
        &self,
        user_name: &str,
        listens: Vec<Listen>,
        batch_size: usize,
    ) -> Result<()> {
        if listens.is_empty() {
            return Ok(());
        }

        info!(
            "Batch submitting {} listens for user {} in batches of {}",
            listens.len(),
            user_name,
            batch_size
        );

        for chunk in listens.chunks(batch_size) {
            if let Err(e) = self.submit_listens(user_name, chunk.to_vec()).await {
                warn!("Failed to submit batch of {} listens: {}", chunk.len(), e);
                // Continue with next batch instead of failing completely
            }

            // Small delay between batches to be respectful to the API
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client =
            ListenBrainzClient::new("https://api.listenbrainz.org", Some("test_token")).unwrap();
        assert_eq!(client.base_url, "https://api.listenbrainz.org");
        assert!(client.user_token.is_some());
    }

    #[test]
    fn test_create_listen_from_play() {
        let client = ListenBrainzClient::new("https://api.listenbrainz.org", None).unwrap();
        let now = Utc::now();

        let listen = client.create_listen_from_play(
            "Test Artist",
            "Test Track",
            Some("Test Album"),
            Some(180000),
            now,
        );

        assert_eq!(listen.track_metadata.artist_name, "Test Artist");
        assert_eq!(listen.track_metadata.track_name, "Test Track");
        assert_eq!(
            listen.track_metadata.release_name,
            Some("Test Album".to_string())
        );
        assert_eq!(listen.listened_at, Some(now.timestamp()));
    }
}
