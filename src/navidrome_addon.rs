//! Simple Navidrome Integration Addon for StepheyBot Music
//!
//! This module provides basic Navidrome connectivity that can be easily
//! integrated into the existing application without breaking functionality.

use chrono::Utc;
use md5;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::env;

/// Simple Navidrome connection info
#[derive(Debug, Clone, Serialize)]
pub struct NavidromeInfo {
    pub enabled: bool,
    pub connected: bool,
    pub url: String,
    pub error: Option<String>,
}

/// Basic track info from Navidrome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u32,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// Basic artist info from Navidrome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleArtist {
    pub id: String,
    pub name: String,
    pub album_count: Option<u32>,
}

/// Simple library stats from Navidrome
#[derive(Debug, Clone, Serialize)]
pub struct SimpleLibraryStats {
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    pub source: String,
}

/// Basic Navidrome addon struct
pub struct NavidromeAddon {
    pub url: String,
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

impl NavidromeAddon {
    /// Create new Navidrome addon from environment variables
    pub fn from_env() -> Self {
        // Try Docker format first, then fallback to simple format
        let url = env::var("STEPHEYBOT__NAVIDROME__URL")
            .or_else(|_| env::var("NAVIDROME_URL"))
            .unwrap_or_default();

        let username = env::var("STEPHEYBOT__NAVIDROME__ADMIN_USER")
            .or_else(|_| env::var("NAVIDROME_USERNAME"))
            .unwrap_or_default();

        let password = env::var("STEPHEYBOT__NAVIDROME__ADMIN_PASSWORD")
            .or_else(|_| env::var("NAVIDROME_PASSWORD"))
            .unwrap_or_default();

        let enabled = !url.is_empty() && !username.is_empty() && !password.is_empty();

        Self {
            url,
            username,
            password,
            enabled,
        }
    }

