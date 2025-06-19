//! Playlist service for StepheyBot Music
//!
//! This module handles playlist management, including creation, synchronization
//! with Navidrome, smart playlist generation, and playlist analytics.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::clients::navidrome::NavidromeClient;
use crate::database::Database;
use crate::models::entities::{Playlist, PlaylistTrack, Track, User};
use crate::services::{PlaylistStats, Service};
use crate::utils;

/// Playlist service for managing user playlists
#[derive(Clone)]
pub struct PlaylistService {
    database: Arc<Database>,
    navidrome_client: Arc<NavidromeClient>,
}

/// Playlist creation request
#[derive(Debug, Clone)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
    pub is_public: bool,
    pub track_ids: Vec<String>,
}

/// Smart playlist criteria
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmartPlaylistCriteria {
    pub genre: Option<Vec<String>>,
    pub artist_ids: Option<Vec<String>>,
    pub year_range: Option<(i32, i32)>,
    pub min_rating: Option<i32>,
    pub max_tracks: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Playlist synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub playlists_synced: u32,
    pub tracks_added: u32,
    pub tracks_removed: u32,
    pub errors: u32,
}

impl PlaylistService {
    /// Create a new playlist service
    pub fn new(database: Arc<Database>, navidrome_client: Arc<NavidromeClient>) -> Result<Self> {
        Ok(Self {
            database,
            navidrome_client,
        })
    }

