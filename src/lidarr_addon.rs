//! Lidarr Integration Addon for StepheyBot Music
//!
//! This module provides Lidarr connectivity for automatic music discovery,
//! monitoring, and downloading integration.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;

/// Lidarr connection info
#[derive(Debug, Clone, Serialize)]
pub struct LidarrInfo {
    pub enabled: bool,
    pub connected: bool,
    pub url: String,
    pub error: Option<String>,
}

/// Basic artist info from Lidarr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrArtist {
    pub id: Option<u32>,
    pub status: Option<String>,
    pub ended: Option<bool>,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "foreignArtistId")]
    pub foreign_artist_id: Option<String>,
    #[serde(rename = "tadbId")]
    pub tadb_id: Option<u32>,
    #[serde(rename = "discogsId")]
    pub discogs_id: Option<u32>,
    pub overview: Option<String>,
    #[serde(rename = "artistType")]
    pub artist_type: Option<String>,
    pub disambiguation: Option<String>,
    pub links: Option<Vec<LidarrLink>>,
    pub images: Option<Vec<LidarrImage>>,
    pub path: Option<String>,
    #[serde(rename = "qualityProfileId")]
    pub quality_profile_id: Option<u32>,
    #[serde(rename = "metadataProfileId")]
    pub metadata_profile_id: Option<u32>,
    pub monitored: Option<bool>,
    #[serde(rename = "rootFolderPath")]
    pub root_folder_path: Option<String>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<u32>>,
    pub added: Option<String>,
    pub ratings: Option<LidarrRating>,
    pub statistics: Option<LidarrArtistStatistics>,
}

/// Lidarr artist link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrLink {
    pub url: String,
    pub name: String,
}

/// Lidarr image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrImage {
    pub url: String,
    #[serde(rename = "coverType")]
    pub cover_type: String,
}

/// Lidarr rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrRating {
    pub votes: Option<u32>,
    pub value: Option<f64>,
}

/// Lidarr artist statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrArtistStatistics {
    #[serde(rename = "albumCount")]
    pub album_count: Option<u32>,
    #[serde(rename = "trackFileCount")]
    pub track_file_count: Option<u32>,
    #[serde(rename = "trackCount")]
    pub track_count: Option<u32>,
    #[serde(rename = "totalTrackCount")]
    pub total_track_count: Option<u32>,
    #[serde(rename = "sizeOnDisk")]
    pub size_on_disk: Option<u64>,
    #[serde(rename = "percentOfTracks")]
    pub percent_of_tracks: Option<f64>,
}

/// Lidarr system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrSystemStatus {
    pub version: String,
    #[serde(rename = "buildTime")]
    pub build_time: String,
    #[serde(rename = "isDebug")]
    pub is_debug: bool,
    #[serde(rename = "isProduction")]
    pub is_production: bool,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    #[serde(rename = "isUserInteractive")]
    pub is_user_interactive: bool,
    #[serde(rename = "startupPath")]
    pub startup_path: String,
    #[serde(rename = "appData")]
    pub app_data: String,
    #[serde(rename = "osName")]
    pub os_name: String,
    #[serde(rename = "osVersion")]
    pub os_version: String,
    #[serde(rename = "isMonoRuntime")]
    pub is_mono_runtime: bool,
    #[serde(rename = "isMono")]
    pub is_mono: bool,
    #[serde(rename = "isLinux")]
    pub is_linux: bool,
    #[serde(rename = "isOsx")]
    pub is_osx: bool,
    #[serde(rename = "isWindows")]
    pub is_windows: bool,
    #[serde(rename = "isDocker")]
    pub is_docker: bool,
    pub mode: String,
    pub branch: String,
    pub authentication: String,
    #[serde(rename = "sqliteVersion")]
    pub sqlite_version: String,
    #[serde(rename = "urlBase")]
    pub url_base: Option<String>,
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: String,
    #[serde(rename = "runtimeName")]
    pub runtime_name: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
}

