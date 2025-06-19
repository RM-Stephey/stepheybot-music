//! Sync service for StepheyBot Music
//!
//! This module handles synchronization between different music services,
//! including Navidrome and ListenBrainz data synchronization.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::clients::{listenbrainz::ListenBrainzClient, navidrome::NavidromeClient};
use crate::database::Database;
use crate::models::entities::{ListeningHistory, User};
use crate::services::{Service, SyncStats};
use crate::utils;

/// Sync service for managing data synchronization
#[derive(Clone)]
pub struct SyncService {
    database: Arc<Database>,
    navidrome_client: Arc<NavidromeClient>,
    listenbrainz_client: Arc<ListenBrainzClient>,
}

/// User synchronization result
#[derive(Debug, Clone)]
pub struct UserSyncResult {
    pub user_id: String,
    pub username: String,
    pub listens_synced: u32,
    pub errors: u32,
    pub last_sync_time: DateTime<Utc>,
}

/// Sync operation result
#[derive(Debug, Clone)]
pub struct SyncOperationResult {
    pub total_users_processed: u32,
    pub successful_syncs: u32,
    pub failed_syncs: u32,
    pub total_listens_synced: u32,
    pub duration_seconds: f64,
    pub errors: Vec<String>,
}

impl SyncService {
    /// Create a new sync service
    pub fn new(
        database: Arc<Database>,
        navidrome_client: Arc<NavidromeClient>,
        listenbrainz_client: Arc<ListenBrainzClient>,
    ) -> Result<Self> {
        Ok(Self {
            database,
            navidrome_client,
            listenbrainz_client,
        })
    }

    /// Sync all users' listening data
    pub async fn sync_all_users(&self) -> Result<SyncOperationResult> {
        let start_time = std::time::Instant::now();
        info!("Starting sync operation for all users");

        let mut result = SyncOperationResult {
            total_users_processed: 0,
            successful_syncs: 0,
            failed_syncs: 0,
            total_listens_synced: 0,
            duration_seconds: 0.0,
            errors: Vec::new(),
        };

        // Get all active users
        let users = match self.get_active_users().await {
            Ok(users) => users,
            Err(e) => {
                error!("Failed to get active users: {}", e);
                result.errors.push(format!("Failed to get users: {}", e));
                result.duration_seconds = start_time.elapsed().as_secs_f64();
                return Ok(result);
            }
        };

        result.total_users_processed = users.len() as u32;
        info!("Found {} active users to sync", users.len());

        // Sync each user
        for user in users {
            match self.sync_user(&user).await {
                Ok(user_result) => {
                    result.successful_syncs += 1;
                    result.total_listens_synced += user_result.listens_synced;
                    debug!(
                        "Successfully synced user {}: {} listens",
                        user.username, user_result.listens_synced
                    );
                }
                Err(e) => {
                    result.failed_syncs += 1;
                    let error_msg = format!("Failed to sync user {}: {}", user.username, e);
                    warn!("{}", error_msg);
                    result.errors.push(error_msg);
                }
            }
        }

        result.duration_seconds = start_time.elapsed().as_secs_f64();

        info!(
            "Sync operation completed: {}/{} users synced successfully, {} total listens, {:.2}s",
            result.successful_syncs,
            result.total_users_processed,
            result.total_listens_synced,
            result.duration_seconds
        );

        // Update sync statistics in database
        if let Err(e) = self.update_sync_stats(&result).await {
            warn!("Failed to update sync statistics: {}", e);
        }

        Ok(result)
    }

    /// Sync a single user's data
    pub async fn sync_user(&self, user: &User) -> Result<UserSyncResult> {
        debug!("Syncing user: {}", user.username);

        let mut result = UserSyncResult {
            user_id: user.id.clone(),
            username: user.username.clone(),
            listens_synced: 0,
            errors: 0,
            last_sync_time: utils::now(),
        };

        // Get last sync time for this user
        let last_sync = self.get_last_sync_time(&user.id).await?;

        // Get recent listening history from Navidrome
        let recent_plays = match self
            .navidrome_client
            .get_recent_plays(&user.id, last_sync)
            .await
        {
            Ok(plays) => plays,
            Err(e) => {
                warn!(
                    "Failed to get recent plays for user {}: {}",
                    user.username, e
                );
                result.errors += 1;
                return Ok(result);
            }
        };

        if recent_plays.is_empty() {
            debug!("No new plays found for user {}", user.username);
            self.update_last_sync_time(&user.id, result.last_sync_time)
                .await?;
            return Ok(result);
        }

        // Convert Navidrome plays to ListenBrainz format
        let listens = self.convert_plays_to_listens(&recent_plays).await?;

        if !listens.is_empty() {
            // Submit to ListenBrainz
            match self
                .listenbrainz_client
                .batch_submit_listens(&user.username, listens, 50)
                .await
            {
                Ok(_) => {
                    result.listens_synced = recent_plays.len() as u32;
                    debug!(
                        "Successfully submitted {} listens for user {}",
                        result.listens_synced, user.username
                    );
                }
                Err(e) => {
                    warn!("Failed to submit listens for user {}: {}", user.username, e);
                    result.errors += 1;
                }
            }
        }

        // Store listening history in our database
        if let Err(e) = self.store_listening_history(&user.id, &recent_plays).await {
            warn!(
                "Failed to store listening history for user {}: {}",
                user.username, e
            );
            result.errors += 1;
        }

        // Update last sync time
        self.update_last_sync_time(&user.id, result.last_sync_time)
            .await?;

        Ok(result)
    }

