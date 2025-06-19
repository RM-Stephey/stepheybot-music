//! Lidarr API client for StepheyBot Music
//!
//! This module provides integration with Lidarr for automated music acquisition,
//! artist monitoring, and download management.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Lidarr API client
#[derive(Clone)]
pub struct LidarrClient {
    client: Client,
    base_url: String,
    api_key: String,
}

/// Artist information from Lidarr
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: Option<u32>,
    pub artist_name: String,
    pub foreign_artist_id: Option<String>, // MusicBrainz ID
    pub overview: Option<String>,
    pub disambiguation: Option<String>,
    pub artist_type: Option<String>,
    pub status: Option<String>,
    pub ended: Option<bool>,
    pub genres: Option<Vec<String>>,
    pub images: Option<Vec<Image>>,
    pub links: Option<Vec<Link>>,
    pub statistics: Option<ArtistStatistics>,
    pub quality_profile_id: Option<u32>,
    pub metadata_profile_id: Option<u32>,
    pub monitored: Option<bool>,
    pub monitor_new_items: Option<String>,
    pub root_folder_path: Option<String>,
    pub folder_name: Option<String>,
    pub path: Option<String>,
    pub added: Option<DateTime<Utc>>,
}

/// Album information from Lidarr
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: Option<u32>,
    pub title: String,
    pub disambiguation: Option<String>,
    pub overview: Option<String>,
    pub artist_id: Option<u32>,
    pub foreign_album_id: Option<String>, // MusicBrainz ID
    pub monitored: Option<bool>,
    pub any_release_ok: Option<bool>,
    pub profile_id: Option<u32>,
    pub duration: Option<u32>,
    pub album_type: Option<String>,
    pub secondary_types: Option<Vec<String>>,
    pub medium_count: Option<u32>,
    pub ratings: Option<Ratings>,
    pub release_date: Option<String>,
    pub releases: Option<Vec<Release>>,
    pub genres: Option<Vec<String>>,
    pub media: Option<Vec<Medium>>,
    pub artist: Option<Artist>,
    pub images: Option<Vec<Image>>,
    pub links: Option<Vec<Link>>,
    pub statistics: Option<AlbumStatistics>,
    pub grabbed: Option<bool>,
}

/// Release information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: Option<u32>,
    pub album_id: Option<u32>,
    pub foreign_release_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub packaging: Option<String>,
    pub text_representation: Option<TextRepresentation>,
    pub country: Option<Vec<String>>,
    pub release_date: Option<String>,
    pub media: Option<Vec<Medium>>,
    pub track_count: Option<u32>,
    pub monitored: Option<bool>,
}

/// Medium information (CD, Vinyl, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    pub medium_number: Option<u32>,
    pub medium_name: Option<String>,
    pub medium_format: Option<String>,
}

/// Text representation for releases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRepresentation {
    pub language: Option<String>,
    pub script: Option<String>,
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub cover_type: String,
    pub url: String,
    pub remote_url: Option<String>,
}

/// Link information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub url: String,
    pub name: String,
}

/// Ratings information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratings {
    pub votes: Option<u32>,
    pub value: Option<f64>,
}

/// Artist statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistStatistics {
    pub album_count: Option<u32>,
    pub track_file_count: Option<u32>,
    pub track_count: Option<u32>,
    pub total_track_count: Option<u32>,
    pub size_on_disk: Option<u64>,
    pub percent_of_tracks: Option<f64>,
}

/// Album statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumStatistics {
    pub track_file_count: Option<u32>,
    pub track_count: Option<u32>,
    pub total_track_count: Option<u32>,
    pub size_on_disk: Option<u64>,
    pub percent_of_tracks: Option<f64>,
}

/// Quality profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityProfile {
    pub id: u32,
    pub name: String,
    pub upgrade_allowed: Option<bool>,
    pub cutoff: Option<u32>,
    pub items: Option<Vec<QualityItem>>,
}

/// Quality item in a profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityItem {
    pub quality: Option<Quality>,
    pub allowed: Option<bool>,
}

/// Quality information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quality {
    pub id: u32,
    pub name: String,
}

/// System status from Lidarr
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub version: String,
    pub build_time: Option<DateTime<Utc>>,
    pub is_debug: Option<bool>,
    pub is_production: Option<bool>,
    pub is_admin: Option<bool>,
    pub is_user_interactive: Option<bool>,
    pub startup_path: Option<String>,
    pub app_data: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub is_mono_runtime: Option<bool>,
    pub is_mono: Option<bool>,
    pub is_linux: Option<bool>,
    pub is_osx: Option<bool>,
    pub is_windows: Option<bool>,
    pub branch: Option<String>,
    pub authentication: Option<String>,
    pub sqlite_version: Option<String>,
    pub migration_version: Option<u32>,
    pub url_base: Option<String>,
    pub runtime_version: Option<String>,
}

