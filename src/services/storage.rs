use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Configuration for tiered storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub hot_downloads_path: PathBuf,
    pub cold_downloads_path: PathBuf,
    pub processing_path: PathBuf,
    pub final_library_path: PathBuf,
    pub enable_tiered: bool,
    pub auto_offload: bool,
    pub offload_delay_seconds: u64,
    pub verify_integrity: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            hot_downloads_path: PathBuf::from("/hot_downloads"),
            cold_downloads_path: PathBuf::from("/cold_downloads"),
            processing_path: PathBuf::from("/processing"),
            final_library_path: PathBuf::from("/final_library"),
            enable_tiered: true,
            auto_offload: true,
            offload_delay_seconds: 300, // 5 minutes
            verify_integrity: false,    // Simplified for now
        }
    }
}

/// Storage operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageOperationResult {
    pub success: bool,
    pub operation: String,
    pub source_path: PathBuf,
    pub destination_path: Option<PathBuf>,
    pub duration_ms: u64,
    pub bytes_transferred: u64,
    pub error: Option<String>,
}

/// Storage management service for handling tiered storage operations
pub struct StorageManager {
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager with the given configuration
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Initialize storage directories
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing storage manager");

        // Create required directories
        let directories = [
            &self.config.hot_downloads_path,
            &self.config.cold_downloads_path,
            &self.config.processing_path,
            &self.config.final_library_path,
        ];

        for dir in directories {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .await
                    .with_context(|| format!("Failed to create directory: {:?}", dir))?;
                info!("Created directory: {:?}", dir);
            }
        }

        Ok(())
    }

    /// Start the background storage monitor
    pub async fn start_monitor(&self) -> Result<()> {
        if !self.config.enable_tiered || !self.config.auto_offload {
            info!("Tiered storage or auto-offload disabled, skipping monitor");
            return Ok(());
        }

        info!("Starting storage monitor for tiered offloading");

        let config = self.config.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::monitor_and_offload(&config).await {
                    error!("Storage monitor error: {}", e);
                }
                sleep(Duration::from_secs(30)).await; // Check every 30 seconds
            }
        });

        Ok(())
    }

    /// Monitor downloads and offload completed files
    async fn monitor_and_offload(config: &StorageConfig) -> Result<()> {
        let completed_files = Self::find_completed_downloads(&config.hot_downloads_path).await?;

        for file_path in completed_files {
            // Check if file is old enough to offload
            if Self::is_ready_for_offload(&file_path, config.offload_delay_seconds).await? {
                info!("Offloading file: {:?}", file_path);

                match Self::offload_file(&file_path, config).await {
                    Ok(result) => {
                        info!(
                            "Successfully offloaded file: {:?} -> {:?}",
                            result.source_path, result.destination_path
                        );
                    }
                    Err(e) => {
                        error!("Failed to offload file {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Find completed download files ready for offloading
    async fn find_completed_downloads(downloads_path: &Path) -> Result<Vec<PathBuf>> {
        let mut completed_files = Vec::new();

        if !downloads_path.exists() {
            return Ok(completed_files);
        }

        let mut entries = fs::read_dir(downloads_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip directories and hidden files
            if !path.is_file() {
                continue;
            }

            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            // Check for audio files
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac") {
                    // Check if file is not being written to (simple check)
                    if !Self::is_being_downloaded(&path).await? {
                        completed_files.push(path);
                    }
                }
            }
        }

        Ok(completed_files)
    }

    /// Check if a file is currently being downloaded (simplified check)
    async fn is_being_downloaded(file_path: &Path) -> Result<bool> {
        let parent = file_path.parent().unwrap_or(file_path);
        let filename = file_path.file_name().unwrap().to_string_lossy();

        // Check for common partial download patterns
        let mut entries = fs::read_dir(parent).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.starts_with(&*filename) && (name.ends_with(".part") || name.ends_with(".tmp")) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a file is ready for offloading based on age
    async fn is_ready_for_offload(file_path: &Path, delay_seconds: u64) -> Result<bool> {
        let metadata = fs::metadata(file_path).await?;
        let modified = metadata.modified()?;
        let age = SystemTime::now().duration_since(modified)?.as_secs();

        Ok(age >= delay_seconds)
    }

    /// Offload a file from hot to cold storage
    async fn offload_file(
        source_path: &Path,
        config: &StorageConfig,
    ) -> Result<StorageOperationResult> {
        let start_time = std::time::Instant::now();

        // Determine destination path in cold storage
        let filename = source_path.file_name().unwrap();
        let destination_path = config.final_library_path.join(filename);

        // Create destination directory
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Get file size
        let file_size = fs::metadata(source_path).await?.len();

        // Copy file to final destination
        fs::copy(source_path, &destination_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to copy to destination: {:?} -> {:?}",
                    source_path, destination_path
                )
            })?;

        // Remove original file from hot storage
        fs::remove_file(source_path)
            .await
            .with_context(|| format!("Failed to remove original file: {:?}", source_path))?;

        let duration = start_time.elapsed();

        Ok(StorageOperationResult {
            success: true,
            operation: "offload".to_string(),
            source_path: source_path.to_path_buf(),
            destination_path: Some(destination_path),
            duration_ms: duration.as_millis() as u64,
            bytes_transferred: file_size,
            error: None,
        })
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<serde_json::Value> {
        let hot_stats = Self::get_directory_stats(&self.config.hot_downloads_path).await?;
        let cold_stats = Self::get_directory_stats(&self.config.cold_downloads_path).await?;
        let processing_stats = Self::get_directory_stats(&self.config.processing_path).await?;
        let library_stats = Self::get_directory_stats(&self.config.final_library_path).await?;

        Ok(serde_json::json!({
            "hot_storage": hot_stats,
            "cold_storage": cold_stats,
            "processing": processing_stats,
            "library": library_stats,
            "config": {
                "tiered_enabled": self.config.enable_tiered,
                "auto_offload": self.config.auto_offload,
                "offload_delay_seconds": self.config.offload_delay_seconds,
                "verify_integrity": self.config.verify_integrity
            }
        }))
    }

    /// Get statistics for a directory
    async fn get_directory_stats(path: &Path) -> Result<serde_json::Value> {
        if !path.exists() {
            return Ok(serde_json::json!({
                "exists": false,
                "total_files": 0,
                "total_size_bytes": 0,
                "audio_files": 0
            }));
        }

        let mut total_files = 0u64;
        let mut total_size = 0u64;
        let mut audio_files = 0u64;

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                total_files += 1;

                if let Ok(metadata) = entry.metadata().await {
                    total_size += metadata.len();
                }

                if let Some(extension) = entry.path().extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac") {
                        audio_files += 1;
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "exists": true,
            "total_files": total_files,
            "total_size_bytes": total_size,
            "audio_files": audio_files,
            "path": path.to_string_lossy()
        }))
    }

    /// Manually trigger offload of a specific file
    pub async fn manual_offload(&self, file_path: &Path) -> Result<StorageOperationResult> {
        if !file_path.starts_with(&self.config.hot_downloads_path) {
            anyhow::bail!("File must be in hot downloads directory");
        }

        if !file_path.exists() {
            anyhow::bail!("File does not exist: {:?}", file_path);
        }

        Self::offload_file(file_path, &self.config).await
    }

    /// Clean up empty directories in processing path
    pub async fn cleanup_processing(&self) -> Result<()> {
        Self::remove_empty_directories(&self.config.processing_path).await
    }

    /// Recursively remove empty directories
    async fn remove_empty_directories(path: &Path) -> Result<()> {
        if !path.is_dir() {
            return Ok(());
        }

        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                Box::pin(Self::remove_empty_directories(&entry_path)).await?;
            }
        }

        // Check again if directory is empty after recursive cleanup
        let mut entries = fs::read_dir(path).await?;
        if entries.next_entry().await?.is_none() {
            fs::remove_dir(path).await?;
            debug!("Removed empty directory: {:?}", path);
        }

        Ok(())
    }
}

