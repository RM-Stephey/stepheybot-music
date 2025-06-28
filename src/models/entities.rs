//! Database entities for StepheyBot Music
//!
//! This module contains all the database entity structs that map to the database tables.
//! These structs are used for database operations and API serialization/deserialization.

use super::generate_id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

/// User entity - represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub navidrome_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub listening_time_total: i64,
    pub track_count_total: i64,
}

impl User {
    /// Create a new user
    pub fn new(navidrome_id: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id: crate::models::generate_id(),
            navidrome_id,
            username,
            display_name: None,
            email: None,
            is_admin: false,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_seen_at: None,
            listening_time_total: 0,
            track_count_total: 0,
        }
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get display name or fallback to username
    pub fn display_name(&self) -> &str {
        self.display_name.as_ref().unwrap_or(&self.username)
    }

    /// Format total listening time as human-readable string
    pub fn formatted_listening_time(&self) -> String {
        crate::models::format_duration(self.listening_time_total as u32)
    }
}

/// Artist entity - represents a music artist
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub biography: Option<String>,
    pub country: Option<String>,
    pub formed_year: Option<i32>,
    pub disbanded_year: Option<i32>,
    pub artist_type: Option<String>,
    pub gender: Option<String>,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub play_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
}

impl Artist {
    /// Create a new artist
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: crate::models::generate_id(),
            name,
            sort_name: None,
            musicbrainz_id: None,
            biography: None,
            country: None,
            formed_year: None,
            disbanded_year: None,
            artist_type: None,
            gender: None,
            image_url: None,
            created_at: now,
            updated_at: now,
            play_count: 0,
            last_played_at: None,
        }
    }

    /// Get sort name or fallback to name
    pub fn sort_name(&self) -> &str {
        self.sort_name.as_ref().unwrap_or(&self.name)
    }

    /// Check if artist is still active (not disbanded)
    pub fn is_active(&self) -> bool {
        self.disbanded_year.is_none()
    }

    /// Get artist's active years as a string
    pub fn active_years(&self) -> String {
        match (self.formed_year, self.disbanded_year) {
            (Some(formed), Some(disbanded)) => format!("{}-{}", formed, disbanded),
            (Some(formed), None) => format!("{}-present", formed),
            (None, Some(disbanded)) => format!("unknown-{}", disbanded),
            (None, None) => "unknown".to_string(),
        }
    }
}

/// Album entity - represents a music album
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub sort_title: Option<String>,
    pub artist_id: String,
    pub musicbrainz_id: Option<String>,
    pub release_date: Option<String>,
    pub release_year: Option<i32>,
    pub album_type: Option<String>,
    pub track_count: i32,
    pub duration: i32,
    pub genre: Option<String>,
    pub artwork_url: Option<String>,
    pub artwork_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub play_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
}

impl Album {
    /// Create a new album
    pub fn new(title: String, artist_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: crate::models::generate_id(),
            title,
            sort_title: None,
            artist_id,
            musicbrainz_id: None,
            release_date: None,
            release_year: None,
            album_type: None,
            track_count: 0,
            duration: 0,
            genre: None,
            artwork_url: None,
            artwork_path: None,
            created_at: now,
            updated_at: now,
            play_count: 0,
            last_played_at: None,
        }
    }

    /// Get sort title or fallback to title
    pub fn sort_title(&self) -> &str {
        self.sort_title.as_ref().unwrap_or(&self.title)
    }

    /// Format album duration
    pub fn formatted_duration(&self) -> String {
        crate::models::format_duration(self.duration as u32)
    }

    /// Get album type or default
    pub fn album_type(&self) -> &str {
        self.album_type
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("album")
    }
}

/// Track entity - represents an individual song
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub sort_title: Option<String>,
    pub artist_id: String,
    pub album_id: Option<String>,
    pub musicbrainz_id: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: i32,
    pub duration: Option<i32>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub format: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub play_count: i64,
    pub last_played_at: Option<DateTime<Utc>>,
    pub love_count: i64,
}

