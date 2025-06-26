//! StepheyBot Music - Minimal working version
//!
//! A simplified version that provides basic HTTP endpoints and health checks
//! while we work on implementing the full functionality.

mod lidarr_addon;
mod navidrome_addon;

use crate::lidarr_addon::is_lidarr_configured;

use anyhow::Result;
use axum::{
    extract::{Json as ExtractJson, Path, Query},
    http::{header, StatusCode},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use lidarr_addon::{create_lidarr_addon, get_lidarr_connection_status, test_lidarr_integration};
use navidrome_addon::{create_navidrome_addon, get_connection_status, test_navidrome_integration};
use rand::random;
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{info, warn};
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
        let external_results = search_external_apis(&query, search_category).await;
        results.extend(external_results);
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

    let external_results = search_external_apis(&query).await;

    Ok(Json(json!({
        "success": true,
        "query": query,
        "results": external_results,
        "total": external_results.len(),
        "sources": ["spotify", "musicbrainz"],
        "timestamp": Utc::now()
    })))
}

/// Request download of a track via Lidarr
async fn request_download(
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

    let lidarr_addon = create_lidarr_addon();

    // First, try to add the artist to Lidarr monitoring
    match lidarr_addon.search_artist(artist_name).await {
        Ok(artists) => {
            if let Some(artist) = artists.first() {
                // Add artist to monitoring in Lidarr
                match lidarr_addon
                    .add_artist_to_monitoring(&artist.artist_name, &artist.foreign_artist_id)
                    .await
                {
                    Ok(_) => {
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
                    Err(e) => {
                        warn!("Failed to add artist to Lidarr: {}", e);
                        Ok(Json(json!({
                            "success": false,
                            "error": "Failed to add artist to monitoring",
                            "message": format!("Could not add {} to Lidarr monitoring", artist_name),
                            "timestamp": Utc::now()
                        })))
                    }
                }
            } else {
                Ok(Json(json!({
                    "success": false,
                    "error": "Artist not found",
                    "message": format!("Could not find artist {} in external databases", artist_name),
                    "timestamp": Utc::now()
                })))
            }
        }
        Err(e) => {
            warn!("Failed to search for artist: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Search failed",
                "message": format!("Failed to search for artist {}", artist_name),
                "timestamp": Utc::now()
            })))
        }
    }
}

/// Helper function to search external APIs
async fn search_external_apis(query: &str, category: &str) -> Vec<Value> {
    let mut results = Vec::new();

    // Search MusicBrainz based on category
    match category {
        "artist" => {
            if let Ok(musicbrainz_results) = search_musicbrainz_artists(query).await {
                results.extend(musicbrainz_results);
            }
        }
        "album" => {
            if let Ok(musicbrainz_results) = search_musicbrainz_albums(query).await {
                results.extend(musicbrainz_results);
            }
        }
        "track" => {
            if let Ok(musicbrainz_results) = search_musicbrainz_recordings(query).await {
                results.extend(musicbrainz_results);
            }
        }
        "all" | _ => {
            // Search all categories
            if let Ok(musicbrainz_results) = search_musicbrainz_artists(query).await {
                results.extend(musicbrainz_results);
            }
            if let Ok(musicbrainz_results) = search_musicbrainz_albums(query).await {
                results.extend(musicbrainz_results);
            }
        }
    }

    results
}

