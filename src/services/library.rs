//! Library service for StepheyBot Music
//!
//! This module handles music library management, including file scanning,
//! organization, metadata extraction, and library statistics.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, info, warn};

use crate::database::Database;
use crate::services::{LibraryStats, Service};
use crate::utils;

/// Library service for managing the music collection
#[derive(Clone)]
pub struct LibraryService {
    database: Arc<Database>,
    music_path: PathBuf,
    download_path: PathBuf,
}

/// Library scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub files_scanned: u64,
    pub files_added: u64,
    pub files_updated: u64,
    pub files_removed: u64,
    pub errors: u64,
    pub duration_seconds: f64,
}

/// Music file information
#[derive(Debug, Clone)]
pub struct MusicFile {
    pub path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
    pub modified_time: std::time::SystemTime,
    pub audio_quality: utils::AudioQuality,
}

impl LibraryService {
    /// Create a new library service
    pub fn new(database: Arc<Database>, music_path: &str, download_path: &str) -> Result<Self> {
        let music_path = PathBuf::from(music_path);
        let download_path = PathBuf::from(download_path);

        if !music_path.exists() {
            warn!("Music path does not exist: {}", music_path.display());
        }

        if !download_path.exists() {
            warn!("Download path does not exist: {}", download_path.display());
        }

        Ok(Self {
            database,
            music_path,
            download_path,
        })
    }

