//! User models and entities for StepheyBot Music Profile System
//!
//! This module contains all user-related data structures, including user accounts,
//! profiles, integrations, and authentication data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::collections::HashMap;

/// Core user account linked to Keycloak SSO
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub keycloak_id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: i64,
    pub last_active: i64,
    pub is_active: bool,
    pub preferences_json: String,
}

impl User {
    /// Parse user preferences from JSON
    pub fn preferences(&self) -> Result<UserPreferences, serde_json::Error> {
        serde_json::from_str(&self.preferences_json)
    }

    /// Update user preferences
    pub fn set_preferences(
        &mut self,
        preferences: &UserPreferences,
    ) -> Result<(), serde_json::Error> {
        self.preferences_json = serde_json::to_string(preferences)?;
        Ok(())
    }

    /// Get display name or fallback to username
    pub fn display_name_or_username(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }

    /// Check if user was active within the last N days
    pub fn is_recently_active(&self, days: i64) -> bool {
        let threshold = chrono::Utc::now().timestamp() - (days * 24 * 60 * 60);
        self.last_active > threshold
    }
}

/// User profile information and privacy settings
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub user_id: i64,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub privacy_level: i32,
    pub share_listening_history: bool,
    pub share_playlists: bool,
    pub updated_at: i64,
}

impl UserProfile {
    /// Get privacy level as enum
    pub fn privacy_level(&self) -> PrivacyLevel {
        match self.privacy_level {
            0 => PrivacyLevel::Private,
            1 => PrivacyLevel::Friends,
            2 => PrivacyLevel::Public,
            _ => PrivacyLevel::Private,
        }
    }

    /// Set privacy level from enum
    pub fn set_privacy_level(&mut self, level: PrivacyLevel) {
        self.privacy_level = level as i32;
    }
}

/// Privacy levels for user profiles
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[repr(i32)]
pub enum PrivacyLevel {
    Private = 0,
    Friends = 1,
    Public = 2,
}

/// User preferences stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub email_notifications: bool,
    pub scrobble_enabled: bool,
    pub auto_recommendations: bool,
    pub discovery_mode: DiscoveryMode,
    pub volume_normalization: bool,
    pub crossfade_duration: Option<f64>,
    pub repeat_mode: RepeatMode,
    pub shuffle_enabled: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: Some("neon".to_string()),
            language: Some("en".to_string()),
            timezone: None,
            email_notifications: true,
            scrobble_enabled: true,
            auto_recommendations: true,
            discovery_mode: DiscoveryMode::Balanced,
            volume_normalization: true,
            crossfade_duration: Some(2.0),
            repeat_mode: RepeatMode::Off,
            shuffle_enabled: false,
        }
    }
}

/// Discovery mode for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMode {
    Conservative, // Stick to similar music
    Balanced,     // Mix of similar and new
    Adventurous,  // Explore new genres
}

/// Music repeat modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    Track,
    Playlist,
}

/// External service integration (ListenBrainz, Spotify, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserIntegration {
    pub id: i64,
    pub user_id: i64,
    pub service_name: String,
    pub service_user_id: Option<String>,
    pub api_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub enabled: bool,
    pub last_sync: Option<i64>,
    pub sync_settings_json: String,
    pub created_at: i64,
}

impl UserIntegration {
    /// Parse sync settings from JSON
    pub fn sync_settings(&self) -> Result<SyncSettings, serde_json::Error> {
        serde_json::from_str(&self.sync_settings_json)
    }

    /// Update sync settings
    pub fn set_sync_settings(&mut self, settings: &SyncSettings) -> Result<(), serde_json::Error> {
        self.sync_settings_json = serde_json::to_string(settings)?;
        Ok(())
    }

    /// Check if integration is overdue for sync
    pub fn is_sync_overdue(&self, max_hours: i64) -> bool {
        match self.last_sync {
            Some(last_sync) => {
                let threshold = chrono::Utc::now().timestamp() - (max_hours * 60 * 60);
                last_sync < threshold
            }
            None => true, // Never synced
        }
    }
}

/// Sync settings for external integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub auto_sync: bool,
    pub sync_interval_hours: i32,
    pub sync_playlists: bool,
    pub sync_favorites: bool,
    pub sync_listening_history: bool,
    pub max_tracks_per_sync: Option<i32>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_interval_hours: 24,
            sync_playlists: true,
            sync_favorites: true,
            sync_listening_history: true,
            max_tracks_per_sync: Some(1000),
        }
    }
}

/// Complete user profile with statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfileWithStats {
    pub id: i64,
    pub keycloak_id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: i64,
    pub last_active: i64,
    pub is_active: bool,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub privacy_level: i32,
    pub share_listening_history: bool,
    pub share_playlists: bool,
    pub playlist_count: i64,
    pub favorite_count: i64,
    pub total_listens: i64,
    pub following_count: i64,
    pub follower_count: i64,
}

/// User creation request from Keycloak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub keycloak_id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub privacy_level: Option<PrivacyLevel>,
    pub share_listening_history: Option<bool>,
    pub share_playlists: Option<bool>,
}

/// User preferences update request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserPreferencesRequest {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub email_notifications: Option<bool>,
    pub scrobble_enabled: Option<bool>,
    pub auto_recommendations: Option<bool>,
    pub discovery_mode: Option<DiscoveryMode>,
    pub volume_normalization: Option<bool>,
    pub crossfade_duration: Option<f64>,
    pub repeat_mode: Option<RepeatMode>,
    pub shuffle_enabled: Option<bool>,
}

