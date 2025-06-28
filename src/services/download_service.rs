//! Download Service for StepheyBot Music
//!
//! This service manages the download queue, torrent processing, and file organization
//! for automated music acquisition through Transmission.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

use crate::clients::transmission::{TorrentInfo, TransmissionClient};
use crate::models::entities::{DownloadFile, DownloadRequest, TorrentDownload};

/// Download service configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub transmission_url: String,
    pub transmission_username: String,
    pub transmission_password: String,
    pub download_path: PathBuf,
    pub processing_path: PathBuf,
    pub final_library_path: PathBuf,
    pub category: String,
    pub max_concurrent_downloads: usize,
    pub monitor_interval: Duration,
    pub cleanup_interval: Duration,
    pub seed_ratio_limit: f64,
    pub seed_time_limit: Duration,
    pub auto_delete_completed: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            transmission_url: "http://localhost:9091".to_string(),
            transmission_username: "admin".to_string(),
            transmission_password: "adminadmin".to_string(),
            download_path: PathBuf::from("/downloads"),
            processing_path: PathBuf::from("/processing"),
            final_library_path: PathBuf::from("/music"),
            category: "stepheybot-music".to_string(),
            max_concurrent_downloads: 5,
            monitor_interval: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(300),
            seed_ratio_limit: 2.0,
            seed_time_limit: Duration::from_secs(86400), // 24 hours
            auto_delete_completed: true,
        }
    }
}

/// Download service for managing music downloads
pub struct DownloadService {
    config: DownloadConfig,
    transmission: Arc<Mutex<TransmissionClient>>,
    active_downloads: Arc<RwLock<HashMap<String, TorrentDownload>>>,
    download_queue: Arc<Mutex<Vec<DownloadRequest>>>,
    processing_queue: Arc<Mutex<Vec<String>>>, // Torrent hashes ready for processing
    stats: Arc<RwLock<DownloadStats>>,
}

/// Download statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DownloadStats {
    pub total_downloads: u64,
    pub completed_downloads: u64,
    pub failed_downloads: u64,
    pub active_downloads: u64,
    pub queued_downloads: u64,
    pub total_downloaded_bytes: u64,
    pub total_uploaded_bytes: u64,
    pub average_download_speed: u64,
    pub last_updated: DateTime<Utc>,
}

impl DownloadService {
    /// Create a new download service
    pub fn new(config: DownloadConfig) -> Self {
        let transmission = Arc::new(Mutex::new(TransmissionClient::new(
            config.transmission_url.clone(),
            config.transmission_username.clone(),
            config.transmission_password.clone(),
        )));

        Self {
            config,
            transmission,
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            download_queue: Arc::new(Mutex::new(Vec::new())),
            processing_queue: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(RwLock::new(DownloadStats::default())),
        }
    }

    /// Start the download service background tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting DownloadService with config: {:?}", self.config);

        // Test Transmission connection
        self.transmission
            .lock()
            .await
            .health_check()
            .await
            .context("Failed to connect to Transmission")?;

        info!("Transmission connection established");

        // Start background tasks
        self.start_background_tasks().await;