    /// Scan the music library for new or changed files
    pub async fn scan_library(&self) -> Result<ScanResult> {
        let start_time = std::time::Instant::now();
        info!("Starting library scan at: {}", self.music_path.display());

        let mut result = ScanResult {
            files_scanned: 0,
            files_added: 0,
            files_updated: 0,
            files_removed: 0,
            errors: 0,
            duration_seconds: 0.0,
        };

        // Ensure the music directory exists
        if !self.music_path.exists() {
            warn!(
                "Music directory does not exist, creating: {}",
                self.music_path.display()
            );
            fs::create_dir_all(&self.music_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to create music directory: {}",
                        self.music_path.display()
                    )
                })?;
        }

        // Scan for audio files
        match self.scan_directory(&self.music_path, &mut result).await {
            Ok(_) => {
                info!("Library scan completed successfully");
            }
            Err(e) => {
                error!("Library scan failed: {}", e);
                result.errors += 1;
            }
        }

        result.duration_seconds = start_time.elapsed().as_secs_f64();

        info!(
            "Library scan summary: {} files scanned, {} added, {} updated, {} removed, {} errors in {:.2}s",
            result.files_scanned,
            result.files_added,
            result.files_updated,
            result.files_removed,
            result.errors,
            result.duration_seconds
        );

        Ok(result)
    }

    /// Recursively scan a directory for audio files
    async fn scan_directory(&self, dir: &Path, result: &mut ScanResult) -> Result<()> {
        let mut entries = fs::read_dir(dir)
            .await
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                if let Err(e) = Box::pin(self.scan_directory(&path, result)).await {
                    warn!("Failed to scan directory {}: {}", path.display(), e);
                    result.errors += 1;
                }
            } else if utils::is_audio_file(&path) {
                result.files_scanned += 1;

                match self.process_audio_file(&path).await {
                    Ok(processed) => {
                        if processed {
                            result.files_added += 1;
                        } else {
                            result.files_updated += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to process audio file {}: {}", path.display(), e);
                        result.errors += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single audio file
    async fn process_audio_file(&self, path: &Path) -> Result<bool> {
        debug!("Processing audio file: {}", path.display());

        let metadata = fs::metadata(path)
            .await
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        let music_file = MusicFile {
            path: path.to_path_buf(),
            file_name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            file_size: metadata.len(),
            modified_time: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            audio_quality: utils::parse_audio_info_from_filename(
                &path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown"),
            ),
        };

        // For now, just log the file info
        // In a full implementation, this would extract metadata and store in database
        debug!(
            "Found audio file: {} ({}, {})",
            music_file.file_name,
            utils::format_file_size(music_file.file_size),
            music_file.audio_quality.description()
        );

        // Return true if this is a new file (simplified logic)
        Ok(true)
    }

    /// Get library statistics
    pub async fn get_library_stats(&self) -> Result<LibraryStats> {
        // For now, return basic stats
        // In a full implementation, this would query the database
        let stats = LibraryStats {
            total_tracks: self.count_audio_files().await.unwrap_or(0),
            total_albums: 0,  // Would be calculated from database
            total_artists: 0, // Would be calculated from database
            total_size_bytes: self.calculate_total_size().await.unwrap_or(0),
            last_scan: Some(chrono::Utc::now()),
        };

        Ok(stats)
    }

    /// Count audio files in the library
    async fn count_audio_files(&self) -> Result<u64> {
        let mut count = 0;
        let mut stack = vec![self.music_path.clone()];

        while let Some(dir) = stack.pop() {
            if !dir.exists() {
                continue;
            }

            let mut entries = fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if utils::is_audio_file(&path) {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Calculate total size of audio files
    async fn calculate_total_size(&self) -> Result<u64> {
        let mut total_size = 0;
        let mut stack = vec![self.music_path.clone()];

        while let Some(dir) = stack.pop() {
            if !dir.exists() {
                continue;
            }

            let mut entries = fs::read_dir(&dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if utils::is_audio_file(&path) {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// Check if a file exists in the library
    pub async fn file_exists(&self, relative_path: &str) -> bool {
        let full_path = self.music_path.join(relative_path);
        full_path.exists()
    }

    /// Add a file to the library (move from download directory)
    pub async fn add_file_to_library(
        &self,
        source_path: &Path,
        target_relative_path: &str,
    ) -> Result<()> {
        let target_path = self.music_path.join(target_relative_path);

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            utils::ensure_directory_exists(parent).await?;
        }

        // Move the file
        fs::rename(source_path, &target_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to move file from {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;

        info!("Added file to library: {}", target_relative_path);
        Ok(())
    }

    /// Organize a file based on metadata (simplified implementation)
    pub fn generate_organized_path(
        &self,
        artist: &str,
        album: &str,
        track: &str,
        extension: &str,
    ) -> PathBuf {
        let safe_artist = utils::sanitize_filename(artist);
        let safe_album = utils::sanitize_filename(album);
        let safe_track = utils::sanitize_filename(track);

        self.music_path
            .join(&safe_artist)
            .join(&safe_album)
            .join(format!("{}.{}", safe_track, extension))
    }

    /// Clean up empty directories
    pub async fn cleanup_empty_directories(&self) -> Result<u32> {
        let mut removed_count = 0;
        let mut stack = vec![self.music_path.clone()];

        while let Some(dir) = stack.pop() {
            if !dir.exists() || dir == self.music_path {
                continue;
            }

            let mut entries = fs::read_dir(&dir).await?;
            let mut has_files = false;
            let mut subdirs = Vec::new();

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    subdirs.push(path);
                } else {
                    has_files = true;
                    break;
                }
            }

            // Process subdirectories first
            for subdir in subdirs {
                stack.push(subdir);
            }

            // Remove directory if it's empty
            if !has_files {
                if let Ok(mut check_entries) = fs::read_dir(&dir).await {
                    if check_entries.next_entry().await?.is_none() {
                        if let Err(e) = fs::remove_dir(&dir).await {
                            warn!("Failed to remove empty directory {}: {}", dir.display(), e);
                        } else {
                            debug!("Removed empty directory: {}", dir.display());
                            removed_count += 1;
                        }
                    }
                }
            }
        }

        if removed_count > 0 {
            info!("Cleaned up {} empty directories", removed_count);
        }

        Ok(removed_count)
    }

    /// Get the music library root path
    pub fn music_path(&self) -> &Path {
        &self.music_path
    }

    /// Get the download directory path
    pub fn download_path(&self) -> &Path {
        &self.download_path
    }
}

#[async_trait::async_trait]
impl Service for LibraryService {
    type Stats = LibraryStats;

    async fn health_check(&self) -> Result<()> {
        // Check if music directory is accessible
        if !self.music_path.exists() {
            anyhow::bail!(
                "Music directory does not exist: {}",
                self.music_path.display()
            );
        }

        // Check if we can read the directory
        let _entries = fs::read_dir(&self.music_path).await.with_context(|| {
            format!("Cannot read music directory: {}", self.music_path.display())
        })?;

        // Check download directory
        if !self.download_path.exists() {
            warn!(
                "Download directory does not exist: {}",
                self.download_path.display()
            );
        }

        Ok(())
    }

    async fn get_stats(&self) -> Result<Self::Stats> {
        self.get_library_stats().await
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Library service shutting down");
        // No cleanup needed for this service
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_library_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let music_path = temp_dir.path().join("music");
        let download_path = temp_dir.path().join("downloads");

        tokio::fs::create_dir_all(&music_path).await.unwrap();
        tokio::fs::create_dir_all(&download_path).await.unwrap();

        let database = Arc::new(
            crate::database::Database::new("sqlite::memory:")
                .await
                .unwrap(),
        );

        let service = LibraryService::new(
            database,
            music_path.to_str().unwrap(),
            download_path.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(service.music_path(), &music_path);
        assert_eq!(service.download_path(), &download_path);
    }

    #[tokio::test]
    async fn test_generate_organized_path() {
        let temp_dir = TempDir::new().unwrap();
        let music_path = temp_dir.path().join("music");
        let download_path = temp_dir.path().join("downloads");

        let database = Arc::new(
            crate::database::Database::new("sqlite::memory:")
                .await
                .unwrap(),
        );

        let service = LibraryService::new(
            database,
            music_path.to_str().unwrap(),
            download_path.to_str().unwrap(),
        )
        .unwrap();

        let path =
            service.generate_organized_path("The Beatles", "Abbey Road", "Come Together", "mp3");

        let expected = music_path
            .join("The Beatles")
            .join("Abbey Road")
            .join("Come Together.mp3");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_music_file_creation() {
        let temp_path = PathBuf::from("/tmp/test.mp3");
        let file = MusicFile {
            path: temp_path.clone(),
            file_name: "test.mp3".to_string(),
            file_size: 1024,
            modified_time: std::time::SystemTime::UNIX_EPOCH,
            audio_quality: utils::AudioQuality {
                bitrate: Some(320),
                sample_rate: Some(44100),
                channels: Some(2),
                format: Some("mp3".to_string()),
            },
        };

        assert_eq!(file.path, temp_path);
        assert_eq!(file.file_name, "test.mp3");
        assert_eq!(file.file_size, 1024);
    }
}