impl Track {
    /// Create a new track
    pub fn new(title: String, artist_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: crate::models::generate_id(),
            title,
            sort_title: None,
            artist_id,
            album_id: None,
            musicbrainz_id: None,
            track_number: None,
            disc_number: 1,
            duration: None,
            file_path: None,
            file_size: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
            format: None,
            created_at: now,
            updated_at: now,
            play_count: 0,
            last_played_at: None,
            love_count: 0,
        }
    }

    /// Get sort title or fallback to title
    pub fn sort_title(&self) -> &str {
        self.sort_title.as_ref().unwrap_or(&self.title)
    }

    /// Format track duration
    pub fn formatted_duration(&self) -> Option<String> {
        self.duration
            .map(|d| crate::models::format_duration(d as u32))
    }

    /// Check if track has audio file
    pub fn has_file(&self) -> bool {
        self.file_path.is_some()
    }

    /// Get audio quality description
    pub fn quality_description(&self) -> String {
        match (self.bitrate, self.sample_rate, &self.format) {
            (Some(bitrate), Some(sample_rate), Some(format)) => {
                format!(
                    "{} kbps, {} Hz, {}",
                    bitrate,
                    sample_rate,
                    format.to_uppercase()
                )
            }
            (Some(bitrate), _, Some(format)) => {
                format!("{} kbps, {}", bitrate, format.to_uppercase())
            }
            (_, _, Some(format)) => format.to_uppercase(),
            _ => "Unknown".to_string(),
        }
    }

    /// Get file size in human-readable format
    pub fn formatted_file_size(&self) -> Option<String> {
        self.file_size.map(|size| {
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        })
    }
}

/// Genre entity - represents a music genre
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Genre {
    pub id: i64,
    pub name: String,
    pub parent_genre_id: Option<i64>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Genre {
    /// Create a new genre
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            parent_genre_id: None,
            description: None,
            created_at: Utc::now(),
        }
    }
}

/// Track genre association
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackGenre {
    pub id: i64,
    pub track_id: String,
    pub genre_id: i64,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

/// Playlist entity - represents a user playlist
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
    pub navidrome_id: Option<String>,
    pub is_public: bool,
    pub is_smart: bool,
    pub smart_criteria: Option<String>,
    pub track_count: i32,
    pub duration: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_played_at: Option<DateTime<Utc>>,
    pub play_count: i64,
}

impl Playlist {
    /// Create a new playlist
    pub fn new(name: String, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: crate::models::generate_id(),
            name,
            description: None,
            user_id,
            navidrome_id: None,
            is_public: false,
            is_smart: false,
            smart_criteria: None,
            track_count: 0,
            duration: 0,
            created_at: now,
            updated_at: now,
            last_played_at: None,
            play_count: 0,
        }
    }

    /// Create a new smart playlist
    pub fn new_smart(name: String, user_id: String, criteria: String) -> Self {
        let mut playlist = Self::new(name, user_id);
        playlist.is_smart = true;
        playlist.smart_criteria = Some(criteria);
        playlist
    }

    /// Format playlist duration
    pub fn formatted_duration(&self) -> String {
        crate::models::format_duration(self.duration as u32)
    }

    /// Get playlist type description
    pub fn playlist_type(&self) -> &str {
        if self.is_smart {
            "Smart Playlist"
        } else {
            "Playlist"
        }
    }
}

/// Playlist track association
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaylistTrack {
    pub id: i64,
    pub playlist_id: String,
    pub track_id: String,
    pub position: i32,
    pub added_at: DateTime<Utc>,
    pub added_by_user_id: Option<String>,
}

impl PlaylistTrack {
    /// Create a new playlist track entry
    pub fn new(playlist_id: String, track_id: String, position: i32) -> Self {
        Self {
            id: 0, // Will be set by database
            playlist_id,
            track_id,
            position,
            added_at: Utc::now(),
            added_by_user_id: None,
        }
    }
}

/// Listening history entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ListeningHistory {
    pub id: i64,
    pub user_id: String,
    pub track_id: String,
    pub played_at: DateTime<Utc>,
    pub play_duration: Option<i32>,
    pub completion_percentage: Option<f64>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub client_name: Option<String>,
    pub client_version: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub skip_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ListeningHistory {
    /// Create a new listening history entry
    pub fn new(user_id: String, track_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            user_id,
            track_id,
            played_at: now,
            play_duration: None,
            completion_percentage: None,
            source: None,
            source_id: None,
            client_name: None,
            client_version: None,
            ip_address: None,
            user_agent: None,
            skip_reason: None,
            created_at: now,
        }
    }

    /// Check if this was a complete play (>80% completion)
    pub fn is_complete_play(&self) -> bool {
        self.completion_percentage.map_or(false, |p| p > 0.8)
    }

    /// Check if this was a skip (<20% completion)
    pub fn is_skip(&self) -> bool {
        self.completion_percentage.map_or(false, |p| p < 0.2)
    }

    /// Format play duration
    pub fn formatted_play_duration(&self) -> Option<String> {
        self.play_duration
            .map(|d| crate::models::format_duration(d as u32))
    }
}