    /// Get all active users from the database
    async fn get_active_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE is_active = true")
            .fetch_all(self.database.pool())
            .await
            .context("Failed to fetch active users")?;

        Ok(users)
    }

    /// Get the last sync time for a user
    async fn get_last_sync_time(&self, user_id: &str) -> Result<DateTime<Utc>> {
        // Try to get from user preferences or use a default
        let last_sync: Option<String> = sqlx::query_scalar(
            "SELECT preference_value FROM user_preferences WHERE user_id = ? AND preference_key = 'last_sync_time'",
        )
        .bind(user_id)
        .fetch_optional(self.database.pool())
        .await?;

        if let Some(sync_time_str) = last_sync {
            if let Ok(sync_time) = sync_time_str.parse::<DateTime<Utc>>() {
                return Ok(sync_time);
            }
        }

        // Default to 7 days ago if no previous sync
        Ok(utils::now() - chrono::Duration::days(7))
    }

    /// Update the last sync time for a user
    async fn update_last_sync_time(&self, user_id: &str, sync_time: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_preferences (user_id, preference_key, preference_value, preference_type, created_at, updated_at)
            VALUES (?, 'last_sync_time', ?, 'string', ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(sync_time.to_rfc3339())
        .bind(utils::now())
        .bind(utils::now())
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    /// Convert Navidrome plays to ListenBrainz listens
    async fn convert_plays_to_listens(
        &self,
        plays: &[crate::clients::navidrome::ScrobbleEntry],
    ) -> Result<Vec<crate::clients::listenbrainz::Listen>> {
        let mut listens = Vec::new();

        for play in plays {
            // In a real implementation, you'd need to get track metadata
            // For now, create a basic listen structure
            let listen = self.listenbrainz_client.create_listen_from_play(
                "Unknown Artist", // Would fetch from database
                "Unknown Track",  // Would fetch from database
                None,             // Album name
                None,             // Duration
                DateTime::from_timestamp((play.time / 1000) as i64, 0)
                    .unwrap_or_else(|| utils::now()),
            );

            listens.push(listen);
        }

        Ok(listens)
    }

    /// Store listening history in the database
    async fn store_listening_history(
        &self,
        user_id: &str,
        plays: &[crate::clients::navidrome::ScrobbleEntry],
    ) -> Result<()> {
        let mut tx = self.database.begin_transaction().await?;

        for play in plays {
            // Create listening history entry
            let history = ListeningHistory::new(user_id.to_string(), play.id.clone());

            sqlx::query(
                r#"
                INSERT OR IGNORE INTO listening_history
                (user_id, track_id, played_at, created_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(&history.user_id)
            .bind(&history.track_id)
            .bind(history.played_at)
            .bind(history.created_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Update sync statistics in the database
    async fn update_sync_stats(&self, result: &SyncOperationResult) -> Result<()> {
        // Store sync stats for monitoring
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_preferences
            (user_id, preference_key, preference_value, preference_type, created_at, updated_at)
            VALUES ('system', 'last_sync_stats', ?, 'json', ?, ?)
            "#,
        )
        .bind(serde_json::to_string(result)?)
        .bind(utils::now())
        .bind(utils::now())
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    /// Sync user accounts from Navidrome
    pub async fn sync_user_accounts(&self) -> Result<u32> {
        info!("Syncing user accounts from Navidrome");

        let navidrome_users = self.navidrome_client.get_users().await?;
        let mut synced_count = 0;

        for nav_user in navidrome_users {
            // Check if user already exists
            let existing_user: Option<User> =
                sqlx::query_as("SELECT * FROM users WHERE navidrome_id = ? OR username = ?")
                    .bind(&nav_user.username) // Using username as navidrome_id for simplicity
                    .bind(&nav_user.username)
                    .fetch_optional(self.database.pool())
                    .await?;

            if existing_user.is_none() {
                // Create new user
                let user = User::new(nav_user.username.clone(), nav_user.username.clone());

                sqlx::query(
                    r#"
                    INSERT INTO users (id, navidrome_id, username, display_name, email, is_admin, is_active, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&user.id)
                .bind(&nav_user.username)
                .bind(&nav_user.username)
                .bind(&nav_user.username)
                .bind(nav_user.email.as_deref())
                .bind(nav_user.admin_role.unwrap_or(false))
                .bind(true)
                .bind(user.created_at)
                .bind(user.updated_at)
                .execute(self.database.pool())
                .await?;

                synced_count += 1;
                info!("Created new user: {}", nav_user.username);
            } else {
                // Update existing user if needed
                sqlx::query(
                    "UPDATE users SET email = ?, is_admin = ?, updated_at = ? WHERE username = ?",
                )
                .bind(nav_user.email.as_deref())
                .bind(nav_user.admin_role.unwrap_or(false))
                .bind(utils::now())
                .bind(&nav_user.username)
                .execute(self.database.pool())
                .await?;

                debug!("Updated existing user: {}", nav_user.username);
            }
        }

        info!("Synced {} user accounts", synced_count);
        Ok(synced_count)
    }

    /// Force sync a specific user
    pub async fn force_sync_user(&self, user_id: &str) -> Result<UserSyncResult> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(self.database.pool())
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", user_id))?;

        info!("Force syncing user: {}", user.username);
        self.sync_user(&user).await
    }

    /// Get sync statistics
    pub async fn get_sync_statistics(&self) -> Result<SyncStats> {
        let last_sync_str: Option<String> = sqlx::query_scalar(
            "SELECT preference_value FROM user_preferences WHERE user_id = 'system' AND preference_key = 'last_sync_stats'",
        )
        .fetch_optional(self.database.pool())
        .await?;

        let last_sync = if let Some(stats_str) = last_sync_str {
            match serde_json::from_str::<SyncOperationResult>(&stats_str) {
                Ok(stats) => Some(
                    DateTime::from_timestamp(stats.duration_seconds as i64, 0)
                        .unwrap_or_else(|| utils::now()),
                ),
                Err(_) => None,
            }
        } else {
            None
        };

        let total_synced_plays: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM listening_history")
            .fetch_one(self.database.pool())
            .await?;

        let sync_errors: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_preferences WHERE preference_key LIKE 'sync_error_%'",
        )
        .fetch_one(self.database.pool())
        .await?;

        let users_synced: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_id) FROM user_preferences WHERE preference_key = 'last_sync_time'",
        )
        .fetch_one(self.database.pool())
        .await?;

        Ok(SyncStats {
            last_sync,
            total_synced_plays: total_synced_plays as u64,
            sync_errors: sync_errors as u64,
            users_synced: users_synced as u64,
        })
    }

    /// Clean up old sync data
    pub async fn cleanup_old_sync_data(&self, retention_days: u32) -> Result<u32> {
        let cutoff_date = utils::now() - chrono::Duration::days(retention_days as i64);

        let deleted_rows = sqlx::query("DELETE FROM listening_history WHERE created_at < ?")
            .bind(cutoff_date)
            .execute(self.database.pool())
            .await?
            .rows_affected();

        info!("Cleaned up {} old listening history records", deleted_rows);
        Ok(deleted_rows as u32)
    }
}

#[async_trait::async_trait]
impl Service for SyncService {
    type Stats = SyncStats;

    async fn health_check(&self) -> Result<()> {
        // Check database connectivity
        self.database
            .health_check()
            .await
            .context("Database health check failed")?;

        // Check Navidrome connectivity
        self.navidrome_client
            .health_check()
            .await
            .context("Navidrome health check failed")?;

        // Check ListenBrainz connectivity
        self.listenbrainz_client
            .health_check()
            .await
            .context("ListenBrainz health check failed")?;

        Ok(())
    }

    async fn get_stats(&self) -> Result<Self::Stats> {
        self.get_sync_statistics().await
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Sync service shutting down");
        // No cleanup needed for this service
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_operation_result() {
        let result = SyncOperationResult {
            total_users_processed: 5,
            successful_syncs: 4,
            failed_syncs: 1,
            total_listens_synced: 150,
            duration_seconds: 12.5,
            errors: vec!["Test error".to_string()],
        };

        assert_eq!(result.total_users_processed, 5);
        assert_eq!(result.successful_syncs, 4);
        assert_eq!(result.total_listens_synced, 150);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_user_sync_result() {
        let result = UserSyncResult {
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            listens_synced: 25,
            errors: 0,
            last_sync_time: utils::now(),
        };

        assert_eq!(result.user_id, "user123");
        assert_eq!(result.username, "testuser");
        assert_eq!(result.listens_synced, 25);
        assert_eq!(result.errors, 0);
    }
}
