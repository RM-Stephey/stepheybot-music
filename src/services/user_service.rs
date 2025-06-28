//! User service for managing user operations in StepheyBot Music
//!
//! This service provides comprehensive user management functionality including
//! authentication, profile management, preferences, and user statistics.

use crate::database::Database;
use crate::models::user::{
    ActivityEntry, ActivityType, CreateUserRequest, UpdateUserPreferencesRequest,
    UpdateUserRequest, User, UserDashboardStats, UserError, UserIntegration, UserPreferences,
    UserProfileWithStats, UserResult, UserSearchQuery, UserSearchResult,
};

use anyhow::Result;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Service for managing user operations
#[derive(Clone)]
pub struct UserService {
    database: Arc<Database>,
}

impl UserService {
    /// Create a new user service
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> UserResult<User> {
        debug!("Creating user: {}", request.username);

        // Validate input
        crate::models::user::validation::validate_username(&request.username)?;
        crate::models::user::validation::validate_email(&request.email)?;

        if let Some(ref display_name) = request.display_name {
            crate::models::user::validation::validate_display_name(display_name)?;
        }

        let pool = self.database.pool();
        let mut tx = pool.begin().await?;

        // Check if username already exists
        let existing_username = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = ? OR email = ?",
        )
        .bind(&request.username)
        .bind(&request.email)
        .fetch_one(&mut *tx)
        .await?;

        if existing_username > 0 {
            return Err(UserError::UsernameExists(request.username));
        }

        // Check if Keycloak ID already exists
        let existing_keycloak =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE keycloak_id = ?")
                .bind(&request.keycloak_id)
                .fetch_one(&mut *tx)
                .await?;

        if existing_keycloak > 0 {
            return Err(UserError::Validation(format!(
                "User with Keycloak ID {} already exists",
                request.keycloak_id
            )));
        }