/// Create storage manager from environment variables
pub fn create_storage_manager() -> StorageManager {
    let config = StorageConfig {
        hot_downloads_path: PathBuf::from(
            std::env::var("STEPHEYBOT__PATHS__DOWNLOAD_PATH")
                .unwrap_or_else(|_| "/hot_downloads".to_string()),
        ),
        cold_downloads_path: PathBuf::from(
            std::env::var("STEPHEYBOT__PATHS__COLD_DOWNLOAD_PATH")
                .unwrap_or_else(|_| "/cold_downloads".to_string()),
        ),
        processing_path: PathBuf::from(
            std::env::var("STEPHEYBOT__PATHS__PROCESSING_PATH")
                .unwrap_or_else(|_| "/processing".to_string()),
        ),
        final_library_path: PathBuf::from(
            std::env::var("STEPHEYBOT__PATHS__FINAL_LIBRARY_PATH")
                .unwrap_or_else(|_| "/final_library".to_string()),
        ),
        enable_tiered: std::env::var("STEPHEYBOT__STORAGE__ENABLE_TIERED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        auto_offload: std::env::var("STEPHEYBOT__STORAGE__AUTO_OFFLOAD")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        offload_delay_seconds: std::env::var("STEPHEYBOT__STORAGE__OFFLOAD_DELAY")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300),
        verify_integrity: std::env::var("STEPHEYBOT__STORAGE__VERIFY_INTEGRITY")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
    };

    StorageManager::new(config)
}