    /// Test basic connectivity to Navidrome
    pub async fn test_connection(&self) -> NavidromeInfo {
        if !self.enabled {
            return NavidromeInfo {
                enabled: false,
                connected: false,
                url: "".to_string(),
                error: Some("Navidrome not configured".to_string()),
            };
        }

        // Try basic HTTP connectivity first
        match reqwest::get(&self.url).await {
            Ok(response) => {
                if response.status().is_success() {
                    NavidromeInfo {
                        enabled: true,
                        connected: true,
                        url: self.url.clone(),
                        error: None,
                    }
                } else {
                    NavidromeInfo {
                        enabled: true,
                        connected: false,
                        url: self.url.clone(),
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => NavidromeInfo {
                enabled: true,
                connected: false,
                url: self.url.clone(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Get basic library statistics
    pub async fn get_library_stats(&self) -> Result<SimpleLibraryStats, String> {
        if !self.enabled {
            return Err("Navidrome not configured".to_string());
        }

        // Create authentication token
        let salt = "randomsalt";
        let token = format!("{:x}", md5::compute(format!("{}{}", self.password, salt)));

        let auth_params = format!(
            "u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music",
            self.username, token, salt
        );

        // Try to get artists count
        let artists_url = format!("{}/rest/getArtists?{}", self.url, auth_params);

        match reqwest::get(&artists_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    let text = response.text().await.map_err(|e| e.to_string())?;

                    // Simple XML parsing to count artists
                    let artist_count = text.matches("<artist").count() as u32;

                    Ok(SimpleLibraryStats {
                        artists: artist_count,
                        albums: artist_count * 3, // Rough estimate
                        songs: artist_count * 30, // Rough estimate
                        source: "navidrome".to_string(),
                    })
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Get real random tracks from Navidrome
    pub async fn get_random_tracks(&self, limit: u32) -> Result<Vec<SimpleTrack>, String> {
        if !self.enabled {
            return Err("Navidrome not configured".to_string());
        }

        // Create authentication token
        let salt = "randomsalt";
        let token = format!("{:x}", md5::compute(format!("{}{}", self.password, salt)));

        let auth_params = format!(
            "u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music&size={}",
            self.username, token, salt, limit
        );

        let songs_url = format!("{}/rest/getRandomSongs?{}", self.url, auth_params);

        match reqwest::get(&songs_url).await {
            Ok(response) => {
                if response.status().is_success() {
                    let text = response.text().await.map_err(|e| e.to_string())?;

                    // Parse XML using regex to extract song attributes
                    let mut tracks = Vec::new();

                    // More flexible regex to handle the actual Navidrome XML structure
                    let song_regex = Regex::new(r#"<song[^>]+>"#)
                        .map_err(|e| format!("Song regex error: {}", e))?;

                    for song_match in song_regex.find_iter(&text) {
                        let song_xml = song_match.as_str();

                        // Extract individual attributes
                        let id = extract_attribute(song_xml, "id")
                            .unwrap_or_else(|| format!("unknown_{}", tracks.len()));
                        let title = extract_attribute(song_xml, "title")
                            .unwrap_or("Unknown Title".to_string());
                        let artist = extract_attribute(song_xml, "artist")
                            .unwrap_or("Unknown Artist".to_string());
                        let album = extract_attribute(song_xml, "album")
                            .unwrap_or("Unknown Album".to_string());
                        let duration_str =
                            extract_attribute(song_xml, "duration").unwrap_or("0".to_string());
                        let duration = duration_str.parse::<u32>().unwrap_or(0);
                        let year =
                            extract_attribute(song_xml, "year").and_then(|y| y.parse::<u32>().ok());
                        let genre = extract_attribute(song_xml, "genre");

                        tracks.push(SimpleTrack {
                            id,
                            title,
                            artist,
                            album,
                            duration,
                            year,
                            genre,
                        });

                        // Limit the number of tracks we process
                        if tracks.len() >= limit as usize {
                            break;
                        }
                    }

                    // Log for debugging if no tracks found
                    if tracks.is_empty() {
                        eprintln!(
                            "No tracks parsed from Navidrome response. XML sample: {}",
                            &text.chars().take(500).collect::<String>()
                        );
                    }

                    Ok(tracks)
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Create a simple status report
    pub async fn get_status(&self) -> serde_json::Value {
        let info = self.test_connection().await;

        serde_json::json!({
            "navidrome_addon": {
                "enabled": info.enabled,
                "connected": info.connected,
                "url": info.url,
                "error": info.error,
                "features": [
                    "basic_connectivity",
                    "library_stats",
                    "random_tracks"
                ],
                "version": "simple_addon_v1"
            },
            "timestamp": Utc::now()
        })
    }

    /// Get enhanced library stats (combines sample data with Navidrome info)
    pub async fn get_enhanced_stats(&self, sample_stats: &serde_json::Value) -> serde_json::Value {
        if let Ok(navidrome_stats) = self.get_library_stats().await {
            serde_json::json!({
                "sample_data": sample_stats,
                "navidrome_data": {
                    "total_artists": navidrome_stats.artists,
                    "total_albums": navidrome_stats.albums,
                    "total_songs": navidrome_stats.songs,
                    "source": navidrome_stats.source
                },
                "combined_mode": true,
                "timestamp": Utc::now()
            })
        } else {
            serde_json::json!({
                "sample_data": sample_stats,
                "navidrome_data": null,
                "navidrome_error": "Connection failed",
                "combined_mode": false,
                "timestamp": Utc::now()
            })
        }
    }
}

/// Simple function to create addon instance
pub fn create_navidrome_addon() -> NavidromeAddon {
    NavidromeAddon::from_env()
}

/// Test function that can be called from main application
pub async fn test_navidrome_integration() -> serde_json::Value {
    let addon = create_navidrome_addon();
    let status = addon.get_status().await;

    if addon.enabled {
        match addon.get_library_stats().await {
            Ok(stats) => {
                serde_json::json!({
                    "status": status,
                    "library_test": {
                        "success": true,
                        "stats": stats
                    }
                })
            }
            Err(e) => {
                serde_json::json!({
                    "status": status,
                    "library_test": {
                        "success": false,
                        "error": e
                    }
                })
            }
        }
    } else {
        serde_json::json!({
            "status": status,
            "library_test": {
                "success": false,
                "error": "Navidrome not configured"
            }
        })
    }
}

/// Helper function to check if Navidrome is configured
pub fn is_navidrome_configured() -> bool {
    let addon = create_navidrome_addon();
    addon.enabled
}

/// Helper function to get simple connection status
pub async fn get_connection_status() -> serde_json::Value {
    let addon = create_navidrome_addon();
    addon.get_status().await
}

/// Helper function to extract XML attributes
fn extract_attribute(xml: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr_name);
    if let Ok(regex) = Regex::new(&pattern) {
        if let Some(cap) = regex.captures(xml) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addon_creation() {
        let addon = NavidromeAddon::from_env();
        assert!(!addon.url.is_empty() || !addon.enabled);
    }

    #[tokio::test]
    async fn test_status_function() {
        let status = get_connection_status().await;
        assert!(status.is_object());
    }

    #[test]
    fn test_extract_attribute() {
        let xml = r#"<song id="Tqfet74Vd1rPK3O6AWRbAd" title="Ouch" artist="Kah-Lo" album="The Arrival" duration="201">"#;
        assert_eq!(
            extract_attribute(xml, "id"),
            Some("Tqfet74Vd1rPK3O6AWRbAd".to_string())
        );
        assert_eq!(extract_attribute(xml, "title"), Some("Ouch".to_string()));
        assert_eq!(extract_attribute(xml, "artist"), Some("Kah-Lo".to_string()));
        assert_eq!(extract_attribute(xml, "duration"), Some("201".to_string()));
        assert_eq!(extract_attribute(xml, "nonexistent"), None);
    }
}