        // Insert user
        let user_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO users (keycloak_id, username, email, display_name, preferences_json)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&request.keycloak_id)
        .bind(&request.username)
        .bind(&request.email)
        .bind(&request.display_name)
        .bind(serde_json::to_string(&UserPreferences::default())?)
        .fetch_one(&mut *tx)
        .await?;

        // Create default user profile
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, privacy_level, share_listening_history, share_playlists)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(1) // Friends privacy level by default
        .bind(false)
        .bind(true)
        .execute(&mut *tx)
        .await?;

        // Create default taste profile
        sqlx::query(
            r#"
            INSERT INTO user_taste_profiles (user_id, favorite_genres_json, favorite_artists_json)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind("[]")
        .bind("[]")
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "User created successfully: {} (ID: {})",
            request.username, user_id
        );

        // Fetch and return the created user
        self.get_user_by_id(user_id).await
    }

    /// Get or create user from Keycloak claims
    pub async fn get_or_create_user_from_keycloak(
        &self,
        claims: &crate::auth::KeycloakClaims,
    ) -> UserResult<User> {
        // Try to get existing user by Keycloak ID
        if let Ok(user) = self.get_user_by_keycloak_id(&claims.sub).await {
            // Update last_active timestamp
            self.update_user_last_active(user.id).await?;
            return Ok(user);
        }

        // User doesn't exist, create new one
        let create_request = CreateUserRequest {
            keycloak_id: claims.sub.clone(),
            username: claims.preferred_username.clone(),
            email: claims.email.clone(),
            display_name: claims.name.clone(),
        };

        self.create_user(create_request).await
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: i64) -> UserResult<User> {
        debug!("Getting user by ID: {}", user_id);

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(self.database.pool())
            .await?
            .ok_or_else(|| UserError::NotFound(format!("User ID: {}", user_id)))?;

        Ok(user)
    }

    /// Get user by Keycloak ID
    pub async fn get_user_by_keycloak_id(&self, keycloak_id: &str) -> UserResult<User> {
        debug!("Getting user by Keycloak ID: {}", keycloak_id);

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE keycloak_id = ?")
            .bind(keycloak_id)
            .fetch_optional(self.database.pool())
            .await?
            .ok_or_else(|| UserError::NotFound(format!("Keycloak ID: {}", keycloak_id)))?;

        Ok(user)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> UserResult<User> {
        debug!("Getting user by username: {}", username);

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(self.database.pool())
            .await?
            .ok_or_else(|| UserError::NotFound(format!("Username: {}", username)))?;

        Ok(user)
    }

    /// Get user profile with statistics
    pub async fn get_user_profile_with_stats(
        &self,
        user_id: i64,
    ) -> UserResult<UserProfileWithStats> {
        debug!("Getting user profile with stats: {}", user_id);

        let profile = sqlx::query_as::<_, UserProfileWithStats>(
            "SELECT * FROM v_user_profiles_with_stats WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(self.database.pool())
        .await?
        .ok_or_else(|| UserError::NotFound(format!("User profile: {}", user_id)))?;

        Ok(profile)
    }

    /// Update user profile
    pub async fn update_user_profile(
        &self,
        user_id: i64,
        request: UpdateUserRequest,
    ) -> UserResult<()> {
        debug!("Updating user profile: {}", user_id);

        // Validate input
        if let Some(ref display_name) = request.display_name {
            crate::models::user::validation::validate_display_name(display_name)?;
        }

        if let Some(ref bio) = request.bio {
            crate::models::user::validation::validate_bio(bio)?;
        }

        if let Some(ref website) = request.website {
            crate::models::user::validation::validate_website(website)?;
        }

        let pool = self.database.pool();
        let mut tx = pool.begin().await?;

        // Update user table
        if request.display_name.is_some() {
            sqlx::query("UPDATE users SET display_name = ? WHERE id = ?")
                .bind(&request.display_name)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        // Update user profile table with individual queries
        if let Some(ref bio) = request.bio {
            sqlx::query("UPDATE user_profiles SET bio = ? WHERE user_id = ?")
                .bind(bio)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(ref location) = request.location {
            sqlx::query("UPDATE user_profiles SET location = ? WHERE user_id = ?")
                .bind(location)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(ref website) = request.website {
            sqlx::query("UPDATE user_profiles SET website = ? WHERE user_id = ?")
                .bind(website)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(privacy_level) = request.privacy_level {
            let level = privacy_level as i32;
            sqlx::query("UPDATE user_profiles SET privacy_level = ? WHERE user_id = ?")
                .bind(level)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(share_listening_history) = request.share_listening_history {
            sqlx::query("UPDATE user_profiles SET share_listening_history = ? WHERE user_id = ?")
                .bind(share_listening_history)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(share_playlists) = request.share_playlists {
            sqlx::query("UPDATE user_profiles SET share_playlists = ? WHERE user_id = ?")
                .bind(share_playlists)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        info!("User profile updated successfully: {}", user_id);
        Ok(())
    }

    /// Update user preferences
    pub async fn update_user_preferences(
        &self,
        user_id: i64,
        request: UpdateUserPreferencesRequest,
    ) -> UserResult<()> {
        debug!("Updating user preferences: {}", user_id);

        // Get current preferences
        let current_user = self.get_user_by_id(user_id).await?;
        let mut preferences = current_user.preferences()?;

        // Update preferences with provided values
        if let Some(theme) = request.theme {
            preferences.theme = Some(theme);
        }

        if let Some(language) = request.language {
            preferences.language = Some(language);
        }

        if let Some(timezone) = request.timezone {
            preferences.timezone = Some(timezone);
        }

        if let Some(email_notifications) = request.email_notifications {
            preferences.email_notifications = email_notifications;
        }

        if let Some(scrobble_enabled) = request.scrobble_enabled {
            preferences.scrobble_enabled = scrobble_enabled;
        }

        if let Some(auto_recommendations) = request.auto_recommendations {
            preferences.auto_recommendations = auto_recommendations;
        }

        if let Some(discovery_mode) = request.discovery_mode {
            preferences.discovery_mode = discovery_mode;
        }

        if let Some(volume_normalization) = request.volume_normalization {
            preferences.volume_normalization = volume_normalization;
        }

        if let Some(crossfade_duration) = request.crossfade_duration {
            preferences.crossfade_duration = Some(crossfade_duration);
        }

        if let Some(repeat_mode) = request.repeat_mode {
            preferences.repeat_mode = repeat_mode;
        }

        if let Some(shuffle_enabled) = request.shuffle_enabled {
            preferences.shuffle_enabled = shuffle_enabled;
        }

        // Save updated preferences
        sqlx::query("UPDATE users SET preferences_json = ? WHERE id = ?")
            .bind(serde_json::to_string(&preferences)?)
            .bind(user_id)
            .execute(self.database.pool())
            .await?;

        info!("User preferences updated successfully: {}", user_id);
        Ok(())
    }

    /// Search users based on criteria
    pub async fn search_users(&self, query: UserSearchQuery) -> UserResult<UserSearchResult> {
        debug!("Searching users with query: {:?}", query);

        let limit = query.limit.unwrap_or(20).min(100); // Max 100 results
        let offset = query.offset.unwrap_or(0);

        // Build query based on search parameters
        let (total_count, users) = if let Some(ref search_query) = query.query {
            let search_pattern = format!("%{}%", search_query);
            let where_clause = "WHERE (username LIKE ? OR display_name LIKE ? OR email LIKE ?)";

            let total_count = sqlx::query_scalar::<_, i64>(&format!(
                "SELECT COUNT(*) FROM v_user_profiles_with_stats {}",
                where_clause
            ))
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_one(self.database.pool())
            .await?;

            let users = sqlx::query_as::<_, UserProfileWithStats>(&format!(
                "SELECT * FROM v_user_profiles_with_stats {} ORDER BY username LIMIT ? OFFSET ?",
                where_clause
            ))
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.database.pool())
            .await?;

            (total_count, users)
        } else if let Some(privacy_level) = query.privacy_level {
            let level = privacy_level as i32;
            let where_clause = "WHERE privacy_level = ?";

            let total_count = sqlx::query_scalar::<_, i64>(&format!(
                "SELECT COUNT(*) FROM v_user_profiles_with_stats {}",
                where_clause
            ))
            .bind(level)
            .fetch_one(self.database.pool())
            .await?;

            let users = sqlx::query_as::<_, UserProfileWithStats>(&format!(
                "SELECT * FROM v_user_profiles_with_stats {} ORDER BY username LIMIT ? OFFSET ?",
                where_clause
            ))
            .bind(level)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.database.pool())
            .await?;

            (total_count, users)
        } else {
            // No filters
            let total_count =
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v_user_profiles_with_stats")
                    .fetch_one(self.database.pool())
                    .await?;

            let users = sqlx::query_as::<_, UserProfileWithStats>(
                "SELECT * FROM v_user_profiles_with_stats ORDER BY username LIMIT ? OFFSET ?",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.database.pool())
            .await?;

            (total_count, users)
        };

        let has_more = offset + (users.len() as i64) < total_count;

        Ok(UserSearchResult {
            users,
            total_count,
            has_more,
        })
    }

    /// Get user dashboard statistics
    pub async fn get_user_dashboard_stats(&self, user_id: i64) -> UserResult<UserDashboardStats> {
        debug!("Getting dashboard stats for user: {}", user_id);

        let pool = self.database.pool();

        // Get basic listening statistics
        let listening_stats = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(duration_played), 0) as total_listening_time,
                COUNT(*) as tracks_played,
                COUNT(DISTINCT track_path) as unique_tracks
            FROM listening_sessions
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let total_listening_time = listening_stats.get::<i64, _>("total_listening_time");
        let tracks_played = listening_stats.get::<i64, _>("tracks_played");

        // Get favorites count
        let favorite_tracks =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_favorites WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(pool)
                .await?;

        // Get playlists count
        let playlists_created =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_playlists WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(pool)
                .await?;

        // Get social stats
        let followers = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_follows WHERE following_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let following =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_follows WHERE follower_id = ?")
                .bind(user_id)
                .fetch_one(pool)
                .await?;

        // Calculate listening streak (simplified)
        let listening_streak = self.calculate_listening_streak(user_id).await?;

        // Get top genres (placeholder - would need genre data from tracks)
        let top_genres = Vec::new(); // TODO: Implement when genre data is available

        // Get top artists (placeholder - would need artist data from tracks)
        let top_artists = Vec::new(); // TODO: Implement when artist data is available

        // Get recent activity
        let recent_activity = self.get_recent_activity(user_id, 10).await?;

        Ok(UserDashboardStats {
            total_listening_time,
            tracks_played,
            favorite_tracks,
            playlists_created,
            followers,
            following,
            listening_streak,
            top_genres,
            top_artists,
            recent_activity,
        })
    }

    /// Update user's last active timestamp
    pub async fn update_user_last_active(&self, user_id: i64) -> UserResult<()> {
        sqlx::query("UPDATE users SET last_active = strftime('%s', 'now') WHERE id = ?")
            .bind(user_id)
            .execute(self.database.pool())
            .await?;

        Ok(())
    }

    /// Deactivate user account
    pub async fn deactivate_user(&self, user_id: i64) -> UserResult<()> {
        info!("Deactivating user: {}", user_id);

        sqlx::query("UPDATE users SET is_active = 0 WHERE id = ?")
            .bind(user_id)
            .execute(self.database.pool())
            .await?;

        Ok(())
    }

    /// Reactivate user account
    pub async fn reactivate_user(&self, user_id: i64) -> UserResult<()> {
        info!("Reactivating user: {}", user_id);

        sqlx::query("UPDATE users SET is_active = 1 WHERE id = ?")
            .bind(user_id)
            .execute(self.database.pool())
            .await?;

        Ok(())
    }

    /// Delete user account and all associated data
    pub async fn delete_user(&self, user_id: i64) -> UserResult<()> {
        warn!("Deleting user and all associated data: {}", user_id);

        let pool = self.database.pool();
        let mut tx = pool.begin().await?;

        // Foreign key constraints will handle cascading deletes
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        info!("User deleted successfully: {}", user_id);
        Ok(())
    }

    /// Get user's integrations
    pub async fn get_user_integrations(&self, user_id: i64) -> UserResult<Vec<UserIntegration>> {
        let integrations = sqlx::query_as::<_, UserIntegration>(
            "SELECT * FROM user_integrations WHERE user_id = ? ORDER BY service_name",
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?;

        Ok(integrations)
    }

    /// Check if username is available
    pub async fn is_username_available(&self, username: &str) -> UserResult<bool> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE username = ?")
            .bind(username)
            .fetch_one(self.database.pool())
            .await?;

        Ok(count == 0)
    }

    /// Check if email is available
    pub async fn is_email_available(&self, email: &str) -> UserResult<bool> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(self.database.pool())
            .await?;

        Ok(count == 0)
    }

    /// Calculate user's listening streak in days
    async fn calculate_listening_streak(&self, user_id: i64) -> UserResult<i64> {
        // Simplified implementation - count consecutive days with listening activity
        let rows = sqlx::query(
            r#"
            SELECT DATE(started_at, 'unixepoch') as listen_date
            FROM listening_sessions
            WHERE user_id = ?
            GROUP BY DATE(started_at, 'unixepoch')
            ORDER BY listen_date DESC
            LIMIT 30
            "#,
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?;

        if rows.is_empty() {
            return Ok(0);
        }

        // For now, just return the number of days with activity in the last 30 days
        // TODO: Implement proper consecutive day calculation
        Ok(rows.len() as i64)
    }

    /// Get recent activity for user
    async fn get_recent_activity(
        &self,
        user_id: i64,
        limit: i64,
    ) -> UserResult<Vec<ActivityEntry>> {
        // This is a simplified implementation
        // In a real system, you'd have an activity log table
        let mut activities = Vec::new();

        // Get recent listening sessions
        let recent_listens = sqlx::query(
            r#"
            SELECT track_path, started_at
            FROM listening_sessions
            WHERE user_id = ? AND completed = 1
            ORDER BY started_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit / 2)
        .fetch_all(self.database.pool())
        .await?;

        for row in recent_listens {
            let track_path: String = row.get("track_path");
            let timestamp: i64 = row.get("started_at");

            let mut metadata = HashMap::new();
            metadata.insert("track_path".to_string(), track_path.clone());

            activities.push(ActivityEntry {
                activity_type: ActivityType::TrackPlayed,
                timestamp,
                description: format!("Played track: {}", track_path),
                metadata,
            });
        }

        // Get recent favorites
        let recent_favorites = sqlx::query(
            r#"
            SELECT track_path, favorited_at
            FROM user_favorites
            WHERE user_id = ?
            ORDER BY favorited_at DESC
            LIMIT ?
            "#,
        )
        .bind(user_id)
        .bind(limit / 2)
        .fetch_all(self.database.pool())
        .await?;

        for row in recent_favorites {
            let track_path: String = row.get("track_path");
            let timestamp: i64 = row.get("favorited_at");

            let mut metadata = HashMap::new();
            metadata.insert("track_path".to_string(), track_path.clone());

            activities.push(ActivityEntry {
                activity_type: ActivityType::TrackFavorited,
                timestamp,
                description: format!("Favorited track: {}", track_path),
                metadata,
            });
        }

        // Sort by timestamp descending
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit results
        activities.truncate(limit as usize);

        Ok(activities)
    }

    /// Get user count
    pub async fn get_user_count(&self) -> UserResult<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(self.database.pool())
            .await?;

        Ok(count)
    }

    /// Get active user count (active in last 30 days)
    pub async fn get_active_user_count(&self) -> UserResult<i64> {
        let thirty_days_ago = chrono::Utc::now().timestamp() - (30 * 24 * 60 * 60);

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE is_active = 1 AND last_active > ?",
        )
        .bind(thirty_days_ago)
        .fetch_one(self.database.pool())
        .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use tempfile::NamedTempFile;

    async fn create_test_service() -> UserService {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());
        let database = Arc::new(Database::new(&db_url).await.unwrap());
        database.migrate().await.unwrap();
        UserService::new(database)
    }

    #[tokio::test]
    async fn test_create_user() {
        let service = create_test_service().await;

        let request = CreateUserRequest {
            keycloak_id: "test-keycloak-id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
        };

        let user = service.create_user(request).await.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let service = create_test_service().await;

        let request = CreateUserRequest {
            keycloak_id: "test-keycloak-id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: None,
        };

        let created_user = service.create_user(request).await.unwrap();
        let retrieved_user = service.get_user_by_id(created_user.id).await.unwrap();

        assert_eq!(created_user.id, retrieved_user.id);
        assert_eq!(created_user.username, retrieved_user.username);
    }

    #[tokio::test]
    async fn test_username_availability() {
        let service = create_test_service().await;

        assert!(service.is_username_available("newuser").await.unwrap());

        let request = CreateUserRequest {
            keycloak_id: "test-keycloak-id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: None,
        };

        service.create_user(request).await.unwrap();

        assert!(!service.is_username_available("testuser").await.unwrap());
        assert!(service.is_username_available("anotheruser").await.unwrap());
    }

    #[tokio::test]
    async fn test_user_preferences() {
        let service = create_test_service().await;

        let request = CreateUserRequest {
            keycloak_id: "test-keycloak-id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: None,
        };

        let user = service.create_user(request).await.unwrap();

        // Test updating preferences
        let update_request = UpdateUserPreferencesRequest {
            theme: Some("dark".to_string()),
            scrobble_enabled: Some(false),
            auto_recommendations: Some(false),
            ..Default::default()
        };

        service
            .update_user_preferences(user.id, update_request)
            .await
            .unwrap();

        let updated_user = service.get_user_by_id(user.id).await.unwrap();
        let preferences = updated_user.preferences().unwrap();

        assert_eq!(preferences.theme, Some("dark".to_string()));
        assert!(!preferences.scrobble_enabled);
        assert!(!preferences.auto_recommendations);
    }
}