/// Authenticated user information from JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub keycloak_id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
}

impl AuthenticatedUser {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("stepheybot-admin")
    }

    /// Check if user can manage other users
    pub fn can_manage_users(&self) -> bool {
        self.is_admin() || self.has_role("user-manager")
    }
}

/// User role enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    /// Get all roles that have equal or higher privileges
    pub fn get_allowed_roles(&self) -> Vec<UserRole> {
        match self {
            UserRole::Admin => vec![UserRole::Admin, UserRole::User, UserRole::Guest],
            UserRole::User => vec![UserRole::User, UserRole::Guest],
            UserRole::Guest => vec![UserRole::Guest],
        }
    }
}

/// User search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchQuery {
    pub query: Option<String>,
    pub privacy_level: Option<PrivacyLevel>,
    pub is_active: Option<bool>,
    pub has_playlists: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// User search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub users: Vec<UserProfileWithStats>,
    pub total_count: i64,
    pub has_more: bool,
}

/// User activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: i64,
    pub recent_listens: i64,
    pub recent_favorites: i64,
    pub recent_playlists: i64,
    pub recent_follows: i64,
    pub listening_streak_days: i64,
    pub total_listening_time: i64,
    pub favorite_genres: Vec<String>,
    pub favorite_artists: Vec<String>,
}

/// User statistics for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboardStats {
    pub total_listening_time: i64,
    pub tracks_played: i64,
    pub favorite_tracks: i64,
    pub playlists_created: i64,
    pub followers: i64,
    pub following: i64,
    pub listening_streak: i64,
    pub top_genres: Vec<GenreStats>,
    pub top_artists: Vec<ArtistStats>,
    pub recent_activity: Vec<ActivityEntry>,
}

/// Genre statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreStats {
    pub genre: String,
    pub listen_count: i64,
    pub total_time: i64,
    pub percentage: f64,
}

/// Artist statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistStats {
    pub artist: String,
    pub listen_count: i64,
    pub total_time: i64,
    pub favorite_tracks: i64,
}

/// Activity entry for user feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub activity_type: ActivityType,
    pub timestamp: i64,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

/// Types of user activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    TrackPlayed,
    TrackFavorited,
    PlaylistCreated,
    PlaylistShared,
    UserFollowed,
    IntegrationConnected,
    RecommendationAccepted,
}

/// Error types for user operations
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),

    #[error("Username already exists: {0}")]
    UsernameExists(String),

    #[error("Email already exists: {0}")]
    EmailExists(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for user operations
pub type UserResult<T> = Result<T, UserError>;

/// Validation utilities for user data
pub mod validation {
    use super::*;
    use regex::Regex;

    /// Validate username format
    pub fn validate_username(username: &str) -> UserResult<()> {
        if username.is_empty() {
            return Err(UserError::InvalidUsername(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() < 3 {
            return Err(UserError::InvalidUsername(
                "Username must be at least 3 characters".to_string(),
            ));
        }

        if username.len() > 30 {
            return Err(UserError::InvalidUsername(
                "Username must be at most 30 characters".to_string(),
            ));
        }

        let username_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if !username_regex.is_match(username) {
            return Err(UserError::InvalidUsername(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate email format
    pub fn validate_email(email: &str) -> UserResult<()> {
        if email.is_empty() {
            return Err(UserError::InvalidEmail("Email cannot be empty".to_string()));
        }

        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(email) {
            return Err(UserError::InvalidEmail("Invalid email format".to_string()));
        }

        Ok(())
    }

    /// Validate display name
    pub fn validate_display_name(display_name: &str) -> UserResult<()> {
        if display_name.len() > 50 {
            return Err(UserError::Validation(
                "Display name must be at most 50 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate bio length
    pub fn validate_bio(bio: &str) -> UserResult<()> {
        if bio.len() > 500 {
            return Err(UserError::Validation(
                "Bio must be at most 500 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate website URL
    pub fn validate_website(website: &str) -> UserResult<()> {
        if website.is_empty() {
            return Ok(());
        }

        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if !url_regex.is_match(website) {
            return Err(UserError::Validation(
                "Invalid website URL format".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.theme, Some("neon".to_string()));
        assert!(prefs.scrobble_enabled);
        assert!(prefs.auto_recommendations);
    }

    #[test]
    fn test_privacy_level_conversion() {
        let mut profile = UserProfile {
            user_id: 1,
            bio: None,
            avatar_url: None,
            location: None,
            website: None,
            privacy_level: 1,
            share_listening_history: false,
            share_playlists: true,
            updated_at: 0,
        };

        assert!(matches!(profile.privacy_level(), PrivacyLevel::Friends));

        profile.set_privacy_level(PrivacyLevel::Public);
        assert_eq!(profile.privacy_level, 2);
    }

    #[test]
    fn test_username_validation() {
        use validation::*;

        assert!(validate_username("valid_user123").is_ok());
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("user@example").is_err()); // Invalid character
        assert!(validate_username("").is_err()); // Empty
    }

    #[test]
    fn test_email_validation() {
        use validation::*;

        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_authenticated_user_roles() {
        let user = AuthenticatedUser {
            id: 1,
            keycloak_id: "test".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: None,
            roles: vec!["admin".to_string(), "user".to_string()],
            is_active: true,
        };

        assert!(user.has_role("admin"));
        assert!(user.has_role("user"));
        assert!(!user.has_role("guest"));
        assert!(user.is_admin());
        assert!(user.can_manage_users());
    }
}
