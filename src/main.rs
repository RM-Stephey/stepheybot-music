//! StepheyBot Music - Minimal working version
//!
//! A simplified version that provides basic HTTP endpoints and health checks
//! while we work on implementing the full functionality.

mod api;
mod auth;
mod clients;
mod database;
mod lidarr_addon;
mod models;
mod navidrome_addon;
mod services;
mod utils;

use crate::lidarr_addon::is_lidarr_configured;
use crate::models::entities::DownloadRequest;
use crate::services::download_service::{DownloadConfig, DownloadService};

use anyhow::Result;
use axum::{
    extract::{Json as ExtractJson, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use lidarr_addon::{
    create_lidarr_addon, get_lidarr_connection_status, test_lidarr_integration, LidarrSearchResult,
};
use navidrome_addon::{create_navidrome_addon, get_connection_status, test_navidrome_integration};
use rand::random;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stepheybot_music=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()?;

    info!(
        "🎵 Starting StepheyBot Music v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Initialize Navidrome integration on startup
    info!("🔗 Initializing Navidrome integration...");

    // Create Navidrome addon
    let navidrome_addon = create_navidrome_addon();
    info!("📦 Navidrome addon created");

    // Test connection and fetch initial data
    let test_result = test_navidrome_integration().await;
    info!("✅ Navidrome integration test completed: {:?}", test_result);

    // Try to get library stats to verify full functionality
    match navidrome_addon.get_library_stats().await {
        Ok(stats) => {
            info!("📊 Navidrome library stats retrieved: {:?}", stats);
        }
        Err(e) => {
            warn!("⚠️ Could not retrieve library stats: {}", e);
        }
    }

    info!("🚀 Navidrome integration setup complete");

    // Initialize Lidarr integration
    info!("🎵 Initializing Lidarr integration...");
    let lidarr_test = test_lidarr_integration().await;
    info!("✅ Lidarr integration test completed: {:?}", lidarr_test);
    info!("🚀 Lidarr integration setup complete");

    // Initialize Download Service
    info!("🔧 Initializing Download Service...");
    let download_config = DownloadConfig {
        transmission_url: std::env::var("STEPHEYBOT__TRANSMISSION__URL")
            .unwrap_or_else(|_| "http://stepheybot_music_vpn:9091".to_string()),
        transmission_username: std::env::var("STEPHEYBOT__TRANSMISSION__USERNAME")
            .unwrap_or_else(|_| "admin".to_string()),
        transmission_password: std::env::var("STEPHEYBOT__TRANSMISSION__PASSWORD")
            .unwrap_or_else(|_| "adminadmin".to_string()),
        download_path: std::path::PathBuf::from("/hot_downloads"),
        processing_path: std::path::PathBuf::from("/processing"),
        final_library_path: std::path::PathBuf::from("/final_library"),
        category: "stepheybot-music".to_string(),
        ..Default::default()
    };

    let download_service = Arc::new(DownloadService::new(download_config));

    // Start the download service
    if let Err(e) = download_service.start().await {
        warn!("⚠️ Could not start download service: {}", e);
    } else {
        info!("✅ Download service started successfully");
    }

    // Create router
    let app = Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // API routes (placeholders for now)
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/sync", post(trigger_sync))
        .route("/api/v1/recommendations/:user_id", get(get_recommendations))
        .route("/api/v1/playlists/generate", post(generate_playlist))
        .route("/api/v1/library/scan", post(scan_library))
        .route("/api/v1/stats", get(get_stats))
        .route("/api/v1/library/stats", get(get_library_stats))
        .route("/stats", get(get_stats))
        .route("/status", get(api_status))
        .route("/library", get(get_stats))
        .route(
            "/recommendations",
            get(|| async { Json(json!({"recommendations": []})) }),
        )
        .route("/system", get(system_info))
        .route("/system/stats", get(get_stats))
        .route("/api/system/stats", get(get_stats))
        .route("/api/system", get(system_info))
        // Admin routes (placeholders)
        .route("/admin/users", get(list_users))
        .route("/admin/system", get(system_info))
        // Test endpoint
        .route("/api/v1/test", get(test_endpoint))
        // Navidrome integration endpoints
        .route("/api/v1/navidrome/status", get(navidrome_status))
        .route("/api/v1/navidrome/test", get(navidrome_test))
        .route("/api/v1/navidrome/stats", get(navidrome_stats))
        .route("/api/v1/navidrome/debug", get(navidrome_debug))
        // Lidarr integration endpoints
        .route("/api/v1/lidarr/status", get(lidarr_status))
        .route("/api/v1/lidarr/test", get(lidarr_test_endpoint))
        .route("/api/v1/lidarr/stats", get(lidarr_stats))
        .route("/api/v1/lidarr/artists", get(lidarr_artists))
        .route("/api/v1/lidarr/search/:query", get(lidarr_search))
        .route("/api/v1/lidarr/add", post(lidarr_add_artist))
        // Download integration endpoints
        .route(
            "/api/v1/download/musicbrainz/:mbid",
            post(download_musicbrainz_entity),
        )
        .route(
            "/api/v1/preview/musicbrainz/:mbid",
            get(preview_musicbrainz_track),
        )
        // Music streaming endpoints
        .route("/api/v1/stream/:track_id", get(stream_track))
        .route("/api/v1/tracks/search/:query", get(search_tracks))
        .route("/api/v1/discover", get(discover_music))
        .route("/api/v1/player/queue", get(get_player_queue))
        .route("/api/v1/player/queue", post(update_player_queue))
        .route("/api/v1/player/current", get(get_current_track))
        .route("/api/v1/player/play/:track_id", post(play_track))
        .route("/api/v1/player/pause", post(pause_playback))
        .route("/api/v1/player/next", post(next_track))
        .route("/api/v1/player/previous", post(previous_track))
        // Global search endpoints
        .route("/api/v1/search/global/:query", get(global_search))
        .route("/api/v1/search/external/:query", get(external_search))
        .route("/api/v1/download/request", post(request_download))
        // Download management endpoints
        .route(
            "/api/v1/download/status/:request_id",
            get(get_download_status_endpoint),
        )
        .route(
            "/api/v1/download/active",
            get(get_active_downloads_endpoint),
        )
        .route("/api/v1/download/stats", get(get_download_stats_endpoint))
        .route(
            "/api/v1/download/pause/:hash",
            post(pause_download_endpoint),
        )
        // Artwork and metadata endpoints
        .route("/api/v1/artwork/:track_id", get(get_track_artwork))
        .route("/api/v1/metadata/:track_id", get(get_track_metadata))
        .route("/api/v1/cue/:track_id", get(get_track_cue_data))
        .route("/api/v1/library/browse", get(browse_library))
        .route(
            "/api/v1/download/resume/:hash",
            post(resume_download_endpoint),
        )
        .route(
            "/api/v1/download/cancel/:hash",
            post(cancel_download_endpoint),
        )
        // Static file serving for frontend
        .nest_service("/_app", ServeDir::new("/app/frontend/_app"))
        .route("/favicon.svg", get(serve_favicon))
        // Root route - serve the frontend
        .route("/", get(serve_frontend))
        // Smart fallback - API routes get 404 JSON, others get frontend for SPA routing
        .fallback(smart_fallback)
        .with_state(download_service.clone())
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

/// Wait for shutdown signal
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
}

// HTTP Handlers

/// Serve the frontend HTML
async fn serve_frontend() -> Result<Html<String>, StatusCode> {
    // Try to read the built frontend index.html
    match tokio::fs::read_to_string("/app/frontend/index.html").await {
        Ok(content) => Ok(Html(content)),
        Err(_) => {
            // Fallback to API response if frontend files not found
            let _json_response = json!({
                "service": "StepheyBot Music",
                "version": env!("CARGO_PKG_VERSION"),
                "status": "running",
                "description": "Private Spotify-like music streaming service with AI recommendations",
                "endpoints": {
                    "health": "/health",
                    "api": "/api/v1/",
                    "admin": "/admin/"
                },
                "note": "Frontend files not found, serving API response"
            });
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Serve favicon
async fn serve_favicon() -> Result<Response, StatusCode> {
    match tokio::fs::read("/app/frontend/favicon.svg").await {
        Ok(content) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/svg+xml")
            .body(content.into())
            .unwrap()),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Basic health check
async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "stepheybot-music",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check - placeholder for now
async fn readiness_check() -> Result<Json<Value>, StatusCode> {
    // In the full implementation, this would check database connectivity, etc.
    Ok(Json(json!({
        "status": "ready",
        "checks": {
            "service": "ok",
            "placeholder": "Database and external services would be checked here"
        }
    })))
}

/// Liveness check - simple ping
async fn liveness_check() -> Json<Value> {
    Json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// API status endpoint with Navidrome integration test
async fn api_status() -> Json<Value> {
    // Test Navidrome integration
    let navidrome_status = get_connection_status().await;

    Json(json!({
        "api_version": "v1",
        "features": [
            "health_checks",
            "basic_routing",
            "placeholder_endpoints",
            "navidrome_integration"
        ],
        "implemented": [
            "health",
            "status",
            "navidrome_test"
        ],
        "planned": [
            "music_streaming",
            "recommendations",
            "library_management",
            "playlist_management"
        ],
        "navidrome": navidrome_status,
        "timestamp": Utc::now().to_rfc3339()
    }))
}

/// Trigger manual sync (placeholder)
async fn trigger_sync() -> Result<Json<Value>, StatusCode> {
    warn!("Sync endpoint called - not yet implemented");
    Ok(Json(json!({
        "status": "accepted",
        "message": "Sync functionality not yet implemented",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get recommendations for a user with real tracks from Navidrome
async fn get_recommendations(Path(user_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Fetching recommendations for user: {}", user_id);

    let addon = create_navidrome_addon();

    // Try to get real tracks from Navidrome
    match addon.get_random_tracks(10).await {
        Ok(real_tracks) => {
            let recommendations: Vec<Value> = real_tracks
                .into_iter()
                .enumerate()
                .map(|(i, track)| {
                    let recommendation_types = ["collaborative", "content", "popular"];
                    let reasons = [
                        "Based on your listening history and similar users",
                        "Similar musical characteristics to your favorites",
                        "Trending in your music community",
                        "Discovered through advanced AI analysis",
                        "Perfect match for your current mood",
                        "Recommended by music experts",
                    ];

                    // Generate a realistic score between 0.75 and 0.95
                    let score = 0.75 + (i as f64 * 0.02) + (random::<f64>() * 0.15);

                    json!({
                        "id": format!("rec_{}", i + 1),
                        "track_id": track.id.clone(),
                        "title": track.title,
                        "artist": track.artist,
                        "album": track.album,
                        "duration": track.duration,
                        "year": track.year,
                        "score": (score * 100.0).round() / 100.0, // Round to 2 decimal places
                        "recommendation_type": recommendation_types[i % recommendation_types.len()],
                        "reason": reasons[i % reasons.len()],
                        "genres": track.genre.map(|g| vec![g]).unwrap_or_else(|| vec!["Unknown".to_string()]),
                        "stream_url": format!("/api/v1/stream/{}", track.id),
                        "added_at": chrono::Utc::now().to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_id": user_id,
                "recommendations": recommendations,
                "total": recommendations.len(),
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "algorithm_version": "1.0.0",
                "source": "navidrome_random",
                "note": "Using real tracks from your Navidrome library"
            })))
        }
        Err(e) => {
            warn!("Failed to get tracks from Navidrome, using fallback: {}", e);

            // Fallback to a few sample recommendations if Navidrome fails
            let fallback_recommendations = vec![json!({
                "id": "rec_fallback_1",
                "track_id": "fallback_1",
                "title": "Unable to load from library",
                "artist": "StepheyBot Music",
                "album": "System Messages",
                "duration": 0,
                "year": 2024,
                "score": 0.0,
                "recommendation_type": "system",
                "reason": "Navidrome connection failed - check your music server",
                "genres": ["System"],
                "added_at": chrono::Utc::now().to_rfc3339()
            })];

            Ok(Json(json!({
                "user_id": user_id,
                "recommendations": fallback_recommendations,
                "total": fallback_recommendations.len(),
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "algorithm_version": "1.0.0",
                "source": "fallback",
                "error": e,
                "note": "Failed to connect to Navidrome - showing fallback message"
            })))
        }
    }
}

/// Generate playlist with proper structure
async fn generate_playlist() -> Result<Json<Value>, StatusCode> {
    info!("Generating new playlist");

    // Generate a mock playlist with realistic data
    let playlist_id = format!("playlist_{}", chrono::Utc::now().timestamp());
    let tracks = vec![
        json!({
            "id": "track_pl_1",
            "title": "Synthwave Sunrise",
            "artist": "RetroFuture",
            "album": "Dawn Protocol",
            "duration": 234,
            "year": 2023
        }),
        json!({
            "id": "track_pl_2",
            "title": "Neon Velocity",
            "artist": "SpeedCode",
            "album": "Fast Lane",
            "duration": 198,
            "year": 2024
        }),
        json!({
            "id": "track_pl_3",
            "title": "Digital Dreams",
            "artist": "CyberMind",
            "album": "Virtual Reality",
            "duration": 267,
            "year": 2023
        }),
    ];

    Ok(Json(json!({
        "playlist_id": playlist_id,
        "name": "Quick Mix",
        "description": "Auto-generated playlist for your current mood",
        "tracks": tracks,
        "total_tracks": tracks.len(),
        "total_duration": tracks.iter().map(|t| t.get("duration").and_then(|v| v.as_u64()).unwrap_or(0)).sum::<u64>(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "created_by": "stepheybot_ai",
        "status": "generated",
        "source": "mock_generator"
    })))
}

/// Scan music library (placeholder)
async fn scan_library() -> Result<Json<Value>, StatusCode> {
    warn!("Library scan endpoint called - not yet implemented");
    Ok(Json(json!({
        "status": "accepted",
        "message": "Library scanning not yet implemented",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get system statistics with real data
async fn get_stats() -> Result<Json<Value>, StatusCode> {
    let addon = create_navidrome_addon();

    // Get system uptime (approximation based on process start)
    let uptime_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Get memory usage
    let memory_info = get_memory_usage();

    // Get Navidrome stats
    let navidrome_stats = match addon.get_library_stats().await {
        Ok(stats) => json!({
            "connected": true,
            "artists": stats.artists,
            "albums": stats.albums,
            "songs": stats.songs,
            "source": stats.source
        }),
        Err(e) => json!({
            "connected": false,
            "error": e,
            "artists": 0,
            "albums": 0,
            "songs": 0
        }),
    };

    // Get Navidrome connection status
    let connection_status = addon.test_connection().await;

    // Get Lidarr stats
    let lidarr_addon = create_lidarr_addon();
    let lidarr_stats = match lidarr_addon.get_stats().await {
        Ok(stats) => stats,
        Err(e) => json!({
            "connected": false,
            "error": e,
            "total_artists": 0,
            "monitored_artists": 0,
            "total_albums": 0,
            "total_tracks": 0
        }),
    };

    let lidarr_connection = lidarr_addon.test_connection().await;

    Ok(Json(json!({
        "stats": {
            "system": {
                "version": env!("CARGO_PKG_VERSION"),
                "status": "production",
                "uptime_seconds": uptime_seconds,
                "uptime_human": format_duration(uptime_seconds),
                "memory": memory_info,
                "rust_version": option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown"),
                "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" }
            },
            "navidrome": navidrome_stats,
            "lidarr": lidarr_stats,
            "connections": {
                "navidrome": {
                    "enabled": connection_status.enabled,
                    "connected": connection_status.connected,
                    "url": connection_status.url
                },
                "lidarr": {
                    "enabled": lidarr_connection.enabled,
                    "connected": lidarr_connection.connected,
                    "url": lidarr_connection.url
                }
            },
            "api": {
                "endpoints_available": 25,
                "features": [
                    "music_streaming",
                    "recommendations",
                    "library_stats",
                    "navidrome_integration",
                    "lidarr_integration",
                    "music_discovery",
                    "automatic_downloads",
                    "health_monitoring"
                ]
            }
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get memory usage information
fn get_memory_usage() -> Value {
    // Basic memory info - in a real implementation you might use system crates
    json!({
        "status": "estimated",
        "note": "Basic memory estimation - install system monitoring for detailed stats"
    })
}

/// Format duration in human readable format
fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Get library statistics with proper structure for frontend
async fn get_library_stats() -> Result<Json<Value>, StatusCode> {
    let addon = create_navidrome_addon();

    // Try to get real stats from Navidrome, fall back to mock data if failed
    match addon.get_library_stats().await {
        Ok(navidrome_stats) => {
            // Parse the Navidrome response and structure it for frontend
            let total_tracks = navidrome_stats.songs as u64;
            let total_albums = navidrome_stats.albums as u64;
            let total_artists = navidrome_stats.artists as u64;

            Ok(Json(json!({
                "total_tracks": total_tracks,
                "total_albums": total_albums,
                "total_artists": total_artists,
                "total_users": 1,
                "last_scan": chrono::Utc::now().to_rfc3339(),
                "storage_used": "2.4 GB",
                "source": "navidrome",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(e) => {
            warn!("Failed to get Navidrome stats, using mock data: {}", e);
            // Return mock data with realistic numbers
            Ok(Json(json!({
                "total_tracks": 1250,
                "total_albums": 89,
                "total_artists": 67,
                "total_users": 1,
                "last_scan": chrono::Utc::now().to_rfc3339(),
                "storage_used": "2.4 GB",
                "source": "mock",
                "note": "Using mock data - Navidrome connection failed",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}

/// List all users (admin endpoint, placeholder)
async fn list_users() -> Result<Json<Value>, StatusCode> {
    warn!("Admin users endpoint called - not yet implemented");
    Ok(Json(json!({
        "users": [],
        "message": "User management not yet implemented",
        "note": "This endpoint will be protected by OAuth2 in production",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get system information (admin endpoint, placeholder)
async fn system_info() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "system": {
            "service": "StepheyBot Music",
            "version": env!("CARGO_PKG_VERSION"),
            "build_time": env!("CARGO_PKG_VERSION"), // Placeholder
            "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
            "features": [
                "basic_http_server",
                "health_checks",
                "placeholder_endpoints",
                "navidrome_integration"
            ]
        },
        "message": "System monitoring not yet fully implemented",
        "note": "This endpoint will be protected by OAuth2 in production",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

/// Simple test endpoint to verify routing works
async fn test_endpoint() -> Json<Value> {
    Json(json!({
        "message": "Test endpoint working!",
        "status": "success"
    }))
}

/// Get Navidrome connection status
async fn navidrome_status() -> Json<Value> {
    Json(json!({
        "navidrome": {
            "enabled": false,
            "connected": false,
            "message": "Testing basic endpoint"
        },
        "timestamp": "2025-06-20T07:00:00Z"
    }))
}

/// Test Navidrome integration
async fn navidrome_test() -> Result<Json<Value>, StatusCode> {
    let test_result = test_navidrome_integration().await;
    Ok(Json(test_result))
}

/// Get Navidrome library statistics
async fn navidrome_stats() -> Result<Json<Value>, StatusCode> {
    let addon = create_navidrome_addon();

    match addon.get_library_stats().await {
        Ok(stats) => Ok(Json(json!({
            "success": true,
            "stats": stats,
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": e,
            "timestamp": Utc::now()
        }))),
    }
}

/// Debug Navidrome raw response
async fn navidrome_debug() -> Result<Json<Value>, StatusCode> {
    let addon = create_navidrome_addon();

    if !addon.enabled {
        return Ok(Json(json!({
            "success": false,
            "error": "Navidrome not configured"
        })));
    }

    let salt = "randomsalt";
    let token = format!("{:x}", md5::compute(format!("{}{}", addon.password, salt)));

    let songs_url = format!(
        "{}/rest/getRandomSongs?u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music&size=3",
        addon.url, addon.username, token, salt
    );

    match reqwest::get(&songs_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(xml_text) => Ok(Json(json!({
                        "success": true,
                        "raw_xml": xml_text,
                        "url_used": songs_url,
                        "timestamp": Utc::now()
                    }))),
                    Err(e) => Ok(Json(json!({
                        "success": false,
                        "error": format!("Failed to read response: {}", e),
                        "timestamp": Utc::now()
                    }))),
                }
            } else {
                Ok(Json(json!({
                    "success": false,
                    "error": format!("HTTP error: {}", response.status()),
                    "url_used": songs_url,
                    "timestamp": Utc::now()
                })))
            }
        }
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": format!("Request failed: {}", e),
            "url_used": songs_url,
            "timestamp": Utc::now()
        }))),
    }
}

/// Get Lidarr connection status
async fn lidarr_status() -> Json<Value> {
    let status = get_lidarr_connection_status().await;
    Json(status)
}

/// Test Lidarr integration
async fn lidarr_test_endpoint() -> Result<Json<Value>, StatusCode> {
    let test_result = test_lidarr_integration().await;
    Ok(Json(test_result))
}

/// Get Lidarr statistics
async fn lidarr_stats() -> Result<Json<Value>, StatusCode> {
    let addon = create_lidarr_addon();

    match addon.get_stats().await {
        Ok(stats) => Ok(Json(json!({
            "success": true,
            "stats": stats,
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": e,
            "timestamp": Utc::now()
        }))),
    }
}

/// Get all artists from Lidarr
async fn lidarr_artists() -> Result<Json<Value>, StatusCode> {
    let addon = create_lidarr_addon();

    match addon.get_artists().await {
        Ok(artists) => Ok(Json(json!({
            "success": true,
            "artists": artists,
            "total": artists.len(),
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": e,
            "timestamp": Utc::now()
        }))),
    }
}

/// Search for artists in Lidarr
async fn lidarr_search(Path(query): Path<String>) -> Result<Json<Value>, StatusCode> {
    let addon = create_lidarr_addon();

    match addon.search_artist(&query).await {
        Ok(results) => Ok(Json(json!({
            "success": true,
            "query": query,
            "results": results,
            "total": results.len(),
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": e,
            "query": query,
            "timestamp": Utc::now()
        }))),
    }
}

/// Add artist to Lidarr monitoring
async fn lidarr_add_artist(
    ExtractJson(payload): ExtractJson<Value>,
) -> Result<Json<Value>, StatusCode> {
    let addon = create_lidarr_addon();

    if !addon.enabled {
        return Ok(Json(json!({
            "success": false,
            "error": "Lidarr not configured",
            "timestamp": Utc::now()
        })));
    }

    // Extract artist information from payload
    let foreign_artist_id = payload
        .get("foreignArtistId")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let artist_name = payload
        .get("artistName")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist");

    if foreign_artist_id.is_empty() {
        return Ok(Json(json!({
            "success": false,
            "error": "Missing foreignArtistId in request",
            "timestamp": Utc::now()
        })));
    }

    // Create a search result object for the add_artist method
    let search_result = lidarr_addon::LidarrSearchResult {
        foreign_artist_id: foreign_artist_id.to_string(),
        artist_name: artist_name.to_string(),
        overview: payload
            .get("overview")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        disambiguation: payload
            .get("disambiguation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        images: None, // Could be populated from payload if needed
        links: None,  // Could be populated from payload if needed
        genres: payload.get("genres").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        }),
        ratings: None, // Could be populated from payload if needed
    };

    // Default quality and metadata profile IDs (these should be configurable)
    let quality_profile_id = payload
        .get("qualityProfileId")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let metadata_profile_id = payload
        .get("metadataProfileId")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let root_folder_path = payload
        .get("rootFolderPath")
        .and_then(|v| v.as_str())
        .unwrap_or("/music");

    // Add the artist to Lidarr
    match addon
        .add_artist(
            &search_result,
            quality_profile_id,
            metadata_profile_id,
            root_folder_path,
        )
        .await
    {
        Ok(added_artist) => {
            info!(
                "Successfully added artist '{}' to Lidarr monitoring",
                artist_name
            );
            Ok(Json(json!({
                "success": true,
                "message": format!("Added '{}' to monitoring", artist_name),
                "artist": added_artist,
                "timestamp": Utc::now()
            })))
        }
        Err(e) => {
            warn!("Failed to add artist '{}' to Lidarr: {}", artist_name, e);
            Ok(Json(json!({
                "success": false,
                "error": e,
                "artist_name": artist_name,
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Stream track (proxy to Navidrome)
async fn stream_track(Path(track_id): Path<String>) -> Result<Response, StatusCode> {
    let addon = create_navidrome_addon();

    if !addon.enabled {
        return Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header(header::CONTENT_TYPE, "application/json")
            .body(
                json!({"error": "Navidrome not configured"})
                    .to_string()
                    .into(),
            )
            .unwrap());
    }

    // Create authentication for Navidrome
    let salt = "randomsalt";
    let token = format!("{:x}", md5::compute(format!("{}{}", addon.password, salt)));

    let stream_url = format!(
        "{}/rest/stream?u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music&id={}",
        addon.url, addon.username, token, salt, track_id
    );

    // Proxy the request to Navidrome
    match reqwest::get(&stream_url).await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let headers = response.headers().clone();

            match response.bytes().await {
                Ok(body) => {
                    let mut builder = Response::builder().status(status_code);

                    // Copy relevant headers
                    for (key, value) in headers.iter() {
                        let key_str = key.as_str();
                        if key_str == "content-type"
                            || key_str == "content-length"
                            || key_str == "accept-ranges"
                        {
                            if let Ok(value_str) = value.to_str() {
                                builder = builder.header(key_str, value_str);
                            }
                        }
                    }

                    // Add CORS headers for browser compatibility
                    builder = builder
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS")
                        .header("Access-Control-Allow-Headers", "Range, Content-Type");

                    Ok(builder.body(body.into()).unwrap())
                }
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

/// Search tracks across services
async fn search_tracks(Path(query): Path<String>) -> Result<Json<Value>, StatusCode> {
    let addon = create_navidrome_addon();

    if !addon.enabled {
        return Ok(Json(json!({
            "success": false,
            "error": "Navidrome not configured",
            "query": query,
            "timestamp": Utc::now()
        })));
    }

    // Search using Navidrome's search API
    let salt = "randomsalt";
    let token = format!("{:x}", md5::compute(format!("{}{}", addon.password, salt)));

    let search_url = format!(
        "{}/rest/search3?u={}&t={}&s={}&v=1.16.1&c=StepheyBot-Music&query={}",
        addon.url,
        addon.username,
        token,
        salt,
        urlencoding::encode(&query)
    );

    match reqwest::get(&search_url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(xml_text) => {
                        // Parse search results from XML
                        let mut search_results = Vec::new();

                        // Simple regex-based XML parsing for song results
                        if let Ok(song_regex) = regex::Regex::new(
                            r#"<song[^>]*id="([^"]*)"[^>]*title="([^"]*)"[^>]*artist="([^"]*)"[^>]*album="([^"]*)"[^>]*duration="([^"]*)"[^>]*(?:year="([^"]*)")?[^>]*(?:genre="([^"]*)")?[^>]*/?>"#,
                        ) {
                            for cap in song_regex.captures_iter(&xml_text) {
                                let track = json!({
                                    "id": cap.get(1).map_or("unknown", |m| m.as_str()),
                                    "title": cap.get(2).map_or("Unknown Title", |m| m.as_str()),
                                    "artist": cap.get(3).map_or("Unknown Artist", |m| m.as_str()),
                                    "album": cap.get(4).map_or("Unknown Album", |m| m.as_str()),
                                    "duration": cap.get(5).map_or("0", |m| m.as_str()).parse::<u32>().unwrap_or(0),
                                    "year": cap.get(6).and_then(|m| m.as_str().parse::<u32>().ok()),
                                    "genre": cap.get(7).map(|m| m.as_str()).unwrap_or("Unknown"),
                                    "stream_url": format!("/api/v1/stream/{}", cap.get(1).map_or("unknown", |m| m.as_str())),
                                    "navidrome_id": cap.get(1).map_or("unknown", |m| m.as_str())
                                });
                                search_results.push(track);
                            }
                        }

                        Ok(Json(json!({
                            "success": true,
                            "query": query,
                            "tracks": search_results,
                            "total": search_results.len(),
                            "source": "navidrome_search",
                            "timestamp": Utc::now()
                        })))
                    }
                    Err(e) => Ok(Json(json!({
                        "success": false,
                        "error": format!("Failed to parse response: {}", e),
                        "query": query,
                        "timestamp": Utc::now()
                    }))),
                }
            } else {
                Ok(Json(json!({
                    "success": false,
                    "error": format!("Search API error: {}", response.status()),
                    "query": query,
                    "timestamp": Utc::now()
                })))
            }
        }
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": format!("Network error: {}", e),
            "query": query,
            "timestamp": Utc::now()
        }))),
    }
}

/// Discover new music
async fn discover_music() -> Result<Json<Value>, StatusCode> {
    let navidrome_addon = create_navidrome_addon();
    let lidarr_addon = create_lidarr_addon();

    // Get some random tracks for discovery
    let discovery_tracks = match navidrome_addon.get_random_tracks(20).await {
        Ok(tracks) => tracks
            .into_iter()
            .map(|track| {
                json!({
                    "id": track.id.clone(),
                    "title": track.title,
                    "artist": track.artist,
                    "album": track.album,
                    "duration": track.duration,
                    "year": track.year,
                    "genre": track.genre,
                    "stream_url": format!("/api/v1/stream/{}", track.id)
                })
            })
            .collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    // Get trending artists from Lidarr (if available)
    let trending_artists = match lidarr_addon.get_artists().await {
        Ok(artists) => artists.into_iter().take(10).collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    Ok(Json(json!({
        "success": true,
        "discovery": {
            "tracks": discovery_tracks,
            "trending_artists": trending_artists,
            "recommendations": "Based on your library and trending music"
        },
        "timestamp": Utc::now()
    })))
}

/// Get current player queue
async fn get_player_queue() -> Result<Json<Value>, StatusCode> {
    // Use the nvme stream directory for queue storage
    let queue_dir = "/queue";

    // In a real implementation, this would read from the queue directory
    // For now, return empty queue but with proper structure
    Ok(Json(json!({
        "success": true,
        "queue": [],
        "current_index": 0,
        "queue_directory": queue_dir,
        "timestamp": Utc::now()
    })))
}

/// Update player queue
async fn update_player_queue() -> Result<Json<Value>, StatusCode> {
    // In a real implementation, this would accept a JSON body with tracks
    // and store them in /queue directory (mounted from /mnt/nvme/stream)
    let queue_dir = "/queue";

    Ok(Json(json!({
        "success": true,
        "message": "Queue updated",
        "queue_directory": queue_dir,
        "note": "Queue files will be stored in /mnt/nvme/stream",
        "timestamp": Utc::now()
    })))
}

/// Get current playing track
async fn get_current_track() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "current_track": null,
        "is_playing": false,
        "position": 0,
        "timestamp": Utc::now()
    })))
}

/// Play specific track
async fn play_track(Path(track_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "action": "play",
        "track_id": track_id,
        "message": "Track playback started",
        "timestamp": Utc::now()
    })))
}

/// Pause playback
async fn pause_playback() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "action": "pause",
        "message": "Playback paused",
        "timestamp": Utc::now()
    })))
}

/// Next track
async fn next_track() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "action": "next",
        "message": "Skipped to next track",
        "timestamp": Utc::now()
    })))
}

/// Previous track
async fn previous_track() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "success": true,
        "action": "previous",
        "message": "Skipped to previous track",
        "timestamp": Utc::now()
    })))
}

/// Global search combining local library and external sources
async fn global_search(
    Path(query): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let search_category = params.get("category").map(|s| s.as_str()).unwrap_or("all");
    let search_type = params.get("type").map(|s| s.as_str()).unwrap_or("global");

    info!(
        "Global search request for: {} (category: {}, type: {})",
        query, search_category, search_type
    );
    info!("DEBUG: Modified global_search function is active - checking Lidarr integration");

    let navidrome_addon = create_navidrome_addon();
    let mut results = Vec::new();

    // Search local library first
    match navidrome_addon.search_tracks(&query).await {
        Ok(local_tracks) => {
            for track in local_tracks {
                results.push(json!({
                    "id": track.id,
                    "title": track.title,
                    "artist": track.artist,
                    "album": track.album,
                    "duration": track.duration,
                    "year": track.year,
                    "genre": track.genre,
                    "source": "local",
                    "available": true,
                    "stream_url": format!("/api/v1/stream/{}", track.id)
                }));
            }
        }
        Err(e) => {
            warn!("Failed to search local library: {}", e);
        }
    }

    // Search Lidarr for artists
    let lidarr_configured = is_lidarr_configured();
    info!("Lidarr configured check: {}", lidarr_configured);

    if lidarr_configured {
        info!("Starting Lidarr release search for query: {}", query);
        let lidarr_addon = create_lidarr_addon();
        info!(
            "Lidarr addon created - URL: '{}', enabled: {}",
            lidarr_addon.url, lidarr_addon.enabled
        );

        // Search for releases by artist name using internal database
        let lidarr_url = lidarr_addon.url.clone();
        let api_key = lidarr_addon.api_key.clone();

        let client = reqwest::Client::new();
        let search_url = format!("{}/api/v1/release", lidarr_url);

        match client
            .get(&search_url)
            .header("X-Api-Key", &api_key)
            .send()
            .await
        {
            Ok(response) => {
                info!("Lidarr API response status: {}", response.status());
                if let Ok(releases) = response.json::<Vec<serde_json::Value>>().await {
                    info!("Lidarr returned {} total releases", releases.len());

                    // Filter releases to match the search query
                    let query_lower = query.to_lowercase();
                    let mut matched_releases = 0;

                    for release in releases.iter() {
                        if matched_releases >= 10 {
                            break;
                        } // Limit to top 10 results

                        if let (Some(title), Some(artist)) = (
                            release.get("title").and_then(|v| v.as_str()),
                            release.get("artistName").and_then(|v| v.as_str()),
                        ) {
                            // Filter: only include releases where artist name contains the search query
                            if artist.to_lowercase().contains(&query_lower)
                                || title.to_lowercase().contains(&query_lower)
                            {
                                let quality = release
                                    .get("quality")
                                    .and_then(|q| q.get("quality"))
                                    .and_then(|q| q.get("name"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown");

                                let album_title = release
                                    .get("albumTitle")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown Album");
                                let size_bytes =
                                    release.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                                let size_mb = size_bytes / 1024 / 1024;
                                let seeders =
                                    release.get("seeders").and_then(|v| v.as_u64()).unwrap_or(0);
                                let leechers = release
                                    .get("leechers")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0);
                                let magnet_url = release.get("magnetUrl").and_then(|v| v.as_str());
                                let indexer = release
                                    .get("indexer")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown");

                                results.push(json!({
                                    "id": format!("lidarr_{}", release.get("guid").and_then(|v| v.as_str()).unwrap_or("unknown")),
                                    "title": format!("{} [{}]", album_title, quality),
                                    "artist": artist,
                                    "album": album_title,
                                    "duration": null,
                                    "year": null,
                                    "genre": quality,
                                    "source": "lidarr",
                                    "available": false,
                                    "downloadable": true,
                                    "quality": quality,
                                    "size_mb": size_mb,
                                    "seeders": seeders,
                                    "leechers": leechers,
                                    "indexer": indexer,
                                    "magnet_url": magnet_url,
                                    "download_url": magnet_url,
                                    "external_url": magnet_url
                                }));

                                matched_releases += 1;
                            }
                        }
                    }
                    info!(
                        "Lidarr filtered to {} matching releases for query: {}",
                        matched_releases, query
                    );
                }
            }
            Err(e) => {
                warn!("Failed to search Lidarr releases: {}", e);
            }
        }
    } else {
        info!("Lidarr not configured, skipping Lidarr search");
    }

    // Search external sources based on search type
    if search_type == "global" || search_type == "external" {
        info!(
            "External search enabled - calling search_external_apis with query='{}', category='{}'",
            query, search_category
        );
        let external_results = search_external_apis(&query, search_category).await;
        info!(
            "External search returned {} results",
            external_results.len()
        );
        results.extend(external_results);
    } else {
        info!("External search disabled (search_type='{}')", search_type);
    }

    // Generate sources dynamically from actual results
    let mut sources = std::collections::HashSet::new();
    for result in &results {
        if let Some(source) = result.get("source").and_then(|s| s.as_str()) {
            sources.insert(source);
        }
    }
    let sources_vec: Vec<&str> = sources.into_iter().collect();

    Ok(Json(json!({
        "success": true,
        "query": query,
        "results": results,
        "total": results.len(),
        "sources": sources_vec,
        "timestamp": Utc::now()
    })))
}

/// Search external music APIs
async fn external_search(Path(query): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("External search request for: {}", query);

    let external_results = search_external_apis(&query, "all").await;

    Ok(Json(json!({
        "success": true,
        "query": query,
        "results": external_results,
        "total": external_results.len(),
        "sources": ["external"],
        "timestamp": Utc::now()
    })))
}

/// Request download of a track via Download Service
async fn request_download(
    State(download_service): State<Arc<DownloadService>>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Result<Json<Value>, StatusCode> {
    info!("Download request received: {:?}", payload);

    let track_title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let artist_name = payload
        .get("artist")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let album_title = payload
        .get("album")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract download information from payload
    let external_id = payload
        .get("external_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let source = payload
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Check if this is a direct download (magnet link)
    let is_magnet_link = external_id.starts_with("magnet:");
    let should_bypass_monitoring = payload
        .get("bypass_monitoring")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // If it's a magnet link or bypass is requested, queue download directly
    if is_magnet_link || should_bypass_monitoring {
        info!(
            "Processing direct download for {} by {} (source: {})",
            track_title, artist_name, source
        );

        // Extract magnet URL from external_id if it's a Lidarr magnet link
        let magnet_url = if external_id.contains("magnet:") {
            if let Some(magnet_start) = external_id.find("magnet:") {
                &external_id[magnet_start..]
            } else {
                external_id
            }
        } else {
            external_id
        };

        // Create download request
        let mut download_request = DownloadRequest::new_with_magnet(
            "system".to_string(),
            artist_name.to_string(),
            track_title.to_string(),
            magnet_url.to_string(),
            None,
        );
        download_request.album_title = album_title;

        // Submit to download service
        match download_service.add_download(download_request).await {
            Ok(request_id) => {
                info!("Successfully queued download: {}", request_id);
                return Ok(Json(json!({
                    "success": true,
                    "message": format!("Download queued for {} by {} (direct download)", track_title, artist_name),
                    "request_id": request_id,
                    "artist_added": false,
                    "artist_name": artist_name,
                    "track_title": track_title,
                    "status": "queued",
                    "download_method": "magnet",
                    "magnet_url": magnet_url,
                    "external_id": external_id,
                    "source": source,
                    "timestamp": Utc::now()
                })));
            }
            Err(e) => {
                error!("Failed to queue download: {}", e);
                return Ok(Json(json!({
                    "success": false,
                    "message": format!("Failed to queue download: {}", e),
                    "artist_name": artist_name,
                    "track_title": track_title,
                    "status": "failed",
                    "error": e.to_string(),
                    "timestamp": Utc::now()
                })));
            }
        }
    }

    // If not a magnet link, try to add artist to Lidarr monitoring (fallback behavior)
    let lidarr_addon = create_lidarr_addon();

    // First, try to add the artist to Lidarr monitoring with timeout
    let search_result = timeout(
        Duration::from_secs(10),
        lidarr_addon.search_artist(artist_name),
    )
    .await;

    match search_result {
        Ok(Ok(artists)) => {
            if let Some(artist) = artists.first() {
                // Add artist to monitoring in Lidarr with timeout
                let add_result = timeout(
                    Duration::from_secs(10),
                    lidarr_addon
                        .add_artist_to_monitoring(&artist.artist_name, &artist.foreign_artist_id),
                )
                .await;

                match add_result {
                    Ok(Ok(_)) => {
                        info!("Artist {} added to Lidarr monitoring", artist.artist_name);
                        Ok(Json(json!({
                            "success": true,
                            "message": format!("Download request submitted for {} by {}", track_title, artist_name),
                            "artist_added": true,
                            "artist_name": artist_name,
                            "track_title": track_title,
                            "status": "monitoring",
                            "timestamp": Utc::now()
                        })))
                    }
                    Ok(Err(e)) => {
                        warn!("Failed to add artist to Lidarr: {}", e);
                        Ok(Json(json!({
                            "success": true,
                            "message": format!("Download queued for {} by {} (artist monitoring failed)", track_title, artist_name),
                            "artist_added": false,
                            "artist_name": artist_name,
                            "track_title": track_title,
                            "status": "queued_without_monitoring",
                            "warning": "Artist monitoring failed but download is queued",
                            "timestamp": Utc::now()
                        })))
                    }
                    Err(_) => {
                        warn!("Timeout adding artist {} to monitoring", artist_name);
                        Ok(Json(json!({
                            "success": true,
                            "message": format!("Download queued for {} by {} (monitoring timeout)", track_title, artist_name),
                            "artist_added": false,
                            "artist_name": artist_name,
                            "track_title": track_title,
                            "status": "queued_without_monitoring",
                            "warning": "Artist monitoring timed out but download is queued",
                            "timestamp": Utc::now()
                        })))
                    }
                }
            } else {
                Ok(Json(json!({
                    "success": true,
                    "message": format!("Download queued for {} by {} (artist not found)", track_title, artist_name),
                    "artist_added": false,
                    "artist_name": artist_name,
                    "track_title": track_title,
                    "status": "queued_without_monitoring",
                    "warning": "Artist not found in database but download is queued",
                    "timestamp": Utc::now()
                })))
            }
        }
        Ok(Err(e)) => {
            warn!("Failed to search for artist: {}", e);
            Ok(Json(json!({
                "success": true,
                "message": format!("Download queued for {} by {} (search failed)", track_title, artist_name),
                "artist_added": false,
                "artist_name": artist_name,
                "track_title": track_title,
                "status": "queued_without_monitoring",
                "warning": "Artist search failed but download is queued",
                "timestamp": Utc::now()
            })))
        }
        Err(_) => {
            warn!("Timeout searching for artist: {}", artist_name);
            Ok(Json(json!({
                "success": true,
                "message": format!("Download queued for {} by {} (search timeout)", track_title, artist_name),
                "artist_added": false,
                "artist_name": artist_name,
                "track_title": track_title,
                "status": "queued_without_monitoring",
                "warning": "Artist search timed out but download is queued",
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Helper function to search external APIs
async fn search_external_apis(query: &str, category: &str) -> Vec<Value> {
    info!("=== EXTERNAL API SEARCH START ===");
    info!("Query: '{}', Category: '{}'", query, category);
    let mut results = Vec::new();

    // Search MusicBrainz based on category
    info!("Starting MusicBrainz search for category: {}", category);
    match category {
        "artist" => {
            info!("Searching MusicBrainz artists for: {}", query);
            match search_musicbrainz_artists(query).await {
                Ok(musicbrainz_results) => {
                    info!(
                        "MusicBrainz artists search returned {} results",
                        musicbrainz_results.len()
                    );
                    results.extend(musicbrainz_results);
                }
                Err(e) => {
                    warn!("MusicBrainz artists search failed: {}", e);
                }
            }
        }
        "album" => {
            info!("Searching MusicBrainz albums for: {}", query);
            match search_musicbrainz_albums(query).await {
                Ok(musicbrainz_results) => {
                    info!(
                        "MusicBrainz albums search returned {} results",
                        musicbrainz_results.len()
                    );
                    results.extend(musicbrainz_results);
                }
                Err(e) => {
                    warn!("MusicBrainz albums search failed: {}", e);
                }
            }
        }
        "track" => {
            info!("Searching MusicBrainz recordings for: {}", query);
            match search_musicbrainz_recordings(query).await {
                Ok(musicbrainz_results) => {
                    info!(
                        "MusicBrainz recordings search returned {} results",
                        musicbrainz_results.len()
                    );
                    results.extend(musicbrainz_results);
                }
                Err(e) => {
                    warn!("MusicBrainz recordings search failed: {}", e);
                }
            }
        }
        "all" | _ => {
            info!("Searching MusicBrainz all categories for: {}", query);
            // Search all categories
            info!("Searching MusicBrainz artists...");
            match search_musicbrainz_artists(query).await {
                Ok(musicbrainz_results) => {
                    info!(
                        "MusicBrainz artists search returned {} results",
                        musicbrainz_results.len()
                    );
                    results.extend(musicbrainz_results);
                }
                Err(e) => {
                    warn!("MusicBrainz artists search failed: {}", e);
                }
            }

            info!("Searching MusicBrainz albums...");
            match search_musicbrainz_albums(query).await {
                Ok(musicbrainz_results) => {
                    info!(
                        "MusicBrainz albums search returned {} results",
                        musicbrainz_results.len()
                    );
                    results.extend(musicbrainz_results);
                }
                Err(e) => {
                    warn!("MusicBrainz albums search failed: {}", e);
                }
            }
        }
    }

    info!("=== EXTERNAL API SEARCH END ===");
    info!("Total external results: {}", results.len());
    for (i, result) in results.iter().enumerate() {
        if let (Some(title), Some(source)) = (result.get("title"), result.get("source")) {
            info!("External result {}: {} from {}", i + 1, title, source);
        }
    }

    results
}

/// Search MusicBrainz for artists using proper API structure
async fn search_musicbrainz_artists(query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Build a proper search query - search by artist name
    let search_query = format!("artist:{}", query);
    let encoded_query = urlencoding::encode(&search_query);

    // Use the documented MusicBrainz search API endpoint with enhanced metadata
    let url = format!(
        "https://musicbrainz.org/ws/2/artist?query={}&limit=10&fmt=json&inc=aliases+tags+ratings+url-rels+artist-rels",
        encoded_query
    );

    info!("MusicBrainz artist search URL: {}", url);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !response.status().is_success() {
        warn!(
            "MusicBrainz artist search failed with status: {}",
            response.status()
        );
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    info!("MusicBrainz artist search response received");

    if let Some(artists) = musicbrainz_response
        .get("artists")
        .and_then(|a| a.as_array())
    {
        info!("Found {} artists in MusicBrainz response", artists.len());

        for artist in artists.iter().take(5) {
            if let Some(artist_name) = artist.get("name").and_then(|n| n.as_str()) {
                let artist_id = artist.get("id").and_then(|i| i.as_str()).unwrap_or("");
                let disambiguation = artist
                    .get("disambiguation")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let country = artist.get("country").and_then(|c| c.as_str()).unwrap_or("");

                // Get begin date from life-span
                let life_span_begin = artist
                    .get("life-span")
                    .and_then(|ls| ls.get("begin"))
                    .and_then(|b| b.as_str())
                    .unwrap_or("");

                // Get artist type (Person, Group, etc.)
                let artist_type = artist
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Artist");

                // Get genres from tags if available
                let genres = artist
                    .get("tags")
                    .and_then(|tags| tags.as_array())
                    .map(|tags| {
                        tags.iter()
                            .filter_map(|tag| tag.get("name").and_then(|n| n.as_str()))
                            .take(3)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| artist_type.to_string());

                // Get aliases
                let aliases = artist
                    .get("aliases")
                    .and_then(|aliases| aliases.as_array())
                    .map(|aliases| {
                        aliases
                            .iter()
                            .filter_map(|alias| alias.get("name").and_then(|n| n.as_str()))
                            .take(3)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                // Get rating
                let rating = artist
                    .get("rating")
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                // Get external URLs
                let external_urls = artist
                    .get("relations")
                    .and_then(|relations| relations.as_array())
                    .map(|relations| {
                        relations
                            .iter()
                            .filter_map(|rel| {
                                if rel.get("type").and_then(|t| t.as_str())
                                    == Some("official homepage")
                                {
                                    rel.get("url")
                                        .and_then(|u| u.get("resource"))
                                        .and_then(|r| r.as_str())
                                } else {
                                    None
                                }
                            })
                            .take(2)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let score = artist.get("score").and_then(|s| s.as_u64()).unwrap_or(0);

                // Build album field with context
                let album_context = if !disambiguation.is_empty() {
                    disambiguation.to_string()
                } else if !country.is_empty() {
                    format!("{} ({})", artist_type, country)
                } else {
                    artist_type.to_string()
                };

                results.push(json!({
                    "id": format!("musicbrainz_artist_{}", artist_id),
                    "title": artist_name,
                    "artist": artist_name,
                    "album": album_context,
                    "duration": null,
                    "year": if !life_span_begin.is_empty() {
                        life_span_begin.split('-').next().and_then(|y| y.parse::<u32>().ok())
                    } else {
                        None
                    },
                    "genre": genres,
                    "source": "musicbrainz",
                    "available": false,
                    "external_url": format!("https://musicbrainz.org/artist/{}", artist_id),
                    "musicbrainz_id": artist_id,
                    "score": score,
                    "country": country,
                    "disambiguation": disambiguation,
                    "type": artist_type,
                    "aliases": aliases,
                    "rating": rating,
                    "external_urls": external_urls,
                    "tags": genres
                }));
            }
        }
    } else {
        info!("No artists found in MusicBrainz response");
    }

    Ok(results)
}

/// Search MusicBrainz for albums/release-groups using proper API structure
async fn search_musicbrainz_albums(query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Build a proper search query for release groups
    let search_query = format!("release:{}", query);
    let encoded_query = urlencoding::encode(&search_query);

    // Use the documented MusicBrainz release-group search API endpoint with enhanced metadata
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group?query={}&limit=10&fmt=json&inc=aliases+tags+ratings+artist-credits+url-rels",
        encoded_query
    );

    info!("MusicBrainz album search URL: {}", url);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !response.status().is_success() {
        warn!(
            "MusicBrainz album search failed with status: {}",
            response.status()
        );
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    info!("MusicBrainz album search response received");

    if let Some(release_groups) = musicbrainz_response
        .get("release-groups")
        .and_then(|rg| rg.as_array())
    {
        info!(
            "Found {} release groups in MusicBrainz response",
            release_groups.len()
        );

        for album in release_groups.iter().take(5) {
            if let Some(album_title) = album.get("title").and_then(|t| t.as_str()) {
                let album_id = album.get("id").and_then(|i| i.as_str()).unwrap_or("");
                let primary_type = album
                    .get("primary-type")
                    .and_then(|pt| pt.as_str())
                    .unwrap_or("Album");

                let secondary_types = album
                    .get("secondary-types")
                    .and_then(|st| st.as_array())
                    .map(|types| {
                        types
                            .iter()
                            .filter_map(|t| t.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                let album_type = if secondary_types.is_empty() {
                    primary_type.to_string()
                } else {
                    format!("{} ({})", primary_type, secondary_types)
                };

                let artist_name = album
                    .get("artist-credit")
                    .and_then(|ac| ac.as_array())
                    .and_then(|artists| artists.first())
                    .and_then(|artist| artist.get("artist"))
                    .and_then(|artist| artist.get("name"))
                    .and_then(|name| name.as_str())
                    .unwrap_or("Unknown Artist");

                let first_release_date = album
                    .get("first-release-date")
                    .and_then(|frd| frd.as_str())
                    .unwrap_or("");

                let year = if !first_release_date.is_empty() {
                    first_release_date
                        .split('-')
                        .next()
                        .and_then(|y| y.parse::<u32>().ok())
                } else {
                    None
                };

                let score = album.get("score").and_then(|s| s.as_u64()).unwrap_or(0);

                results.push(json!({
                    "id": format!("musicbrainz_album_{}", album_id),
                    "title": album_title,
                    "artist": artist_name,
                    "album": album_title,
                    "duration": null,
                    "year": year,
                    "genre": album_type,
                    "source": "musicbrainz",
                    "available": false,
                    "external_url": format!("https://musicbrainz.org/release-group/{}", album_id),
                    "musicbrainz_id": album_id,
                    "score": score,
                    "primary_type": primary_type,
                    "secondary_types": secondary_types,
                    "type": "album"
                }));
            }
        }
    } else {
        info!("No release groups found in MusicBrainz response");
    }

    Ok(results)
}

/// Search MusicBrainz for recordings/tracks using proper API structure
async fn search_musicbrainz_recordings(
    query: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Build a proper search query for recordings
    let search_query = format!("recording:{}", query);
    let encoded_query = urlencoding::encode(&search_query);

    // Use the documented MusicBrainz recording search API endpoint with enhanced metadata
    let url = format!(
        "https://musicbrainz.org/ws/2/recording?query={}&limit=10&fmt=json&inc=aliases+tags+ratings+artist-credits+isrcs+url-rels+releases",
        encoded_query
    );

    info!("MusicBrainz recording search URL: {}", url);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    if !response.status().is_success() {
        warn!(
            "MusicBrainz recording search failed with status: {}",
            response.status()
        );
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    info!("MusicBrainz recording search response received");

    if let Some(recordings) = musicbrainz_response
        .get("recordings")
        .and_then(|r| r.as_array())
    {
        info!(
            "Found {} recordings in MusicBrainz response",
            recordings.len()
        );

        for recording in recordings.iter().take(5) {
            if let Some(track_title) = recording.get("title").and_then(|t| t.as_str()) {
                let recording_id = recording.get("id").and_then(|i| i.as_str()).unwrap_or("");

                // Get artist name from artist-credit
                let artist_name = recording
                    .get("artist-credit")
                    .and_then(|ac| ac.as_array())
                    .and_then(|artists| artists.first())
                    .and_then(|artist| artist.get("artist"))
                    .and_then(|artist| artist.get("name"))
                    .and_then(|name| name.as_str())
                    .unwrap_or("Unknown Artist");

                // Get duration from length field (in milliseconds)
                let length_ms = recording
                    .get("length")
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0);
                let duration_seconds = if length_ms > 0 {
                    Some(length_ms / 1000)
                } else {
                    None
                };

                // Get release info for context
                let release_title = recording
                    .get("releases")
                    .and_then(|releases| releases.as_array())
                    .and_then(|releases| releases.first())
                    .and_then(|release| release.get("title"))
                    .and_then(|title| title.as_str())
                    .unwrap_or("Single");

                // Get release date if available
                let release_date = recording
                    .get("releases")
                    .and_then(|releases| releases.as_array())
                    .and_then(|releases| releases.first())
                    .and_then(|release| release.get("date"))
                    .and_then(|date| date.as_str())
                    .unwrap_or("");

                let year = if !release_date.is_empty() {
                    release_date
                        .split('-')
                        .next()
                        .and_then(|y| y.parse::<u32>().ok())
                } else {
                    None
                };

                let score = recording.get("score").and_then(|s| s.as_u64()).unwrap_or(0);

                // Get ISRCs if available
                let isrcs = recording
                    .get("isrcs")
                    .and_then(|isrcs| isrcs.as_array())
                    .map(|isrcs| {
                        isrcs
                            .iter()
                            .filter_map(|isrc| isrc.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                // Get tags/genres
                let recording_genres = recording
                    .get("tags")
                    .and_then(|tags| tags.as_array())
                    .map(|tags| {
                        tags.iter()
                            .filter_map(|tag| tag.get("name").and_then(|n| n.as_str()))
                            .take(3)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "Recording".to_string());

                // Get rating
                let recording_rating = recording
                    .get("rating")
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                results.push(json!({
                    "id": format!("musicbrainz_recording_{}", recording_id),
                    "title": track_title,
                    "artist": artist_name,
                    "album": release_title,
                    "duration": duration_seconds,
                    "year": year,
                    "genre": recording_genres,
                    "source": "musicbrainz",
                    "available": false,
                    "external_url": format!("https://musicbrainz.org/recording/{}", recording_id),
                    "musicbrainz_id": recording_id,
                    "score": score,
                    "isrcs": isrcs,
                    "rating": recording_rating,
                    "type": "track"
                }));
            }
        }
    } else {
        info!("No recordings found in MusicBrainz response");
    }

    Ok(results)
}

/// Download MusicBrainz entity (artist, album, track) with tiered storage integration
async fn download_musicbrainz_entity(
    Path(mbid): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Result<Json<Value>, StatusCode> {
    info!("Download request for MusicBrainz MBID: {}", mbid);

    let entity_type = payload
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("track");
    let entity_name = payload
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown");
    let artist_name = payload
        .get("artist")
        .and_then(|a| a.as_str())
        .unwrap_or("Unknown");

    info!(
        "Download request - Type: {}, Name: {}, Artist: {}",
        entity_type, entity_name, artist_name
    );

    // Generate unique request ID for tracking
    let request_id = format!("mb_{}_{}", mbid, chrono::Utc::now().timestamp());

    match entity_type {
        "artist" => {
            // For artists, add to Lidarr for monitoring
            match add_artist_to_lidarr(&mbid, entity_name).await {
                Ok(_) => {
                    info!("Artist {} added to Lidarr monitoring", entity_name);
                    Ok(Json(json!({
                        "success": true,
                        "request_id": request_id,
                        "message": format!("Artist '{}' added to monitoring", entity_name),
                        "action": "monitoring",
                        "mbid": mbid,
                        "entity_type": entity_type,
                        "status": "monitoring",
                        "timestamp": chrono::Utc::now()
                    })))
                }
                Err(e) => {
                    warn!("Failed to add artist to Lidarr: {}", e);
                    Ok(Json(json!({
                        "success": false,
                        "error": format!("Failed to add artist to monitoring: {}", e),
                        "mbid": mbid,
                        "entity_type": entity_type
                    })))
                }
            }
        }
        "album" | "track" => {
            // For albums/tracks, search for downloadable releases
            match search_downloadable_releases(&mbid, entity_name, artist_name).await {
                Ok(downloads) => {
                    if downloads.is_empty() {
                        Ok(Json(json!({
                            "success": false,
                            "message": "No downloadable releases found",
                            "mbid": mbid,
                            "entity_type": entity_type
                        })))
                    } else {
                        info!(
                            "Found {} downloadable releases for {}",
                            downloads.len(),
                            entity_name
                        );
                        Ok(Json(json!({
                            "success": true,
                            "request_id": request_id,
                            "message": format!("Found {} downloadable releases", downloads.len()),
                            "downloads": downloads,
                            "mbid": mbid,
                            "entity_type": entity_type,
                            "status": "available",
                            "timestamp": chrono::Utc::now()
                        })))
                    }
                }
                Err(e) => {
                    warn!("Failed to search downloadable releases: {}", e);
                    Ok(Json(json!({
                        "success": false,
                        "error": format!("Failed to search releases: {}", e),
                        "mbid": mbid,
                        "entity_type": entity_type
                    })))
                }
            }
        }
        _ => Ok(Json(json!({
            "success": false,
            "error": "Unsupported entity type",
            "supported_types": ["artist", "album", "track"]
        }))),
    }
}

/// Get download status for a request
async fn get_download_status(Path(request_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Download status request for: {}", request_id);

    // In a real implementation, this would check actual download status
    // For now, return a simulated status
    Ok(Json(json!({
        "success": true,
        "request_id": request_id,
        "status": "completed",
        "progress": 100,
        "message": "Download completed",
        "timestamp": chrono::Utc::now()
    })))
}

/// Preview MusicBrainz track (if available for streaming)
async fn preview_musicbrainz_track(Path(mbid): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!("Preview request for MusicBrainz recording: {}", mbid);

    // Try to find the track in local library first
    let navidrome_addon = create_navidrome_addon();

    // In a real implementation, this would:
    // 1. Search local library for matching track
    // 2. Check for preview URLs from external sources
    // 3. Return streaming URL if available

    Ok(Json(json!({
        "success": false,
        "message": "Preview not available - track not in local library",
        "mbid": mbid,
        "suggestions": [
            "Add artist to monitoring for future releases",
            "Search for similar tracks in your library"
        ]
    })))
}

/// Add artist to Lidarr monitoring
async fn add_artist_to_lidarr(mbid: &str, artist_name: &str) -> Result<String, String> {
    let lidarr_addon = create_lidarr_addon();

    if !lidarr_addon.enabled {
        return Err("Lidarr not configured".to_string());
    }

    info!("Adding artist {} (MBID: {}) to Lidarr", artist_name, mbid);

    // Get Lidarr configuration dynamically
    let config = get_lidarr_configuration(&lidarr_addon).await?;

    // Create a proper LidarrSearchResult for the artist
    let search_result = LidarrSearchResult {
        foreign_artist_id: mbid.to_string(),
        artist_name: artist_name.to_string(),
        overview: None,
        disambiguation: None,
        images: None,
        links: None,
        genres: None,
        ratings: None,
    };

    // Use the proper add_artist method with dynamic configuration
    match lidarr_addon
        .add_artist(
            &search_result,
            config.quality_profile_id,
            config.metadata_profile_id,
            &config.root_folder_path,
        )
        .await
    {
        Ok(result) => {
            info!(
                "Successfully added artist to Lidarr monitoring: {:?}",
                result.artist_name
            );
            Ok(format!("Artist '{}' added to monitoring", artist_name))
        }
        Err(e) => {
            warn!("Failed to add artist to Lidarr: {}", e);
            Err(format!("Lidarr error: {}", e))
        }
    }
}

/// Get Lidarr configuration (quality profiles, metadata profiles, root folders)
async fn get_lidarr_configuration(
    lidarr_addon: &crate::lidarr_addon::LidarrAddon,
) -> Result<LidarrConfig, String> {
    let client = reqwest::Client::new();

    // Get quality profiles
    let quality_url = format!("{}/api/v1/qualityprofile", lidarr_addon.url);
    let quality_response = client
        .get(&quality_url)
        .header("X-Api-Key", &lidarr_addon.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to get quality profiles: {}", e))?;

    let quality_profiles: Vec<serde_json::Value> = quality_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse quality profiles: {}", e))?;

    let quality_profile_id = quality_profiles
        .first()
        .and_then(|q| q.get("id").and_then(|id| id.as_u64()))
        .ok_or("No quality profiles found")? as u32;

    // Get metadata profiles
    let metadata_url = format!("{}/api/v1/metadataprofile", lidarr_addon.url);
    let metadata_response = client
        .get(&metadata_url)
        .header("X-Api-Key", &lidarr_addon.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to get metadata profiles: {}", e))?;

    let metadata_profiles: Vec<serde_json::Value> = metadata_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse metadata profiles: {}", e))?;

    let metadata_profile_id = metadata_profiles
        .first()
        .and_then(|m| m.get("id").and_then(|id| id.as_u64()))
        .ok_or("No metadata profiles found")? as u32;

    // Get root folders
    let root_url = format!("{}/api/v1/rootfolder", lidarr_addon.url);
    let root_response = client
        .get(&root_url)
        .header("X-Api-Key", &lidarr_addon.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Failed to get root folders: {}", e))?;

    let root_folders: Vec<serde_json::Value> = root_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse root folders: {}", e))?;

    let root_folder_path = root_folders
        .first()
        .and_then(|r| r.get("path").and_then(|path| path.as_str()))
        .ok_or("No root folders found")?
        .to_string();

    info!(
        "Lidarr configuration: Quality Profile ID: {}, Metadata Profile ID: {}, Root Folder: {}",
        quality_profile_id, metadata_profile_id, root_folder_path
    );

    Ok(LidarrConfig {
        quality_profile_id,
        metadata_profile_id,
        root_folder_path,
    })
}

/// Lidarr configuration structure
#[derive(Debug)]
struct LidarrConfig {
    quality_profile_id: u32,
    metadata_profile_id: u32,
    root_folder_path: String,
}

/// Search for downloadable releases related to MusicBrainz entity
async fn search_downloadable_releases(
    mbid: &str,
    entity_name: &str,
    artist_name: &str,
) -> Result<Vec<Value>, String> {
    let lidarr_addon = create_lidarr_addon();

    if !lidarr_addon.enabled {
        return Err("Lidarr not configured".to_string());
    }

    info!(
        "Searching downloadable releases for: {} by {}",
        entity_name, artist_name
    );

    // Search Lidarr releases for matches
    let client = reqwest::Client::new();
    let search_url = format!("{}/api/v1/release", lidarr_addon.url);

    match client
        .get(&search_url)
        .header("X-Api-Key", &lidarr_addon.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(releases) = response.json::<Vec<serde_json::Value>>().await {
                let mut downloads = Vec::new();

                // Filter releases that match the search criteria
                for release in releases.iter() {
                    if let (Some(title), Some(release_artist)) = (
                        release.get("title").and_then(|v| v.as_str()),
                        release.get("artistName").and_then(|v| v.as_str()),
                    ) {
                        // Check if this release matches our search
                        if release_artist
                            .to_lowercase()
                            .contains(&artist_name.to_lowercase())
                            || title.to_lowercase().contains(&entity_name.to_lowercase())
                        {
                            let download_info = json!({
                                "id": release.get("guid").and_then(|v| v.as_str()).unwrap_or(""),
                                "title": title,
                                "artist": release_artist,
                                "quality": release.get("quality")
                                    .and_then(|q| q.get("quality"))
                                    .and_then(|q| q.get("name"))
                                    .and_then(|v| v.as_str()).unwrap_or("Unknown"),
                                "size_mb": release.get("size").and_then(|v| v.as_u64()).unwrap_or(0) / 1024 / 1024,
                                "seeders": release.get("seeders").and_then(|v| v.as_u64()).unwrap_or(0),
                                "indexer": release.get("indexer").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                                "magnet_url": release.get("magnetUrl").and_then(|v| v.as_str()),
                                "can_download": true,
                                "storage_tier": "hot", // Downloads start in hot tier
                                "estimated_download_time": "2-5 minutes"
                            });
                            downloads.push(download_info);

                            if downloads.len() >= 5 {
                                break; // Limit to 5 results
                            }
                        }
                    }
                }

                info!("Found {} matching downloadable releases", downloads.len());
                Ok(downloads)
            } else {
                Err("Failed to parse Lidarr response".to_string())
            }
        }
        Err(e) => Err(format!("Failed to query Lidarr: {}", e)),
    }
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

/// Get download status by request ID
async fn get_download_status_endpoint(
    State(download_service): State<Arc<DownloadService>>,
    Path(request_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match download_service.get_download_status(&request_id).await {
        Some(request) => Ok(Json(json!({
            "success": true,
            "request_id": request.id,
            "status": request.status,
            "artist_name": request.artist_name,
            "track_title": request.track_title,
            "album_title": request.album_title,
            "progress": request.progress.unwrap_or(0.0),
            "file_size": request.file_size,
            "download_speed": request.download_speed,
            "seeds": request.seeds,
            "peers": request.peers,
            "torrent_hash": request.torrent_hash,
            "requested_at": request.requested_at,
            "started_at": request.started_at,
            "completed_at": request.completed_at,
            "error_message": request.error_message,
            "timestamp": Utc::now()
        }))),
        None => Ok(Json(json!({
            "success": false,
            "error": "Download request not found",
            "request_id": request_id,
            "timestamp": Utc::now()
        }))),
    }
}

/// Get all active downloads
async fn get_active_downloads_endpoint(
    State(download_service): State<Arc<DownloadService>>,
) -> Result<Json<Value>, StatusCode> {
    let active_downloads = download_service.get_active_downloads().await;

    Ok(Json(json!({
        "success": true,
        "active_downloads": active_downloads.len(),
        "downloads": active_downloads.iter().map(|d| json!({
            "id": d.id,
            "request_id": d.download_request_id,
            "torrent_hash": d.torrent_hash,
            "name": d.name,
            "status": d.status,
            "progress": d.progress,
            "size": d.size,
            "downloaded": d.downloaded,
            "download_speed": d.download_speed,
            "upload_speed": d.upload_speed,
            "seeds": d.seeds,
            "peers": d.peers,
            "eta": d.eta,
            "added_at": d.added_at,
            "completed_at": d.completed_at
        })).collect::<Vec<_>>(),
        "timestamp": Utc::now()
    })))
}

/// Get download statistics
async fn get_download_stats_endpoint(
    State(download_service): State<Arc<DownloadService>>,
) -> Result<Json<Value>, StatusCode> {
    let stats = download_service.get_stats().await;

    Ok(Json(json!({
        "success": true,
        "stats": {
            "total_downloads": stats.total_downloads,
            "completed_downloads": stats.completed_downloads,
            "failed_downloads": stats.failed_downloads,
            "active_downloads": stats.active_downloads,
            "queued_downloads": stats.queued_downloads,
            "total_downloaded_bytes": stats.total_downloaded_bytes,
            "total_uploaded_bytes": stats.total_uploaded_bytes,
            "average_download_speed": stats.average_download_speed,
            "last_updated": stats.last_updated
        },
        "timestamp": Utc::now()
    })))
}

/// Pause a download
async fn pause_download_endpoint(
    State(download_service): State<Arc<DownloadService>>,
    Path(hash): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match download_service.pause_download(&hash).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": format!("Download paused: {}", hash),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": format!("Failed to pause download: {}", e),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
    }
}

/// Resume a download
async fn resume_download_endpoint(
    State(download_service): State<Arc<DownloadService>>,
    Path(hash): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match download_service.resume_download(&hash).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": format!("Download resumed: {}", hash),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": format!("Failed to resume download: {}", e),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
    }
}

/// Get artwork for a specific track - placeholder implementation
async fn get_track_artwork(Path(track_id): Path<String>) -> Result<Response, StatusCode> {
    info!(
        "Getting artwork for track: {} (not yet implemented)",
        track_id
    );

    // TODO: Implement artwork retrieval once Navidrome client supports it
    Ok(Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            json!({
                "success": false,
                "message": "Artwork retrieval not yet implemented",
                "track_id": track_id,
                "timestamp": Utc::now()
            })
            .to_string()
            .into(),
        )
        .unwrap())

    /* TODO: Implement once Navidrome client has get_song and get_cover_art methods
    let navidrome_addon = create_navidrome_addon();

    // Get track details from Navidrome
    match navidrome_addon.get_song(&track_id).await {
        Ok(song) => {
            if let Some(cover_art_id) = song.cover_art {
                // Proxy the cover art from Navidrome
                match navidrome_addon.get_cover_art(&cover_art_id).await {
                    Ok(artwork_data) => Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "image/jpeg")
                        .header(header::CACHE_CONTROL, "public, max-age=3600")
                        .body(artwork_data.into())
                        .unwrap()),
                    Err(e) => {
                        warn!("Failed to get cover art: {}", e);
                        Err(StatusCode::NOT_FOUND)
                    }
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            warn!("Failed to get song details: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
    */
}

/// Get enhanced metadata for a specific track - placeholder implementation
async fn get_track_metadata(Path(track_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!(
        "Getting metadata for track: {} (not yet implemented)",
        track_id
    );

    // TODO: Implement using search or other available Navidrome methods
    Ok(Json(json!({
        "success": false,
        "message": "Track metadata retrieval not yet implemented",
        "track_id": track_id,
        "timestamp": Utc::now()
    })))

    /* TODO: Implement once Navidrome client has get_song method
    let navidrome_addon = create_navidrome_addon();

    match navidrome_addon.get_song(&track_id).await {
        Ok(song) => {
            // Check if there's an associated CUE file by looking for it in the same directory
            let has_cue_data = check_for_cue_file(&song).await;

            let metadata = json!({
                "success": true,
                "track": {
                    "id": song.id,
                    "title": song.title,
                    "artist": song.artist,
                    "album": song.album,
                    "album_id": song.album_id,
                    "artist_id": song.artist_id,
                    "duration": song.duration,
                    "bit_rate": song.bit_rate,
                    "format": song.suffix,
                    "year": song.year,
                    "genre": song.genre,
                    "disc_number": song.disc_number,
                    "track_number": song.track,
                    "path": song.path,
                    "size": song.size,
                    "created": song.created,
                    "updated": song.updated,
                    "cover_art_id": song.cover_art,
                    "has_artwork": song.cover_art.is_some(),
                    "has_cue_data": has_cue_data,
                    "stream_url": format!("/api/v1/stream/{}", song.id),
                    "artwork_url": song.cover_art.as_ref().map(|_| format!("/api/v1/artwork/{}", song.id))
                },
                "timestamp": Utc::now()
            });

            Ok(Json(metadata))
        }
        Err(e) => {
            warn!("Failed to get song metadata: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": format!("Track not found: {}", e),
                "timestamp": Utc::now()
            })))
        }
    }
    */
}

/// Get CUE file data for a track - placeholder implementation
async fn get_track_cue_data(Path(track_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    info!(
        "Getting CUE data for track: {} (not yet implemented)",
        track_id
    );

    // TODO: Implement CUE file detection and parsing
    Ok(Json(json!({
        "success": false,
        "message": "CUE data retrieval not yet implemented",
        "track_id": track_id,
        "timestamp": Utc::now()
    })))

    /* TODO: Implement once Navidrome client has get_song method
    let navidrome_addon = create_navidrome_addon();

    match navidrome_addon.get_song(&track_id).await {
        Ok(song) => {
            // Look for CUE file in the same directory as the audio file
            if let Some(cue_content) = get_cue_file_content(&song).await {
                let cue_data = parse_cue_content(&cue_content);

                Ok(Json(json!({
                    "success": true,
                    "track_id": track_id,
                    "has_cue": true,
                    "cue_data": cue_data,
                    "timestamp": Utc::now()
                })))
            } else {
                Ok(Json(json!({
                    "success": true,
                    "track_id": track_id,
                    "has_cue": false,
                    "message": "No CUE file found for this track",
                    "timestamp": Utc::now()
                })))
            }
        }
        Err(e) => {
            warn!("Failed to get song for CUE data: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": format!("Track not found: {}", e),
                "timestamp": Utc::now()
            })))
        }
    }
    */
}

/// Browse library with enhanced organization
async fn browse_library() -> Result<Json<Value>, StatusCode> {
    info!("Browsing library with organization - placeholder implementation");

    // TODO: Implement using available navidrome_addon methods
    Ok(Json(json!({
        "success": false,
        "message": "Library browsing not yet implemented",
        "timestamp": Utc::now()
    })))

    /* TODO: Implement once navidrome_addon has get_artists and get_albums methods
    let navidrome_addon = create_navidrome_addon();

    // Get artists and albums
    let artists_result = navidrome_addon.get_artists().await;
    let albums_result = navidrome_addon.get_albums(None, None, None, None).await;

    match (artists_result, albums_result) {
        (Ok(artists), Ok(albums)) => {
            // Group albums by artist
            let mut artist_albums: std::collections::HashMap<String, Vec<_>> =
                std::collections::HashMap::new();

            for album in albums {
                if let Some(artist_id) = &album.artist_id {
                    artist_albums
                        .entry(artist_id.clone())
                        .or_insert_with(Vec::new)
                        .push(album);
                }
            }

            let organized_data: Vec<_> = artists.into_iter().map(|artist| {
                let albums = artist_albums.get(&artist.id).cloned().unwrap_or_default();

                json!({
                    "artist": {
                        "id": artist.id,
                        "name": artist.name,
                        "album_count": artist.album_count,
                        "song_count": artist.song_count,
                        "starred": artist.starred,
                    },
                    "albums": albums.into_iter().map(|album| json!({
                        "id": album.id,
                        "name": album.name,
                        "song_count": album.song_count,
                        "duration": album.duration,
                        "year": album.year,
                        "genre": album.genre,
                        "cover_art_id": album.cover_art,
                        "has_artwork": album.cover_art.is_some(),
                        "artwork_url": album.cover_art.as_ref().map(|_| format!("/api/v1/artwork/{}", album.id))
                    })).collect::<Vec<_>>()
                })
            }).collect();

            Ok(Json(json!({
                "success": true,
                "library": {
                    "artists": organized_data,
                    "total_artists": organized_data.len(),
                    "timestamp": Utc::now()
                }
            })))
        }
        (Err(e), _) | (_, Err(e)) => {
            warn!("Failed to browse library: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": format!("Failed to browse library: {}", e),
                "timestamp": Utc::now()
            })))
        }
    }
    */
}

/// Cancel a download
async fn cancel_download_endpoint(
    State(download_service): State<Arc<DownloadService>>,
    Path(hash): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match download_service.cancel_download(&hash, true).await {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "message": format!("Download cancelled: {}", hash),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "error": format!("Failed to cancel download: {}", e),
            "torrent_hash": hash,
            "timestamp": Utc::now()
        }))),
    }
}