/// Search result from Lidarr
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub artist: Option<Artist>,
    pub album: Option<Album>,
}

impl LidarrClient {
    /// Create a new Lidarr client
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
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
            api_key: api_key.to_string(),
        })
    }

    /// Perform health check on Lidarr
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/v1/system/status", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to connect to Lidarr")?;

        if response.status().is_success() {
            debug!("Lidarr health check passed");
            Ok(())
        } else {
            anyhow::bail!("Lidarr health check failed: {}", response.status());
        }
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let url = format!("{}/api/v1/system/status", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to get system status")?;

        if response.status().is_success() {
            let status: SystemStatus = response
                .json()
                .await
                .context("Failed to parse system status")?;
            Ok(status)
        } else {
            anyhow::bail!("Failed to get system status: {}", response.status());
        }
    }

    /// Search for artists
    pub async fn search_artist(&self, query: &str) -> Result<Vec<Artist>> {
        let url = format!("{}/api/v1/search", self.base_url);

        let mut params = HashMap::new();
        params.insert("term", query);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .query(&params)
            .send()
            .await
            .context("Failed to search for artist")?;

        if response.status().is_success() {
            let results: Vec<SearchResult> = response
                .json()
                .await
                .context("Failed to parse search results")?;

            let artists: Vec<Artist> = results
                .into_iter()
                .filter_map(|result| result.artist)
                .collect();

            Ok(artists)
        } else {
            anyhow::bail!("Failed to search for artist: {}", response.status());
        }
    }

    /// Get all monitored artists
    pub async fn get_artists(&self) -> Result<Vec<Artist>> {
        let url = format!("{}/api/v1/artist", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to get artists")?;

        if response.status().is_success() {
            let artists: Vec<Artist> = response.json().await.context("Failed to parse artists")?;
            Ok(artists)
        } else {
            anyhow::bail!("Failed to get artists: {}", response.status());
        }
    }

    /// Add an artist to monitoring
    pub async fn add_artist(&self, artist: &Artist) -> Result<Artist> {
        let url = format!("{}/api/v1/artist", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(artist)
            .send()
            .await
            .context("Failed to add artist")?;

        if response.status().is_success() {
            let added_artist: Artist = response
                .json()
                .await
                .context("Failed to parse added artist")?;

            info!("Successfully added artist: {}", artist.artist_name);
            Ok(added_artist)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Failed to add artist {}: {} - {}",
                artist.artist_name,
                response.status(),
                error_text
            );
        }
    }

    /// Get artist by ID
    pub async fn get_artist(&self, artist_id: u32) -> Result<Option<Artist>> {
        let url = format!("{}/api/v1/artist/{}", self.base_url, artist_id);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to get artist")?;

        match response.status() {
            status if status.is_success() => {
                let artist: Artist = response.json().await.context("Failed to parse artist")?;
                Ok(Some(artist))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => anyhow::bail!("Failed to get artist: {}", response.status()),
        }
    }

    /// Get albums for an artist
    pub async fn get_artist_albums(&self, artist_id: u32) -> Result<Vec<Album>> {
        let url = format!("{}/api/v1/album", self.base_url);

        let mut params = HashMap::new();
        params.insert("artistId", artist_id.to_string());

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .query(&params)
            .send()
            .await
            .context("Failed to get artist albums")?;

        if response.status().is_success() {
            let albums: Vec<Album> = response.json().await.context("Failed to parse albums")?;
            Ok(albums)
        } else {
            anyhow::bail!("Failed to get artist albums: {}", response.status());
        }
    }

    /// Get quality profiles
    pub async fn get_quality_profiles(&self) -> Result<Vec<QualityProfile>> {
        let url = format!("{}/api/v1/qualityprofile", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to get quality profiles")?;

        if response.status().is_success() {
            let profiles: Vec<QualityProfile> = response
                .json()
                .await
                .context("Failed to parse quality profiles")?;
            Ok(profiles)
        } else {
            anyhow::bail!("Failed to get quality profiles: {}", response.status());
        }
    }

    /// Trigger artist search
    pub async fn search_artist_releases(&self, artist_id: u32) -> Result<()> {
        let url = format!("{}/api/v1/command", self.base_url);

        let command = serde_json::json!({
            "name": "ArtistSearch",
            "artistId": artist_id
        });

        let response = self
            .client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&command)
            .send()
            .await
            .context("Failed to trigger artist search")?;

        if response.status().is_success() {
            info!("Successfully triggered search for artist ID: {}", artist_id);
            Ok(())
        } else {
            anyhow::bail!("Failed to trigger artist search: {}", response.status());
        }
    }

    /// Trigger album search
    pub async fn search_album(&self, album_id: u32) -> Result<()> {
        let url = format!("{}/api/v1/command", self.base_url);

        let command = serde_json::json!({
            "name": "AlbumSearch",
            "albumIds": [album_id]
        });

        let response = self
            .client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&command)
            .send()
            .await
            .context("Failed to trigger album search")?;

        if response.status().is_success() {
            info!("Successfully triggered search for album ID: {}", album_id);
            Ok(())
        } else {
            anyhow::bail!("Failed to trigger album search: {}", response.status());
        }
    }

    /// Trigger automatic search for missing albums
    pub async fn search_missing(&self) -> Result<()> {
        let url = format!("{}/api/v1/command", self.base_url);

        let command = serde_json::json!({
            "name": "MissingAlbumSearch"
        });

        let response = self
            .client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&command)
            .send()
            .await
            .context("Failed to trigger missing album search")?;

        if response.status().is_success() {
            info!("Successfully triggered missing album search");
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to trigger missing album search: {}",
                response.status()
            );
        }
    }

    /// Get artist by MusicBrainz ID
    pub async fn find_artist_by_mbid(&self, mbid: &str) -> Result<Option<Artist>> {
        let artists = self.get_artists().await?;

        for artist in artists {
            if let Some(foreign_id) = &artist.foreign_artist_id {
                if foreign_id == mbid {
                    return Ok(Some(artist));
                }
            }
        }

        Ok(None)
    }

    /// Helper function to create an artist for adding to Lidarr
    pub fn create_artist_for_monitoring(
        &self,
        name: &str,
        musicbrainz_id: Option<&str>,
        quality_profile_id: u32,
        root_folder: &str,
        monitor_new_items: bool,
    ) -> Artist {
        Artist {
            id: None,
            artist_name: name.to_string(),
            foreign_artist_id: musicbrainz_id.map(|s| s.to_string()),
            overview: None,
            disambiguation: None,
            artist_type: None,
            status: None,
            ended: None,
            genres: None,
            images: None,
            links: None,
            statistics: None,
            quality_profile_id: Some(quality_profile_id),
            metadata_profile_id: Some(1), // Default metadata profile
            monitored: Some(true),
            monitor_new_items: Some(if monitor_new_items {
                "all".to_string()
            } else {
                "none".to_string()
            }),
            root_folder_path: Some(root_folder.to_string()),
            folder_name: None,
            path: None,
            added: None,
        }
    }

    /// Queue download for a specific track/album
    pub async fn queue_download(&self, artist_name: &str, album_title: &str) -> Result<()> {
        // First, search for the artist
        let artists = self.search_artist(artist_name).await?;

        if let Some(artist) = artists.first() {
            // Check if artist is already monitored
            let existing_artist = if let Some(mbid) = &artist.foreign_artist_id {
                self.find_artist_by_mbid(mbid).await?
            } else {
                None
            };

            let artist_id = if let Some(existing) = existing_artist {
                existing.id.unwrap_or(0)
            } else {
                // Add artist to monitoring with default settings
                let new_artist = self.create_artist_for_monitoring(
                    &artist.artist_name,
                    artist.foreign_artist_id.as_deref(),
                    1,        // Default quality profile
                    "/music", // Default root folder
                    true,     // Monitor new items
                );

                let added = self.add_artist(&new_artist).await?;
                added.id.unwrap_or(0)
            };

            // Trigger search for the artist to find the specific album
            self.search_artist_releases(artist_id).await?;

            info!(
                "Queued download for album '{}' by artist '{}'",
                album_title, artist_name
            );
        } else {
            warn!(
                "Could not find artist '{}' to queue download for album '{}'",
                artist_name, album_title
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LidarrClient::new("http://localhost:8686", "test_api_key").unwrap();
        assert_eq!(client.base_url, "http://localhost:8686");
        assert_eq!(client.api_key, "test_api_key");
    }

    #[test]
    fn test_create_artist_for_monitoring() {
        let client = LidarrClient::new("http://localhost:8686", "test_key").unwrap();

        let artist = client.create_artist_for_monitoring(
            "Test Artist",
            Some("test-mbid"),
            1,
            "/music",
            true,
        );

        assert_eq!(artist.artist_name, "Test Artist");
        assert_eq!(artist.foreign_artist_id, Some("test-mbid".to_string()));
        assert_eq!(artist.quality_profile_id, Some(1));
        assert_eq!(artist.monitored, Some(true));
    }
}