/// Lidarr search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrSearchResult {
    #[serde(rename = "foreignArtistId")]
    pub foreign_artist_id: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    pub overview: Option<String>,
    pub disambiguation: Option<String>,
    pub images: Option<Vec<LidarrImage>>,
    pub links: Option<Vec<LidarrLink>>,
    pub genres: Option<Vec<String>>,
    pub ratings: Option<LidarrRating>,
}

/// Basic Lidarr addon struct
pub struct LidarrAddon {
    pub url: String,
    pub api_key: String,
    pub enabled: bool,
}

impl LidarrAddon {
    /// Create new Lidarr addon from environment variables
    pub fn from_env() -> Self {
        let url = env::var("STEPHEYBOT__LIDARR__URL")
            .or_else(|_| env::var("LIDARR_URL"))
            .unwrap_or_default();

        let api_key = env::var("STEPHEYBOT__LIDARR__API_KEY")
            .or_else(|_| env::var("LIDARR_API_KEY"))
            .unwrap_or_default();

        let enabled = !url.is_empty() && !api_key.is_empty();

        Self {
            url,
            api_key,
            enabled,
        }
    }

    /// Test basic connectivity to Lidarr
    pub async fn test_connection(&self) -> LidarrInfo {
        if !self.enabled {
            return LidarrInfo {
                enabled: false,
                connected: false,
                url: "".to_string(),
                error: Some("Lidarr not configured".to_string()),
            };
        }

        // Test system status endpoint
        let status_url = format!("{}/api/v1/system/status", self.url);

        let client = reqwest::Client::new();
        match client
            .get(&status_url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    LidarrInfo {
                        enabled: true,
                        connected: true,
                        url: self.url.clone(),
                        error: None,
                    }
                } else {
                    LidarrInfo {
                        enabled: true,
                        connected: false,
                        url: self.url.clone(),
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => LidarrInfo {
                enabled: true,
                connected: false,
                url: self.url.clone(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Get system status from Lidarr
    pub async fn get_system_status(&self) -> Result<LidarrSystemStatus, String> {
        if !self.enabled {
            return Err("Lidarr not configured".to_string());
        }

        let status_url = format!("{}/api/v1/system/status", self.url);

        let client = reqwest::Client::new();
        match client
            .get(&status_url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<LidarrSystemStatus>().await {
                        Ok(status) => Ok(status),
                        Err(e) => Err(format!("Failed to parse system status: {}", e)),
                    }
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Get all artists from Lidarr
    pub async fn get_artists(&self) -> Result<Vec<LidarrArtist>, String> {
        if !self.enabled {
            return Err("Lidarr not configured".to_string());
        }

        let artists_url = format!("{}/api/v1/artist", self.url);

        let client = reqwest::Client::new();
        match client
            .get(&artists_url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<LidarrArtist>>().await {
                        Ok(artists) => Ok(artists),
                        Err(e) => Err(format!("Failed to parse artists: {}", e)),
                    }
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Search for artists by name
    pub async fn search_artist(&self, query: &str) -> Result<Vec<LidarrSearchResult>, String> {
        if !self.enabled {
            return Err("Lidarr not configured".to_string());
        }

        let search_url = format!(
            "{}/api/v1/artist/lookup?term={}",
            self.url,
            urlencoding::encode(query)
        );

        let client = reqwest::Client::new();
        match client
            .get(&search_url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<LidarrSearchResult>>().await {
                        Ok(results) => Ok(results),
                        Err(e) => Err(format!("Failed to parse search results: {}", e)),
                    }
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Add artist to monitoring
    pub async fn add_artist(
        &self,
        artist: &LidarrSearchResult,
        quality_profile_id: u32,
        metadata_profile_id: u32,
        root_folder_path: &str,
    ) -> Result<LidarrArtist, String> {
        if !self.enabled {
            return Err("Lidarr not configured".to_string());
        }

        let add_url = format!("{}/api/v1/artist", self.url);

        let payload = serde_json::json!({
            "foreignArtistId": artist.foreign_artist_id,
            "artistName": artist.artist_name,
            "qualityProfileId": quality_profile_id,
            "metadataProfileId": metadata_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": true,
            "addOptions": {
                "searchForMissingAlbums": true
            }
        });

        let client = reqwest::Client::new();
        match client
            .post(&add_url)
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<LidarrArtist>().await {
                        Ok(added_artist) => Ok(added_artist),
                        Err(e) => Err(format!("Failed to parse added artist: {}", e)),
                    }
                } else {
                    let status = response.status();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(format!("API Error {}: {}", status, error_text))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Get Lidarr statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, String> {
        if !self.enabled {
            return Err("Lidarr not configured".to_string());
        }

        let artists = self.get_artists().await?;
        let total_artists = artists.len();
        let monitored_artists = artists
            .iter()
            .filter(|a| a.monitored.unwrap_or(false))
            .count();

        let total_albums: u32 = artists
            .iter()
            .filter_map(|a| a.statistics.as_ref())
            .filter_map(|s| s.album_count)
            .sum();

        let total_tracks: u32 = artists
            .iter()
            .filter_map(|a| a.statistics.as_ref())
            .filter_map(|s| s.track_file_count)
            .sum();

        let total_size: u64 = artists
            .iter()
            .filter_map(|a| a.statistics.as_ref())
            .filter_map(|s| s.size_on_disk)
            .sum();

        Ok(serde_json::json!({
            "total_artists": total_artists,
            "monitored_artists": monitored_artists,
            "total_albums": total_albums,
            "total_tracks": total_tracks,
            "total_size_bytes": total_size,
            "total_size_gb": (total_size as f64) / (1024.0 * 1024.0 * 1024.0),
            "source": "lidarr"
        }))
    }

    /// Create a status report
    pub async fn get_status(&self) -> serde_json::Value {
        let info = self.test_connection().await;

        let system_status = if info.connected {
            match self.get_system_status().await {
                Ok(status) => Some(serde_json::json!({
                    "version": status.version,
                    "is_docker": status.is_docker,
                    "branch": status.branch,
                    "start_time": status.start_time
                })),
                Err(_) => None,
            }
        } else {
            None
        };

        serde_json::json!({
            "lidarr_addon": {
                "enabled": info.enabled,
                "connected": info.connected,
                "url": info.url,
                "error": info.error,
                "system_status": system_status,
                "features": [
                    "artist_monitoring",
                    "music_search",
                    "automatic_downloads",
                    "library_management"
                ],
                "version": "lidarr_addon_v1"
            },
            "timestamp": Utc::now()
        })
    }
}

/// Simple function to create addon instance
pub fn create_lidarr_addon() -> LidarrAddon {
    LidarrAddon::from_env()
}

/// Test function that can be called from main application
pub async fn test_lidarr_integration() -> serde_json::Value {
    let addon = create_lidarr_addon();
    let status = addon.get_status().await;

    if addon.enabled {
        match addon.get_stats().await {
            Ok(stats) => {
                serde_json::json!({
                    "status": status,
                    "stats_test": {
                        "success": true,
                        "stats": stats
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": status,
                    "stats_test": {
                        "success": false,
                        "error": e
                    }
                })
            }
        }
    } else {
        serde_json::json!({
            "status": status,
            "stats_test": {
                "success": false,
                "error": "Lidarr not configured"
            }
        })
    }
}

/// Helper function to check if Lidarr is configured
pub fn is_lidarr_configured() -> bool {
    let addon = create_lidarr_addon();
    addon.enabled
}

/// Helper function to get simple connection status
pub async fn get_lidarr_connection_status() -> serde_json::Value {
    let addon = create_lidarr_addon();
    addon.get_status().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addon_creation() {
        let addon = LidarrAddon::from_env();
        assert!(!addon.url.is_empty() || !addon.enabled);
    }

    #[tokio::test]
    async fn test_status_function() {
        let status = get_lidarr_connection_status().await;
        assert!(status.is_object());
    }
}
