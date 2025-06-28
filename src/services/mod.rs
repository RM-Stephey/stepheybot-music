//! Services module for StepheyBot Music
//!
//! This module contains all the business logic services that power the music
//! recommendation and management system.

pub mod download_service;
pub mod library;
pub mod playlist;
pub mod recommendation;
pub mod storage;
pub mod sync;
pub mod user_service;

// Re-export for convenience
pub use download_service::DownloadService;
pub use library::LibraryService;
pub use playlist::PlaylistService;
pub use recommendation::RecommendationService;
pub use storage::StorageManager;
pub use sync::SyncService;
pub use user_service::UserService;

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    clients::{
        lidarr::LidarrClient, listenbrainz::ListenBrainzClient, musicbrainz::MusicBrainzClient,
        navidrome::NavidromeClient,
    },
    database::Database,
};

/// Service manager that coordinates all services
#[derive(Clone)]
pub struct ServiceManager {
    pub download: Arc<DownloadService>,
    pub library: Arc<LibraryService>,
    pub playlist: Arc<PlaylistService>,
    pub recommendation: Arc<RecommendationService>,
    pub storage: Arc<StorageManager>,
    pub sync: Arc<SyncService>,
    pub user: Arc<UserService>,
}

impl ServiceManager {
    /// Create a new service manager with all services initialized
    pub async fn new(
        database: Arc<Database>,
        navidrome_client: Arc<NavidromeClient>,
        listenbrainz_client: Arc<ListenBrainzClient>,
        musicbrainz_client: Arc<MusicBrainzClient>,
        music_path: &str,
        download_path: &str,
        cache_dir: &str,
    ) -> Result<Self> {
        info!("Initializing service manager");

        // Initialize download service
        let download_config = crate::services::download_service::DownloadConfig {
            download_path: std::path::PathBuf::from(download_path),
            final_library_path: std::path::PathBuf::from(music_path),
            ..Default::default()
        };
        let download = Arc::new(DownloadService::new(download_config));

        // Initialize core services
        let library = Arc::new(LibraryService::new(
            database.clone(),
            music_path,
            download_path,
        )?);

        let playlist = Arc::new(PlaylistService::new(
            database.clone(),
            navidrome_client.clone(),
        )?);

        let recommendation = Arc::new(RecommendationService::new(
            database.clone(),
            listenbrainz_client.clone(),
            musicbrainz_client.clone(),
            cache_dir,
        )?);

        let storage = Arc::new(crate::services::storage::create_storage_manager());
        storage.initialize().await?;
        storage.start_monitor().await?;

        let sync = Arc::new(SyncService::new(
            database.clone(),
            navidrome_client.clone(),
            listenbrainz_client.clone(),
        )?);

        let user = Arc::new(UserService::new(database.clone()));

        info!("All services initialized successfully");

        Ok(Self {
            download,
            library,
            playlist,
            recommendation,
            storage,
            sync,
            user,
        })
    }

    /// Perform health checks on all services
    pub async fn health_check(&self) -> Result<ServiceHealthStatus> {
        let mut status = ServiceHealthStatus::default();

        // Check each service
        status.download = true; // Download service health check via qBittorrent
        status.library = self.library.health_check().await.is_ok();
        status.playlist = self.playlist.health_check().await.is_ok();
        status.recommendation = self.recommendation.health_check().await.is_ok();
        status.storage = true; // Storage manager doesn't have health check yet
        status.sync = self.sync.health_check().await.is_ok();
        status.user = true; // User service is always healthy if database is available

        status.overall = status.download
            && status.library
            && status.playlist
            && status.recommendation
            && status.storage
            && status.sync
            && status.user;

        Ok(status)
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> Result<ServiceStats> {
        let download_stats = self.download.get_stats().await;
        let library_stats = self.library.get_stats().await.unwrap_or_default();
        let playlist_stats = self.playlist.get_stats().await.unwrap_or_default();
        let recommendation_stats = self.recommendation.get_stats().await.unwrap_or_default();
        let storage_stats = self.storage.get_storage_stats().await.ok();
        let sync_stats = self.sync.get_stats().await.unwrap_or_default();

        Ok(ServiceStats {
            download: Some(serde_json::to_value(download_stats).unwrap_or_default()),
            library: library_stats,
            playlist: playlist_stats,
            recommendation: recommendation_stats,
            storage: storage_stats,
            sync: sync_stats,
        })
    }

    /// Shutdown all services gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down all services");

        // Shutdown services in reverse dependency order
        if let Err(e) = self.sync.shutdown().await {
            error!("Failed to shutdown sync service: {}", e);
        }

        if let Err(e) = self.storage.cleanup_processing().await {
            error!("Failed to cleanup storage processing: {}", e);
        }

        if let Err(e) = self.recommendation.shutdown().await {
            error!("Failed to shutdown recommendation service: {}", e);
        }

        if let Err(e) = self.playlist.shutdown().await {
            error!("Failed to shutdown playlist service: {}", e);
        }

        if let Err(e) = self.library.shutdown().await {
            error!("Failed to shutdown library service: {}", e);
        }

        info!("All services shutdown complete");
        Ok(())
    }
}

/// Health status for all services
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ServiceHealthStatus {
    pub overall: bool,
    pub download: bool,
    pub library: bool,
    pub playlist: bool,
    pub recommendation: bool,
    pub storage: bool,
    pub sync: bool,
    pub user: bool,
}

/// Statistics for all services
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ServiceStats {
    pub download: Option<serde_json::Value>,
    pub library: LibraryStats,
    pub playlist: PlaylistStats,
    pub recommendation: crate::services::recommendation::RecommendationStats,
    pub storage: Option<serde_json::Value>,
    pub sync: SyncStats,
}

// Service-specific stats structures
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LibraryStats {
    pub total_tracks: u64,
    pub total_albums: u64,
    pub total_artists: u64,
    pub total_size_bytes: u64,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PlaylistStats {
    pub total_playlists: u64,
    pub smart_playlists: u64,
    pub total_playlist_tracks: u64,
    pub most_popular_playlist: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SyncStats {
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub total_synced_plays: u64,
    pub sync_errors: u64,
    pub users_synced: u64,
}

/// Common trait for all services
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    type Stats: Send + Sync + Default;

    /// Perform a health check
    async fn health_check(&self) -> Result<()>;

    /// Get service statistics
    async fn get_stats(&self) -> Result<Self::Stats>;

    /// Shutdown the service gracefully
    async fn shutdown(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_health_status_default() {
        let status = ServiceHealthStatus::default();
        assert!(!status.overall);
        assert!(!status.library);
        assert!(!status.playlist);
    }

    #[test]
    fn test_service_stats_default() {
        let stats = ServiceStats::default();
        assert_eq!(stats.library.total_tracks, 0);
        assert_eq!(stats.playlist.total_playlists, 0);
    }
}