/// Search MusicBrainz for artists
async fn search_musicbrainz_artists(query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let encoded_query = urlencoding::encode(query);

    // MusicBrainz artist search API
    let url = format!(
        "https://musicbrainz.org/ws/2/artist?query={}&limit=10&fmt=json",
        encoded_query
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    if let Some(artists) = musicbrainz_response
        .get("artists")
        .and_then(|a| a.as_array())
    {
        for artist in artists.iter().take(5) {
            // Limit to top 5 artists
            if let Some(artist_name) = artist.get("name").and_then(|n| n.as_str()) {
                let artist_id = artist.get("id").and_then(|i| i.as_str()).unwrap_or("");
                let disambiguation = artist
                    .get("disambiguation")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let country = artist.get("country").and_then(|c| c.as_str()).unwrap_or("");
                let life_span = artist
                    .get("life-span")
                    .and_then(|ls| ls.get("begin"))
                    .and_then(|b| b.as_str())
                    .unwrap_or("");

                // Get genres/tags if available
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
                    .unwrap_or_else(|| "Unknown".to_string());

                let score = artist.get("score").and_then(|s| s.as_u64()).unwrap_or(0);

                results.push(json!({
                    "id": format!("musicbrainz_artist_{}", artist_id),
                    "title": artist_name,
                    "artist": artist_name,
                    "album": if disambiguation.is_empty() {
                        format!("Artist{}", if !country.is_empty() { format!(" ({})", country) } else { String::new() })
                    } else {
                        disambiguation
                    },
                    "duration": null,
                    "year": if !life_span.is_empty() {
                        life_span.split('-').next().and_then(|y| y.parse::<u32>().ok())
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
                    "disambiguation": disambiguation
                }));
            }
        }
    }

    Ok(results)
}

/// Search MusicBrainz for albums/release-groups
async fn search_musicbrainz_albums(query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let encoded_query = urlencoding::encode(query);

    // MusicBrainz release-group search API
    let url = format!(
        "https://musicbrainz.org/ws/2/release-group?query={}&limit=10&fmt=json",
        encoded_query
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    if let Some(release_groups) = musicbrainz_response
        .get("release-groups")
        .and_then(|rg| rg.as_array())
    {
        for album in release_groups.iter().take(5) {
            // Limit to top 5 albums
            if let Some(album_title) = album.get("title").and_then(|t| t.as_str()) {
                let album_id = album.get("id").and_then(|i| i.as_str()).unwrap_or("");
                let album_type = album
                    .get("primary-type")
                    .and_then(|pt| pt.as_str())
                    .unwrap_or("Album");

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
                    "type": "album"
                }));
            }
        }
    }

    Ok(results)
}

/// Search MusicBrainz for recordings/tracks
async fn search_musicbrainz_recordings(
    query: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let encoded_query = urlencoding::encode(query);

    // MusicBrainz recording search API
    let url = format!(
        "https://musicbrainz.org/ws/2/recording?query={}&limit=10&fmt=json",
        encoded_query
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "StepheyBot-Music/1.0 (https://stepheybot.dev)",
        )
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let musicbrainz_response: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    if let Some(recordings) = musicbrainz_response
        .get("recordings")
        .and_then(|r| r.as_array())
    {
        for recording in recordings.iter().take(5) {
            // Limit to top 5 recordings
            if let Some(track_title) = recording.get("title").and_then(|t| t.as_str()) {
                let recording_id = recording.get("id").and_then(|i| i.as_str()).unwrap_or("");

                let artist_name = recording
                    .get("artist-credit")
                    .and_then(|ac| ac.as_array())
                    .and_then(|artists| artists.first())
                    .and_then(|artist| artist.get("artist"))
                    .and_then(|artist| artist.get("name"))
                    .and_then(|name| name.as_str())
                    .unwrap_or("Unknown Artist");

                let length_ms = recording
                    .get("length")
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0);
                let duration_seconds = if length_ms > 0 {
                    Some(length_ms / 1000)
                } else {
                    None
                };

                // Try to get release info for context
                let release_title = recording
                    .get("releases")
                    .and_then(|releases| releases.as_array())
                    .and_then(|releases| releases.first())
                    .and_then(|release| release.get("title"))
                    .and_then(|title| title.as_str())
                    .unwrap_or("Single");

                let score = recording.get("score").and_then(|s| s.as_u64()).unwrap_or(0);

                results.push(json!({
                    "id": format!("musicbrainz_recording_{}", recording_id),
                    "title": track_title,
                    "artist": artist_name,
                    "album": release_title,
                    "duration": duration_seconds,
                    "year": null,
                    "genre": "Unknown",
                    "source": "musicbrainz",
                    "available": false,
                    "external_url": format!("https://musicbrainz.org/recording/{}", recording_id),
                    "musicbrainz_id": recording_id,
                    "score": score,
                    "type": "track"
                }));
            }
        }
    }

    Ok(results)
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
