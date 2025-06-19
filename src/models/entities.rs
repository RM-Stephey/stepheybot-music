//! Database entities for StepheyBot Music
//!
//! This module contains all the database entity structs that map to the database tables.
//! These structs are used for database operations and API serialization/deserialization.

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
    pub error_message: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
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
            error_message: None,
            requested_at: Utc::now(),
            started_at: None,
            completed_at: None,
            track_id: None,
        }
    }

    /// Check if download is completed
    pub fn is_completed(&self) -> bool {
        self.status == "completed" && self.track_id.is_some()
    }

    /// Check if download failed
    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }

    /// Check if download is in progress
    pub fn is_in_progress(&self) -> bool {
        self.status == "downloading"
    }

    /// Get full track description
    pub fn full_description(&self) -> String {
        match &self.album_title {
            Some(album) => format!("{} - {} ({})", self.artist_name, self.track_title, album),
            None => format!("{} - {}", self.artist_name, self.track_title),
        }
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
