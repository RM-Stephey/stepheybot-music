//! StepheyBot Music - Simplified Storage-Focused Version
//!
//! A minimal music service focused on tiered storage management
//! with basic HTTP endpoints for storage operations.

use anyhow::Result;
use axum::{
    extract::{Json as ExtractJson, Path},
    http::{header, StatusCode},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    async fn find_completed_downloads(downloads_path: &std::path::Path) -> Result<Vec<PathBuf>> {
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
    async fn is_being_downloaded(file_path: &std::path::Path) -> Result<bool> {
        let parent = file_path.parent().unwrap_or(file_path);
        let filename = file_path.file_name().unwrap().to_string_lossy();

        // Check for common partial download patterns
        let mut entries = fs::read_dir(parent).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_name = entry.file_name();
            let name = entry_name.to_string_lossy();
            if name.starts_with(&*filename) && (name.ends_with(".part") || name.ends_with(".tmp")) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a file is ready for offloading based on age
    async fn is_ready_for_offload(file_path: &std::path::Path, delay_seconds: u64) -> Result<bool> {
        let metadata = fs::metadata(file_path).await?;
        let modified = metadata.modified()?;
        let age = SystemTime::now().duration_since(modified)?.as_secs();

        Ok(age >= delay_seconds)
    }

    /// Offload a file from hot to cold storage
    async fn offload_file(
        source_path: &std::path::Path,
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
    async fn get_directory_stats(path: &std::path::Path) -> Result<serde_json::Value> {
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
    pub async fn manual_offload(
        &self,
        file_path: &std::path::Path,
    ) -> Result<StorageOperationResult> {
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
    fn remove_empty_directories(
        path: &std::path::Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            if !path.is_dir() {
                return Ok(());
            }

            let mut entries = fs::read_dir(path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    Self::remove_empty_directories(&entry_path).await?;
                }
            }

            // Check again if directory is empty after recursive cleanup
            let mut entries = fs::read_dir(path).await?;
            if entries.next_entry().await?.is_none() {
                fs::remove_dir(path).await?;
                debug!("Removed empty directory: {:?}", path);
            }

            Ok(())
        })
    }
}

use anyhow::Context;

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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,stepheybot_music=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create global storage manager
    let storage_manager = create_storage_manager();
    storage_manager.initialize().await?;
    storage_manager.start_monitor().await?;

    info!("🎵 StepheyBot Music Storage Service starting up");

    // Build the router
    let app = Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        // API endpoints
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/health", get(health_check))
        // Storage management endpoints
        .route("/api/v1/storage/stats", get(storage_stats))
        .route("/api/v1/storage/offload", post(manual_offload))
        .route("/api/v1/storage/cleanup", post(cleanup_storage))
        // Basic music endpoints (stubs for now)
        .route("/api/v1/tracks/search/:query", get(search_tracks))
        .route("/api/v1/discover", get(discover_music))
        // Static file serving for frontend
        .nest_service("/_app", ServeDir::new("/app/frontend/_app"))
        .route("/favicon.svg", get(serve_favicon))
        // Root route - serve the frontend
        .route("/", get(serve_frontend))
        // Smart fallback - API routes get 404 JSON, others get frontend for SPA routing
        .fallback(smart_fallback)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        );

    // Get port from environment or use default
    let port = std::env::var("STEPHEYBOT__SERVER__PORT")
        .unwrap_or_else(|_| "8083".to_string())
        .parse::<u16>()
        .unwrap_or(8083);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Server running on http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 StepheyBot Music shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Received shutdown signal");
}