/// Recommendation entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Recommendation {
    pub id: String,
    pub user_id: String,
    pub track_id: String,
    pub recommendation_type: String,
    pub score: f64,
    pub reason: Option<String>,
    pub metadata: Option<String>,
    pub is_consumed: bool,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Recommendation {
    /// Create a new recommendation
    pub fn new(user_id: String, track_id: String, recommendation_type: String, score: f64) -> Self {
        Self {
            id: crate::models::generate_id(),
            user_id,
            track_id,
            recommendation_type,
            score,
            reason: None,
            metadata: None,
            is_consumed: false,
            consumed_at: None,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Check if recommendation is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() < expires_at,
            None => true,
        }
    }

    /// Get metadata as HashMap
    pub fn metadata_map(&self) -> Option<HashMap<String, serde_json::Value>> {
        self.metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok())
    }

    /// Set metadata from HashMap
    pub fn set_metadata(&mut self, metadata: HashMap<String, serde_json::Value>) {
        self.metadata = serde_json::to_string(&metadata).ok();
    }
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub id: i64,
    pub user_id: String,
    pub preference_key: String,
    pub preference_value: String,
    pub preference_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserPreference {
    /// Create a new user preference
    pub fn new(user_id: String, key: String, value: String, pref_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            user_id,
            preference_key: key,
            preference_value: value,
            preference_type: pref_type,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get preference value as specific type
    pub fn get_value<T>(&self) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.preference_value.parse().ok()
    }

    /// Check if preference is boolean and get its value
    pub fn as_bool(&self) -> Option<bool> {
        match self.preference_value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }
}