        Ok(())
    }

    /// Add a download request to the queue
    pub async fn add_download(&self, mut request: DownloadRequest) -> Result<String> {
        let request_id = request.id.clone();

        // Validate magnet URL
        if let Some(magnet_url) = &request.magnet_url {
            if !magnet_url.starts_with("magnet:") {
                request.status = "failed".to_string();
                request.error_message = Some("Invalid magnet URL".to_string());
                return Err(anyhow::anyhow!("Invalid magnet URL"));
            }
        } else {
            request.status = "failed".to_string();
            request.error_message = Some("No magnet URL provided".to_string());
            return Err(anyhow::anyhow!("No magnet URL provided"));
        }

        // Add to queue
        {
            let mut queue = self.download_queue.lock().await;
            request.status = "queued".to_string();
            queue.push(request);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.queued_downloads += 1;
            stats.total_downloads += 1;
            stats.last_updated = Utc::now();
        }

        info!("Added download request to queue: {}", request_id);
        Ok(request_id)
    }

    /// Get download status by request ID
    pub async fn get_download_status(&self, request_id: &str) -> Option<DownloadRequest> {
        // Check queue first
        {
            let queue = self.download_queue.lock().await;
            if let Some(request) = queue.iter().find(|r| r.id == request_id) {
                return Some(request.clone());
            }
        }

        // Check active downloads
        {
            let active = self.active_downloads.read().await;
            for download in active.values() {
                if download.download_request_id == request_id {
                    // Convert TorrentDownload back to DownloadRequest format
                    let mut request = DownloadRequest::new(
                        "system".to_string(),
                        "Unknown".to_string(),
                        download.name.clone(),
                    );
                    request.id = request_id.to_string();
                    request.status = download.status.clone();
                    request.progress = Some(download.progress);
                    request.file_size = Some(download.size);
                    request.download_speed = Some(download.download_speed);
                    request.seeds = Some(download.seeds);
                    request.peers = Some(download.peers);
                    request.torrent_hash = Some(download.torrent_hash.clone());
                    request.started_at = Some(download.added_at);
                    request.completed_at = download.completed_at;

                    return Some(request);
                }
            }
        }

        None
    }

    /// Get current download statistics
    pub async fn get_stats(&self) -> DownloadStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get list of active downloads
    pub async fn get_active_downloads(&self) -> Vec<TorrentDownload> {
        let active = self.active_downloads.read().await;
        active.values().cloned().collect()
    }

    /// Pause a download
    pub async fn pause_download(&self, torrent_hash: &str) -> Result<()> {
        self.transmission
            .lock()
            .await
            .pause_torrent(torrent_hash)
            .await?;
        info!("Paused download: {}", torrent_hash);
        Ok(())
    }

    /// Resume a download
    pub async fn resume_download(&self, torrent_hash: &str) -> Result<()> {
        self.transmission
            .lock()
            .await
            .resume_torrent(torrent_hash)
            .await?;
        info!("Resumed download: {}", torrent_hash);
        Ok(())
    }

    /// Cancel a download
    pub async fn cancel_download(&self, torrent_hash: &str, delete_files: bool) -> Result<()> {
        // Remove from active downloads
        {
            let mut active = self.active_downloads.write().await;
            active.remove(torrent_hash);
        }

        // Delete from Transmission
        self.transmission
            .lock()
            .await
            .remove_torrent(torrent_hash, delete_files)
            .await?;

        info!(
            "Cancelled download: {} (delete_files: {})",
            torrent_hash, delete_files
        );
        Ok(())
    }

    /// Start background monitoring tasks
    async fn start_background_tasks(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            service.download_processor_task().await;
        });

        let service = self.clone();
        tokio::spawn(async move {
            service.download_monitor_task().await;
        });

        let service = self.clone();
        tokio::spawn(async move {
            service.cleanup_task().await;
        });

        let service = self.clone();
        tokio::spawn(async move {
            service.file_processor_task().await;
        });
    }

    /// Task to process download queue
    async fn download_processor_task(&self) {
        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Check if we have room for more downloads
            let active_count = {
                let active = self.active_downloads.read().await;
                active.len()
            };

            if active_count >= self.config.max_concurrent_downloads {
                continue;
            }

            // Get next download from queue
            let next_download = {
                let mut queue = self.download_queue.lock().await;
                if queue.is_empty() {
                    continue;
                }
                queue.remove(0)
            };

            // Process the download
            if let Err(e) = self.process_download_request(next_download).await {
                error!("Failed to process download request: {}", e);
            }
        }
    }

    /// Task to monitor active downloads
    async fn download_monitor_task(&self) {
        let mut interval = interval(self.config.monitor_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.update_download_status().await {
                error!("Failed to update download status: {}", e);
            }
        }
    }

    /// Task to cleanup completed downloads
    async fn cleanup_task(&self) {
        let mut interval = interval(self.config.cleanup_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.cleanup_completed_downloads().await {
                error!("Failed to cleanup completed downloads: {}", e);
            }
        }
    }

    /// Task to process completed files
    async fn file_processor_task(&self) {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Err(e) = self.process_completed_files().await {
                error!("Failed to process completed files: {}", e);
            }
        }
    }

    /// Process a single download request
    async fn process_download_request(&self, mut request: DownloadRequest) -> Result<()> {
        info!("Processing download request: {}", request.id);

        let magnet_url = request
            .magnet_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No magnet URL"))?;

        // Add to Transmission
        match self
            .transmission
            .lock()
            .await
            .add_magnet(
                magnet_url,
                Some(&self.config.download_path.to_string_lossy()),
                Some(false), // Don't start paused
            )
            .await
        {
            Ok(torrent_hash) => {
                request.torrent_hash = Some(torrent_hash.clone());
                request.status = "downloading".to_string();
                request.started_at = Some(Utc::now());

                // Create torrent download record
                let torrent_download = TorrentDownload::new(
                    request.id.clone(),
                    torrent_hash.clone(),
                    request.full_description(),
                    self.config.download_path.to_string_lossy().to_string(),
                );

                // Add to active downloads
                {
                    let mut active = self.active_downloads.write().await;
                    active.insert(torrent_hash.clone(), torrent_download);
                }

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.queued_downloads = stats.queued_downloads.saturating_sub(1);
                    stats.active_downloads += 1;
                    stats.last_updated = Utc::now();
                }

                info!(
                    "Successfully started download: {} -> {}",
                    request.id, torrent_hash
                );
            }
            Err(e) => {
                error!("Failed to get torrents from Transmission: {}", e);
                request.status = "failed".to_string();
                request.error_message = Some(e.to_string());

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.queued_downloads = stats.queued_downloads.saturating_sub(1);
                    stats.failed_downloads += 1;
                    stats.last_updated = Utc::now();
                }
            }
        }

        Ok(())
    }

    /// Update status of all active downloads
    async fn update_download_status(&self) -> Result<()> {
        let torrents = match self.transmission.lock().await.get_torrents().await {
            Ok(torrents) => torrents,
            Err(e) => {
                error!("Failed to get torrents from Transmission: {}", e);
                return Err(e);
            }
        };
        let mut completed_hashes = Vec::new();

        {
            let mut active = self.active_downloads.write().await;
            let mut stats = self.stats.write().await;

            for torrent in torrents {
                if let Some(download) = active.get_mut(&torrent.hash) {
                    download.update_from_torrent_info(&torrent);

                    if torrent.is_completed() && !completed_hashes.contains(&torrent.hash) {
                        completed_hashes.push(torrent.hash.clone());
                    }
                }
            }

            // Update global stats
            stats.active_downloads = active.len() as u64;
            stats.last_updated = Utc::now();
        }

        // Add completed torrents to processing queue
        if !completed_hashes.is_empty() {
            let mut processing = self.processing_queue.lock().await;
            for hash in completed_hashes {
                if !processing.contains(&hash) {
                    processing.push(hash.clone());
                    info!("Added completed torrent to processing queue: {}", hash);
                }
            }
        }

        Ok(())
    }

    /// Process completed files
    async fn process_completed_files(&self) -> Result<()> {
        let mut processing = self.processing_queue.lock().await;
        if processing.is_empty() {
            return Ok(());
        }

        let hash = processing.remove(0);
        drop(processing);

        info!("Processing completed torrent: {}", hash);

        // Get torrent info
        let torrent = match self
            .transmission
            .lock()
            .await
            .get_torrent_by_hash(&hash)
            .await?
        {
            Some(t) => t,
            None => {
                warn!("Torrent not found in Transmission: {}", hash);
                return Ok(());
            }
        };

        // Get files in torrent
        // Note: Transmission doesn't have per-file info like qBittorrent
        // Process the entire torrent directory
        info!("Processing completed torrent directory for: {}", hash);

        // Process the entire torrent
        match self.process_torrent_directory(&torrent).await {
            Ok(_) => {
                info!("Successfully processed torrent: {}", torrent.name);
            }
            Err(e) => {
                warn!("Failed to process torrent {}: {}", torrent.name, e);
            }
        }

        info!("Completed processing torrent: {}", hash);

        // Update download status
        {
            let mut active = self.active_downloads.write().await;
            if let Some(download) = active.get_mut(&hash) {
                download.status = "completed".to_string();
                download.completed_at = Some(Utc::now());
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.active_downloads = stats.active_downloads.saturating_sub(1);
            stats.completed_downloads += 1;
            stats.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Copy directory recursively
    async fn copy_directory(&self, source: &Path, destination: &Path) -> Result<()> {
        tokio::fs::create_dir_all(destination).await?;
        let mut entries = tokio::fs::read_dir(source).await?;

        while let Some(entry) = entries.next_entry().await? {
            let source_path = entry.path();
            let dest_path = destination.join(entry.file_name());

            if source_path.is_dir() {
                Box::pin(self.copy_directory(&source_path, &dest_path)).await?;
            } else {
                tokio::fs::copy(&source_path, &dest_path).await?;
            }
        }
        Ok(())
    }

    /// Process completed torrent directory
    async fn process_torrent_directory(&self, torrent: &TorrentInfo) -> Result<()> {
        let source_path = Path::new(&torrent.download_dir).join(&torrent.name);

        // Create processing directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.processing_path).await?;

        // Copy entire torrent directory to processing
        let processing_dir = self.config.processing_path.join(&torrent.hash);
        self.copy_directory(&source_path, &processing_dir).await?;

        // Organize the entire directory
        let final_path = self
            .organize_music_directory(&processing_dir, torrent)
            .await?;

        info!(
            "Successfully processed torrent: {} -> {}",
            torrent.name,
            final_path.display()
        );
        Ok(())
    }

    /// Organize music directory into proper library structure
    async fn organize_music_directory(
        &self,
        processing_path: &Path,
        torrent: &TorrentInfo,
    ) -> Result<PathBuf> {
        // Try to extract metadata from torrent name
        let (artist, album, _) = self.extract_metadata_from_path(&torrent.name, &torrent.name);

        // Create library directory structure
        let artist_dir = sanitize_filename(&artist);
        let album_dir = sanitize_filename(&album);

        let library_path = self
            .config
            .final_library_path
            .join(&artist_dir)
            .join(&album_dir);

        tokio::fs::create_dir_all(&library_path).await?;

        // Generate final filename
        let extension = processing_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("mp3");

        let final_filename = format!("{}.{}", sanitize_filename(&torrent.name), extension);
        let final_path = library_path.join(&final_filename);

        // Move file to final location
        tokio::fs::rename(processing_path, &final_path)
            .await
            .context("Failed to move file to library")?;

        Ok(final_path)
    }

    /// Extract metadata from file path and torrent name
    fn extract_metadata_from_path(
        &self,
        file_path: &str,
        torrent_name: &str,
    ) -> (String, String, String) {
        // Simple extraction - in a real implementation, you'd want to use a proper metadata library
        let path = Path::new(file_path);
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");

        // Try to parse from filename patterns like "Artist - Album - Track"
        let parts: Vec<&str> = filename.split(" - ").collect();

        let (artist, album, track) = match parts.len() {
            3 => (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ),
            2 => (
                parts[0].to_string(),
                "Unknown Album".to_string(),
                parts[1].to_string(),
            ),
            _ => (
                "Unknown Artist".to_string(),
                "Unknown Album".to_string(),
                filename.to_string(),
            ),
        };

        (artist, album, track)
    }

    /// Cleanup completed downloads based on seeding rules
    async fn cleanup_completed_downloads(&self) -> Result<()> {
        if !self.config.auto_delete_completed {
            return Ok(());
        }

        let torrents = self.transmission.lock().await.get_torrents().await?;
        let completed_torrents: Vec<_> =
            torrents.into_iter().filter(|t| t.is_completed()).collect();
        let mut cleanup_count = 0;

        for torrent in completed_torrents {
            let should_delete = torrent.ratio >= self.config.seed_ratio_limit;
            // Note: Transmission doesn't provide seeding_time in basic info

            if should_delete {
                if let Err(e) = self
                    .transmission
                    .lock()
                    .await
                    .remove_torrent(&torrent.hash, false)
                    .await
                {
                    warn!("Failed to delete completed torrent {}: {}", torrent.hash, e);
                } else {
                    cleanup_count += 1;
                    info!(
                        "Cleaned up completed torrent: {} (ratio: {:.2})",
                        torrent.hash, torrent.ratio
                    );
                }
            }
        }

        if cleanup_count > 0 {
            info!("Cleaned up {} completed torrents", cleanup_count);
        }

        Ok(())
    }
}

/// Helper function to check if a file is a music file
fn is_music_file(filename: &str) -> bool {
    const MUSIC_EXTENSIONS: &[&str] = &[
        "mp3", "flac", "wav", "ogg", "m4a", "aac", "wma", "ape", "opus", "aiff", "au",
    ];

    if let Some(ext) = Path::new(filename).extension().and_then(|ext| ext.to_str()) {
        MUSIC_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

/// Sanitize filename for filesystem use
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

impl Clone for DownloadService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transmission: Arc::clone(&self.transmission),
            active_downloads: Arc::clone(&self.active_downloads),
            download_queue: Arc::clone(&self.download_queue),
            processing_queue: Arc::clone(&self.processing_queue),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test/File:Name"), "Test_File_Name");
        assert_eq!(sanitize_filename("Normal Name"), "Normal Name");
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_is_music_file() {
        assert!(is_music_file("song.mp3"));
        assert!(is_music_file("track.flac"));
        assert!(!is_music_file("image.jpg"));
        assert!(!is_music_file("document.txt"));
    }

    #[tokio::test]
    async fn test_download_service_creation() {
        let config = DownloadConfig::default();
        let service = DownloadService::new(config);

        let stats = service.get_stats().await;
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.active_downloads, 0);
    }
}