async fn serve_frontend() -> Result<Html<String>, StatusCode> {
    // Simple HTML frontend
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>StepheyBot Music Storage</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #0a0a0a; color: #e0e0e0; }
        .container { max-width: 1200px; margin: 0 auto; }
        .header { text-align: center; margin-bottom: 40px; }
        .header h1 { color: #ff6b9d; margin: 0; }
        .header p { color: #8a8a8a; margin: 10px 0 0 0; }
        .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; }
        .stat-card { background: #1a1a1a; border: 1px solid #333; border-radius: 8px; padding: 20px; }
        .stat-card h3 { color: #4a9eff; margin: 0 0 15px 0; }
        .stat-item { display: flex; justify-content: space-between; margin-bottom: 10px; }
        .stat-item:last-child { margin-bottom: 0; }
        .actions { margin-top: 40px; text-align: center; }
        .btn { background: #4a9eff; color: white; border: none; padding: 10px 20px; border-radius: 5px; cursor: pointer; margin: 0 10px; }
        .btn:hover { background: #3a8eef; }
        .btn.danger { background: #ff6b6b; }
        .btn.danger:hover { background: #ff5b5b; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎵 StepheyBot Music Storage</h1>
            <p>Tiered Storage Management System</p>
        </div>

        <div class="stats" id="stats">
            <div class="stat-card">
                <h3>Loading...</h3>
                <p>Fetching storage statistics...</p>
            </div>
        </div>

        <div class="actions">
            <button class="btn" onclick="refreshStats()">Refresh Stats</button>
            <button class="btn" onclick="triggerCleanup()">Cleanup Processing</button>
            <button class="btn danger" onclick="showManualOffload()">Manual Offload</button>
        </div>
    </div>

    <script>
        async function fetchStats() {
            try {
                const response = await fetch('/api/v1/storage/stats');
                const data = await response.json();
                displayStats(data);
            } catch (error) {
                console.error('Failed to fetch stats:', error);
                document.getElementById('stats').innerHTML = '<div class="stat-card"><h3>Error</h3><p>Failed to load statistics</p></div>';
            }
        }

        function displayStats(data) {
            const statsContainer = document.getElementById('stats');
            const storage = data.storage;

            statsContainer.innerHTML = `
                <div class="stat-card">
                    <h3>Hot Storage (NVME)</h3>
                    <div class="stat-item"><span>Path:</span><span>${storage.hot_storage.path || 'N/A'}</span></div>
                    <div class="stat-item"><span>Total Files:</span><span>${storage.hot_storage.total_files || 0}</span></div>
                    <div class="stat-item"><span>Audio Files:</span><span>${storage.hot_storage.audio_files || 0}</span></div>
                    <div class="stat-item"><span>Size:</span><span>${formatBytes(storage.hot_storage.total_size_bytes || 0)}</span></div>
                </div>
                <div class="stat-card">
                    <h3>Cold Storage (HDD)</h3>
                    <div class="stat-item"><span>Path:</span><span>${storage.cold_storage.path || 'N/A'}</span></div>
                    <div class="stat-item"><span>Total Files:</span><span>${storage.cold_storage.total_files || 0}</span></div>
                    <div class="stat-item"><span>Audio Files:</span><span>${storage.cold_storage.audio_files || 0}</span></div>
                    <div class="stat-item"><span>Size:</span><span>${formatBytes(storage.cold_storage.total_size_bytes || 0)}</span></div>
                </div>
                <div class="stat-card">
                    <h3>Library</h3>
                    <div class="stat-item"><span>Path:</span><span>${storage.library.path || 'N/A'}</span></div>
                    <div class="stat-item"><span>Total Files:</span><span>${storage.library.total_files || 0}</span></div>
                    <div class="stat-item"><span>Audio Files:</span><span>${storage.library.audio_files || 0}</span></div>
                    <div class="stat-item"><span>Size:</span><span>${formatBytes(storage.library.total_size_bytes || 0)}</span></div>
                </div>
                <div class="stat-card">
                    <h3>Configuration</h3>
                    <div class="stat-item"><span>Tiered Storage:</span><span>${storage.config.tiered_enabled ? 'Enabled' : 'Disabled'}</span></div>
                    <div class="stat-item"><span>Auto Offload:</span><span>${storage.config.auto_offload ? 'Enabled' : 'Disabled'}</span></div>
                    <div class="stat-item"><span>Offload Delay:</span><span>${storage.config.offload_delay_seconds}s</span></div>
                    <div class="stat-item"><span>Verify Integrity:</span><span>${storage.config.verify_integrity ? 'Enabled' : 'Disabled'}</span></div>
                </div>
            `;
        }

        function formatBytes(bytes) {
            if (bytes === 0) return '0 Bytes';
            const k = 1024;
            const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }

        function refreshStats() {
            fetchStats();
        }

        async function triggerCleanup() {
            try {
                const response = await fetch('/api/v1/storage/cleanup', { method: 'POST' });
                const data = await response.json();
                alert(data.success ? 'Cleanup completed successfully' : 'Cleanup failed: ' + data.error);
                fetchStats();
            } catch (error) {
                alert('Failed to trigger cleanup: ' + error.message);
            }
        }

        function showManualOffload() {
            const filePath = prompt('Enter the full path of the file to offload:');
            if (filePath) {
                manualOffload(filePath);
            }
        }

        async function manualOffload(filePath) {
            try {
                const response = await fetch('/api/v1/storage/offload', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ file_path: filePath })
                });
                const data = await response.json();
                alert(data.success ? 'File offloaded successfully' : 'Offload failed: ' + (data.error || 'Unknown error'));
                fetchStats();
            } catch (error) {
                alert('Failed to offload file: ' + error.message);
            }
        }

        // Load stats on page load
        fetchStats();

        // Refresh stats every 30 seconds
        setInterval(fetchStats, 30000);
    </script>
</body>
</html>
    "#;

    Ok(Html(html.to_string()))
}

async fn serve_favicon() -> Result<Response, StatusCode> {
    // Simple SVG favicon
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"#ff6b9d\"><path d=\"M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z\"/></svg>";

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .body(svg.into())
        .unwrap())
}

async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "stepheybot-music-storage",
        "timestamp": Utc::now(),
        "version": "0.1.0"
    })))
}