/// Artist relationships
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtistRelationship {
    pub id: i64,
    pub artist_id: String,
    pub related_artist_id: String,
    pub relationship_type: String,
    pub strength: f64,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ArtistRelationship {
    /// Create a new artist relationship
    pub fn new(
        artist_id: String,
        related_artist_id: String,
        relationship_type: String,
        strength: f64,
    ) -> Self {
        Self {
            id: 0, // Will be set by database
            artist_id,
            related_artist_id,
            relationship_type,
            strength,
            source: None,
            created_at: Utc::now(),
        }
    }
}

/// User track ratings and interactions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserTrackRating {
    pub id: i64,
    pub user_id: String,
    pub track_id: String,
    pub rating: Option<i32>,
    pub is_loved: bool,
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserTrackRating {
    /// Create a new user track rating
    pub fn new(user_id: String, track_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            user_id,
            track_id,
            rating: None,
            is_loved: false,
            is_banned: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set rating (1-5 stars)
    pub fn set_rating(&mut self, rating: i32) -> Result<(), String> {
        if rating < 1 || rating > 5 {
            return Err("Rating must be between 1 and 5".to_string());
        }
        self.rating = Some(rating);
        Ok(())
    }

    /// Get rating as stars string
    pub fn rating_stars(&self) -> String {
        match self.rating {
            Some(rating) => "★".repeat(rating as usize) + &"☆".repeat(5 - rating as usize),
            None => "☆☆☆☆☆".to_string(),
        }
    }
}

/// Download request entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DownloadRequest {
    pub id: String,
    pub user_id: String,
    pub artist_name: String,
    pub track_title: String,
    pub album_title: Option<String>,
    pub status: String,
    pub source_url: Option<String>,
    pub magnet_url: Option<String>,
    pub torrent_hash: Option<String>,
    pub download_path: Option<String>,
    pub file_size: Option<u64>,
    pub progress: Option<f64>,
    pub download_speed: Option<u64>,
    pub upload_speed: Option<u64>,
    pub seeds: Option<i32>,
    pub peers: Option<i32>,
    pub error_message: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub track_id: Option<String>,
}

impl DownloadRequest {
    /// Create a new download request
    pub fn new(user_id: String, artist_name: String, track_title: String) -> Self {
        Self {
            id: crate::models::generate_id(),
            user_id,
            artist_name,
            track_title,
            album_title: None,
            status: "pending".to_string(),
            source_url: None,
            magnet_url: None,
            torrent_hash: None,
            download_path: None,
            file_size: None,
            progress: None,
            download_speed: None,
            upload_speed: None,
            seeds: None,
            peers: None,
            error_message: None,
            requested_at: Utc::now(),
            started_at: None,
            completed_at: None,
            processed_at: None,
            track_id: None,
        }
    }

    /// Create a new download request with magnet URL
    pub fn new_with_magnet(
        user_id: String,
        artist_name: String,
        track_title: String,
        magnet_url: String,
        torrent_hash: Option<String>,
    ) -> Self {
        let mut request = Self::new(user_id, artist_name, track_title);
        request.magnet_url = Some(magnet_url);
        request.torrent_hash = torrent_hash;
        request.status = "queued".to_string();
        request
    }

    /// Check if download is completed
    pub fn is_completed(&self) -> bool {
        self.status == "completed" && self.progress.unwrap_or(0.0) >= 1.0
    }

    /// Check if download failed
    pub fn is_failed(&self) -> bool {
        self.status == "failed" || self.status == "error"
    }

    /// Check if download is in progress
    pub fn is_in_progress(&self) -> bool {
        self.status == "downloading" || self.status == "seeding"
    }

    /// Check if download is queued
    pub fn is_queued(&self) -> bool {
        self.status == "queued" || self.status == "pending"
    }

    /// Get download progress percentage
    pub fn progress_percentage(&self) -> f64 {
        (self.progress.unwrap_or(0.0) * 100.0).round()
    }

    /// Get formatted file size
    pub fn formatted_file_size(&self) -> String {
        match self.file_size {
            Some(size) => Self::format_bytes(size),
            None => "Unknown".to_string(),
        }
    }

    /// Get download speed in human readable format
    pub fn formatted_download_speed(&self) -> String {
        match self.download_speed {
            Some(speed) => format!("{}/s", Self::format_bytes(speed)),
            None => "0 B/s".to_string(),
        }
    }

    /// Get full track description
    pub fn full_description(&self) -> String {
        match &self.album_title {
            Some(album) => format!("{} - {} ({})", self.artist_name, self.track_title, album),
            None => format!("{} - {}", self.artist_name, self.track_title),
        }
    }

    /// Update progress from torrent info
    pub fn update_from_torrent(
        &mut self,
        torrent_info: &crate::clients::transmission::TorrentInfo,
    ) {
        self.progress = Some(torrent_info.progress);
        self.file_size = Some(torrent_info.size);
        self.download_speed = Some(torrent_info.download_speed);
        self.upload_speed = Some(torrent_info.upload_speed);
        self.status = torrent_info.status_string().to_string();
        // Transmission doesn't provide seeds/peers count in basic info
        // self.seeds and self.peers will remain unchanged

        if torrent_info.progress >= 1.0 && self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Helper function to format bytes
    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Active torrent tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentDownload {
    pub id: String,
    pub download_request_id: String,
    pub torrent_hash: String,
    pub magnet_url: Option<String>,
    pub name: String,
    pub save_path: String,
    pub category: Option<String>,
    pub status: String,
    pub progress: f64,
    pub size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub seeds: i32,
    pub peers: i32,
    pub ratio: f64,
    pub eta: i64,
    pub added_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl TorrentDownload {
    /// Create a new torrent download record
    pub fn new(
        download_request_id: String,
        torrent_hash: String,
        name: String,
        save_path: String,
    ) -> Self {
        Self {
            id: generate_id(),
            download_request_id,
            torrent_hash,
            magnet_url: None,
            name,
            save_path,
            category: Some("music".to_string()),
            status: "downloading".to_string(),
            progress: 0.0,
            size: 0,
            downloaded: 0,
            uploaded: 0,
            download_speed: 0,
            upload_speed: 0,
            seeds: 0,
            peers: 0,
            ratio: 0.0,
            eta: -1,
            added_at: Utc::now(),
            completed_at: None,
            last_updated: Utc::now(),
        }
    }

    /// Update from Transmission torrent info
    pub fn update_from_torrent_info(&mut self, info: &crate::clients::transmission::TorrentInfo) {
        self.name = info.name.clone();
        self.progress = info.progress;
        self.size = info.size;
        // Transmission doesn't provide downloaded/uploaded bytes in basic info
        self.download_speed = info.download_speed;
        self.upload_speed = info.upload_speed;
        // Transmission doesn't provide seeds/peers count in basic info
        self.ratio = info.ratio;
        self.eta = info.eta;
        self.status = info.status_string().to_string();
        self.last_updated = Utc::now();

        if info.progress >= 1.0 && self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Check if torrent is completed
    pub fn is_completed(&self) -> bool {
        self.progress >= 1.0
    }

    /// Check if torrent is downloading
    pub fn is_downloading(&self) -> bool {
        self.status == "downloading" || self.status == "download_pending"
    }

    /// Check if torrent is seeding
    pub fn is_seeding(&self) -> bool {
        self.status == "uploading" || self.status == "stalledUP"
    }

    /// Get formatted progress percentage
    pub fn progress_percentage(&self) -> f64 {
        (self.progress * 100.0).round()
    }

    /// Get formatted file size
    pub fn formatted_size(&self) -> String {
        DownloadRequest::format_bytes(self.size)
    }

    /// Get formatted download speed
    pub fn formatted_download_speed(&self) -> String {
        format!("{}/s", DownloadRequest::format_bytes(self.download_speed))
    }
}

/// Individual file within a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFile {
    pub id: String,
    pub torrent_download_id: String,
    pub file_index: u32,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub progress: f64,
    pub priority: i32,
    pub is_music_file: bool,
    pub artist_name: Option<String>,
    pub track_title: Option<String>,
    pub album_title: Option<String>,
    pub processed: bool,
    pub final_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl DownloadFile {
    /// Create a new download file record
    pub fn new(
        torrent_download_id: String,
        file_index: u32,
        file_name: String,
        file_path: String,
        file_size: u64,
    ) -> Self {
        let is_music_file = is_music_file_extension(&file_name);

        Self {
            id: generate_id(),
            torrent_download_id,
            file_index,
            file_name,
            file_path,
            file_size,
            progress: 0.0,
            priority: 1,
            is_music_file,
            artist_name: None,
            track_title: None,
            album_title: None,
            processed: false,
            final_path: None,
            created_at: Utc::now(),
            processed_at: None,
        }
    }

    /// Check if file is completed
    pub fn is_completed(&self) -> bool {
        self.progress >= 1.0
    }

    /// Get formatted file size
    pub fn formatted_size(&self) -> String {
        DownloadRequest::format_bytes(self.file_size)
    }

    /// Get file extension
    pub fn extension(&self) -> Option<String> {
        std::path::Path::new(&self.file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// Mark as processed
    pub fn mark_processed(&mut self, final_path: String) {
        self.processed = true;
        self.final_path = Some(final_path);
        self.processed_at = Some(Utc::now());
    }
}

/// Check if file extension indicates a music file
fn is_music_file_extension(filename: &str) -> bool {
    const MUSIC_EXTENSIONS: &[&str] = &[
        "mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "ape", "opus", "aiff", "au",
    ];

    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        MUSIC_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

// Aggregated data structures for views and statistics

/// User listening statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserListeningStats {
    pub user_id: String,
    pub username: String,
    pub total_plays: i64,
    pub total_listening_time: Option<i64>,
    pub unique_tracks_played: i64,
    pub unique_artists_played: i64,
    pub avg_completion_rate: Option<f64>,
    pub last_played_at: Option<DateTime<Utc>>,
}

/// Track popularity statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackPopularity {
    pub track_id: String,
    pub title: String,
    pub artist_name: String,
    pub album_title: Option<String>,
    pub play_count: i64,
    pub unique_listeners: i64,
    pub avg_completion_rate: Option<f64>,
    pub rating_count: i64,
    pub avg_rating: Option<f64>,
    pub love_count: i64,
}

impl TrackPopularity {
    /// Calculate popularity score (0.0-1.0)
    pub fn popularity_score(&self) -> f64 {
        let play_weight = (self.play_count as f64).ln().max(0.0) / 10.0;
        let listener_weight = (self.unique_listeners as f64).ln().max(0.0) / 5.0;
        let completion_weight = self.avg_completion_rate.unwrap_or(0.5);
        let rating_weight = self.avg_rating.unwrap_or(2.5) / 5.0;
        let love_weight = (self.love_count as f64).ln().max(0.0) / 3.0;

        ((play_weight + listener_weight + completion_weight + rating_weight + love_weight) / 5.0)
            .min(1.0)
    }
}