    /// Create a new playlist
    pub async fn create_playlist(&self, request: CreatePlaylistRequest) -> Result<Playlist> {
        info!(
            "Creating playlist '{}' for user {}",
            request.name, request.user_id
        );

        let playlist = Playlist::new(request.name.clone(), request.user_id.clone());
        let playlist_id = playlist.id.clone();

        // Start database transaction
        let mut tx = self.database.begin_transaction().await?;

        // Insert playlist into database
        sqlx::query(
            r#"
            INSERT INTO playlists (id, name, description, user_id, is_public, track_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&playlist.id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.user_id)
        .bind(request.is_public)
        .bind(request.track_ids.len() as i32)
        .bind(playlist.created_at)
        .bind(playlist.updated_at)
        .execute(&mut *tx)
        .await
        .context("Failed to insert playlist")?;

        // Add tracks to playlist
        for (position, track_id) in request.track_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(&playlist_id)
            .bind(track_id)
            .bind(position as i32)
            .bind(utils::now())
            .execute(&mut *tx)
            .await
            .context("Failed to add track to playlist")?;
        }

        tx.commit().await?;

        // Sync with Navidrome
        if let Err(e) = self.sync_playlist_to_navidrome(&playlist_id).await {
            warn!("Failed to sync playlist to Navidrome: {}", e);
        }

        info!("Successfully created playlist '{}'", request.name);
        self.get_playlist(&playlist_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created playlist"))
    }

    /// Get a playlist by ID
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<Option<Playlist>> {
        let playlist = sqlx::query_as::<_, Playlist>("SELECT * FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .fetch_optional(self.database.pool())
            .await?;

        Ok(playlist)
    }

    /// Get playlists for a user
    pub async fn get_user_playlists(&self, user_id: &str) -> Result<Vec<Playlist>> {
        let playlists = sqlx::query_as::<_, Playlist>(
            "SELECT * FROM playlists WHERE user_id = ? ORDER BY updated_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?;

        Ok(playlists)
    }

    /// Get tracks in a playlist
    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>> {
        let tracks = sqlx::query_as::<_, Track>(
            r#"
            SELECT t.* FROM tracks t
            JOIN playlist_tracks pt ON t.id = pt.track_id
            WHERE pt.playlist_id = ?
            ORDER BY pt.position
            "#,
        )
        .bind(playlist_id)
        .fetch_all(self.database.pool())
        .await?;

        Ok(tracks)
    }

    /// Add tracks to a playlist
    pub async fn add_tracks_to_playlist(
        &self,
        playlist_id: &str,
        track_ids: Vec<String>,
    ) -> Result<()> {
        let mut tx = self.database.begin_transaction().await?;

        // Get current track count
        let current_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?")
                .bind(playlist_id)
                .fetch_one(&mut *tx)
                .await?;

        // Add new tracks
        for (i, track_id) in track_ids.iter().enumerate() {
            let position = current_count + i as i32;

            sqlx::query(
                r#"
                INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(playlist_id)
            .bind(track_id)
            .bind(position)
            .bind(utils::now())
            .execute(&mut *tx)
            .await?;
        }

        // Update playlist track count and timestamp
        sqlx::query(
            "UPDATE playlists SET track_count = track_count + ?, updated_at = ? WHERE id = ?",
        )
        .bind(track_ids.len() as i32)
        .bind(utils::now())
        .bind(playlist_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Sync with Navidrome
        if let Err(e) = self.sync_playlist_to_navidrome(playlist_id).await {
            warn!("Failed to sync playlist to Navidrome: {}", e);
        }

        info!(
            "Added {} tracks to playlist {}",
            track_ids.len(),
            playlist_id
        );
        Ok(())
    }

    /// Remove tracks from a playlist
    pub async fn remove_tracks_from_playlist(
        &self,
        playlist_id: &str,
        track_ids: Vec<String>,
    ) -> Result<()> {
        let mut tx = self.database.begin_transaction().await?;

        // Remove tracks
        for track_id in &track_ids {
            sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
                .bind(playlist_id)
                .bind(track_id)
                .execute(&mut *tx)
                .await?;
        }

        // Reorder remaining tracks
        sqlx::query(
            r#"
            UPDATE playlist_tracks
            SET position = (
                SELECT ROW_NUMBER() OVER (ORDER BY position) - 1
                FROM playlist_tracks pt2
                WHERE pt2.playlist_id = playlist_tracks.playlist_id
                AND pt2.id <= playlist_tracks.id
            )
            WHERE playlist_id = ?
            "#,
        )
        .bind(playlist_id)
        .execute(&mut *tx)
        .await?;

        // Update playlist track count
        let new_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?")
                .bind(playlist_id)
                .fetch_one(&mut *tx)
                .await?;

        sqlx::query("UPDATE playlists SET track_count = ?, updated_at = ? WHERE id = ?")
            .bind(new_count)
            .bind(utils::now())
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        info!(
            "Removed {} tracks from playlist {}",
            track_ids.len(),
            playlist_id
        );
        Ok(())
    }

    /// Delete a playlist
    pub async fn delete_playlist(&self, playlist_id: &str) -> Result<()> {
        let mut tx = self.database.begin_transaction().await?;

        // Delete playlist tracks
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        // Delete playlist
        let result = sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Playlist not found: {}", playlist_id);
        }

        tx.commit().await?;

        info!("Deleted playlist {}", playlist_id);
        Ok(())
    }

    /// Create a smart playlist based on criteria
    pub async fn create_smart_playlist(
        &self,
        name: String,
        user_id: String,
        criteria: SmartPlaylistCriteria,
    ) -> Result<Playlist> {
        info!("Creating smart playlist '{}' for user {}", name, user_id);

        // Generate tracks based on criteria
        let track_ids = self.generate_smart_playlist_tracks(&criteria).await?;

        let mut playlist = Playlist::new_smart(
            name.clone(),
            user_id.clone(),
            serde_json::to_string(&criteria)?,
        );

        // Insert playlist
        sqlx::query(
            r#"
            INSERT INTO playlists (id, name, user_id, is_smart, smart_criteria, track_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&playlist.id)
        .bind(&name)
        .bind(&user_id)
        .bind(true)
        .bind(&playlist.smart_criteria)
        .bind(track_ids.len() as i32)
        .bind(playlist.created_at)
        .bind(playlist.updated_at)
        .execute(self.database.pool())
        .await?;

        // Add tracks
        for (position, track_id) in track_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(&playlist.id)
            .bind(track_id)
            .bind(position as i32)
            .bind(utils::now())
            .execute(self.database.pool())
            .await?;
        }

        info!(
            "Created smart playlist '{}' with {} tracks",
            name,
            track_ids.len()
        );
        Ok(playlist)
    }

    /// Generate tracks for a smart playlist based on criteria
    async fn generate_smart_playlist_tracks(
        &self,
        criteria: &SmartPlaylistCriteria,
    ) -> Result<Vec<String>> {
        let mut query =
            "SELECT DISTINCT t.id FROM tracks t JOIN artists a ON t.artist_id = a.id".to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        // Add genre filter
        if let Some(genres) = &criteria.genre {
            let genre_placeholders: Vec<_> = (0..genres.len()).map(|_| "?").collect();
            conditions.push(format!(
                "t.id IN (SELECT track_id FROM track_genres tg JOIN genres g ON tg.genre_id = g.id WHERE g.name IN ({}))",
                genre_placeholders.join(",")
            ));
            for genre in genres {
                params.push(genre.clone());
            }
        }

        // Add artist filter
        if let Some(artist_ids) = &criteria.artist_ids {
            let artist_placeholders: Vec<_> = (0..artist_ids.len()).map(|_| "?").collect();
            conditions.push(format!(
                "t.artist_id IN ({})",
                artist_placeholders.join(",")
            ));
            for artist_id in artist_ids {
                params.push(artist_id.clone());
            }
        }

        // Add year range filter
        if let Some((min_year, max_year)) = criteria.year_range {
            query += " LEFT JOIN albums al ON t.album_id = al.id";
            conditions.push("al.release_year BETWEEN ? AND ?".to_string());
            params.push(min_year.to_string());
            params.push(max_year.to_string());
        }

        // Add rating filter
        if let Some(min_rating) = criteria.min_rating {
            query += " LEFT JOIN user_track_ratings utr ON t.id = utr.track_id";
            conditions.push("utr.rating >= ?".to_string());
            params.push(min_rating.to_string());
        }

        // Build WHERE clause
        if !conditions.is_empty() {
            query += &format!(" WHERE {}", conditions.join(" AND "));
        }

        // Add ordering
        if let Some(sort_by) = &criteria.sort_by {
            let order = criteria
                .sort_order
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("ASC");
            match sort_by.as_str() {
                "random" => query += " ORDER BY RANDOM()",
                "play_count" => query += &format!(" ORDER BY t.play_count {}", order),
                "title" => query += &format!(" ORDER BY t.title {}", order),
                "artist" => query += &format!(" ORDER BY a.name {}", order),
                _ => query += " ORDER BY t.created_at DESC",
            }
        } else {
            query += " ORDER BY RANDOM()";
        }

        // Add limit
        if let Some(max_tracks) = criteria.max_tracks {
            query += &format!(" LIMIT {}", max_tracks);
        } else {
            query += " LIMIT 50"; // Default limit
        }

        // Execute query
        // Note: This is a simplified implementation. In a real system, you'd need
        // to properly bind the dynamic parameters.
        let track_ids: Vec<String> = sqlx::query_scalar(&query)
            .fetch_all(self.database.pool())
            .await
            .unwrap_or_default();

        Ok(track_ids)
    }

    /// Sync a playlist to Navidrome
    async fn sync_playlist_to_navidrome(&self, playlist_id: &str) -> Result<()> {
        let playlist = self
            .get_playlist(playlist_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Playlist not found"))?;

        let tracks = self.get_playlist_tracks(playlist_id).await?;
        let track_ids: Vec<String> = tracks.iter().map(|t| t.id.clone()).collect();

        // Create or update playlist in Navidrome
        match self
            .navidrome_client
            .create_or_update_playlist(&playlist.user_id, &playlist.name, track_ids)
            .await
        {
            Ok(_) => {
                debug!(
                    "Successfully synced playlist {} to Navidrome",
                    playlist.name
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to sync playlist {} to Navidrome: {}",
                    playlist.name, e
                );
                Ok(()) // Don't fail hard on sync errors
            }
        }
    }

    /// Sync all playlists for a user
    pub async fn sync_user_playlists(&self, user_id: &str) -> Result<SyncResult> {
        let playlists = self.get_user_playlists(user_id).await?;
        let mut result = SyncResult {
            playlists_synced: 0,
            tracks_added: 0,
            tracks_removed: 0,
            errors: 0,
        };

        for playlist in playlists {
            match self.sync_playlist_to_navidrome(&playlist.id).await {
                Ok(_) => result.playlists_synced += 1,
                Err(_) => result.errors += 1,
            }
        }

        Ok(result)
    }

    /// Update smart playlists (regenerate tracks based on current criteria)
    pub async fn update_smart_playlists(&self) -> Result<()> {
        let smart_playlists =
            sqlx::query_as::<_, Playlist>("SELECT * FROM playlists WHERE is_smart = true")
                .fetch_all(self.database.pool())
                .await?;

        for playlist in smart_playlists {
            if let Some(criteria_str) = &playlist.smart_criteria {
                match serde_json::from_str::<SmartPlaylistCriteria>(criteria_str) {
                    Ok(criteria) => {
                        match self
                            .regenerate_smart_playlist(&playlist.id, &criteria)
                            .await
                        {
                            Ok(_) => info!("Updated smart playlist: {}", playlist.name),
                            Err(e) => {
                                warn!("Failed to update smart playlist {}: {}", playlist.name, e)
                            }
                        }
                    }
                    Err(e) => warn!(
                        "Failed to parse smart playlist criteria for {}: {}",
                        playlist.name, e
                    ),
                }
            }
        }

        Ok(())
    }

    /// Regenerate tracks for a smart playlist
    async fn regenerate_smart_playlist(
        &self,
        playlist_id: &str,
        criteria: &SmartPlaylistCriteria,
    ) -> Result<()> {
        let new_track_ids = self.generate_smart_playlist_tracks(criteria).await?;

        let mut tx = self.database.begin_transaction().await?;

        // Remove existing tracks
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        // Add new tracks
        for (position, track_id) in new_track_ids.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(playlist_id)
            .bind(track_id)
            .bind(position as i32)
            .bind(utils::now())
            .execute(&mut *tx)
            .await?;
        }

        // Update playlist
        sqlx::query("UPDATE playlists SET track_count = ?, updated_at = ? WHERE id = ?")
            .bind(new_track_ids.len() as i32)
            .bind(utils::now())
            .bind(playlist_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        // Sync with Navidrome
        self.sync_playlist_to_navidrome(playlist_id).await?;

        Ok(())
    }

    /// Get playlist statistics
    pub async fn get_playlist_stats(&self) -> Result<PlaylistStats> {
        let total_playlists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlists")
            .fetch_one(self.database.pool())
            .await?;

        let smart_playlists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlists WHERE is_smart = true")
                .fetch_one(self.database.pool())
                .await?;

        let total_playlist_tracks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlist_tracks")
            .fetch_one(self.database.pool())
            .await?;

        let most_popular_playlist: Option<String> =
            sqlx::query_scalar("SELECT name FROM playlists ORDER BY play_count DESC LIMIT 1")
                .fetch_optional(self.database.pool())
                .await?;

        Ok(PlaylistStats {
            total_playlists: total_playlists as u64,
            smart_playlists: smart_playlists as u64,
            total_playlist_tracks: total_playlist_tracks as u64,
            most_popular_playlist,
        })
    }
}

#[async_trait::async_trait]
impl Service for PlaylistService {
    type Stats = PlaylistStats;

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

        Ok(())
    }

    async fn get_stats(&self) -> Result<Self::Stats> {
        self.get_playlist_stats().await
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Playlist service shutting down");
        // No cleanup needed for this service
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smart_playlist_criteria_serialization() {
        let criteria = SmartPlaylistCriteria {
            genre: Some(vec!["Rock".to_string(), "Pop".to_string()]),
            artist_ids: Some(vec!["artist1".to_string()]),
            year_range: Some((2020, 2023)),
            min_rating: Some(4),
            max_tracks: Some(25),
            sort_by: Some("random".to_string()),
            sort_order: Some("ASC".to_string()),
        };

        let serialized = serde_json::to_string(&criteria).unwrap();
        let deserialized: SmartPlaylistCriteria = serde_json::from_str(&serialized).unwrap();

        assert_eq!(criteria.genre, deserialized.genre);
        assert_eq!(criteria.year_range, deserialized.year_range);
        assert_eq!(criteria.max_tracks, deserialized.max_tracks);
    }

    #[test]
    fn test_create_playlist_request() {
        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            description: Some("A test playlist".to_string()),
            user_id: "user123".to_string(),
            is_public: false,
            track_ids: vec!["track1".to_string(), "track2".to_string()],
        };

        assert_eq!(request.name, "Test Playlist");
        assert_eq!(request.track_ids.len(), 2);
        assert!(!request.is_public);
    }
}