async fn readiness_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ready",
        "service": "stepheybot-music-storage",
        "timestamp": Utc::now()
    })))
}

async fn liveness_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "alive",
        "service": "stepheybot-music-storage",
        "timestamp": Utc::now()
    })))
}

async fn api_status() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "service": "stepheybot-music-storage",
        "version": "0.1.0",
        "features": {
            "tiered_storage": true,
            "auto_offload": true,
            "manual_offload": true,
            "statistics": true
        },
        "endpoints": {
            "storage_stats": "/api/v1/storage/stats",
            "manual_offload": "/api/v1/storage/offload",
            "cleanup": "/api/v1/storage/cleanup"
        },
        "timestamp": Utc::now()
    })))
}

/// Get storage statistics
async fn storage_stats() -> Result<Json<Value>, StatusCode> {
    info!("Storage stats request");

    let storage_manager = create_storage_manager();

    match storage_manager.get_storage_stats().await {
        Ok(stats) => Ok(Json(json!({
            "success": true,
            "storage": stats,
            "timestamp": Utc::now()
        }))),
        Err(e) => {
            error!("Failed to get storage stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Manually trigger file offload
async fn manual_offload(
    ExtractJson(payload): ExtractJson<Value>,
) -> Result<Json<Value>, StatusCode> {
    info!("Manual offload request: {:?}", payload);

    let file_path = payload
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let storage_manager = create_storage_manager();

    match storage_manager
        .manual_offload(std::path::Path::new(file_path))
        .await
    {
        Ok(result) => Ok(Json(json!({
            "success": true,
            "operation": result,
            "timestamp": Utc::now()
        }))),
        Err(e) => {
            error!("Failed to offload file {}: {}", file_path, e);
            Ok(Json(json!({
                "success": false,
                "error": e.to_string(),
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Cleanup storage processing directory
async fn cleanup_storage() -> Result<Json<Value>, StatusCode> {
    info!("Storage cleanup request");

    let storage_manager = create_storage_manager();

    match storage_manager.cleanup_processing().await {
        Ok(()) => Ok(Json(json!({
            "success": true,
            "message": "Processing directory cleaned up successfully",
            "timestamp": Utc::now()
        }))),
        Err(e) => {
            error!("Failed to cleanup storage: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": e.to_string(),
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Search tracks (stub implementation)
async fn search_tracks(Path(query): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Search tracks request for: {}", query);

    Ok(Json(json!({
        "success": true,
        "query": query,
        "tracks": [],
        "total": 0,
        "message": "Track search not yet implemented - use full music service",
        "timestamp": Utc::now()
    })))
}

/// Discover music (stub implementation)
async fn discover_music() -> Result<Json<Value>, StatusCode> {
    info!("Discover music request");

    Ok(Json(json!({
        "success": true,
        "tracks": [],
        "total": 0,
        "message": "Music discovery not yet implemented - use full music service",
        "timestamp": Utc::now()
    })))
}

/// Smart fallback handler - returns 404 JSON for API routes, frontend for others
async fn smart_fallback(uri: axum::http::Uri) -> Result<Response, StatusCode> {
    let path = uri.path();

    if path.starts_with("/api/") {
        // Return JSON 404 for API routes
        let json_response = Json(json!({
            "error": "Not Found",
            "message": format!("API endpoint {} not found", path),
            "timestamp": Utc::now()
        }));

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&json_response.0).unwrap().into())
            .unwrap())
    } else {
        // Serve frontend for non-API routes (SPA routing)
        match serve_frontend().await {
            Ok(html) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(html.0.into())
                .unwrap()),
            Err(status) => Err(status),
        }
    }
}
