//! StepheyBot Music - Enhanced Main Application with Navidrome Integration
//!
//! This is the main entry point for the StepheyBot Music server with full
//! Navidrome integration, real-time data synchronization, and AI-powered
//! music recommendations.

use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, warn};

// Internal modules
mod clients;
mod config;
mod database;
mod models;
mod services;
mod utils;

use clients::{LidarrClient, ListenBrainzClient, MusicBrainzClient, NavidromeClient};
use config::Config;
use database::Database;
use services::{LibraryService, PlaylistService, RecommendationService, SyncService};

/// Enhanced application state with all services
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: Arc<Database>,
    pub navidrome: Arc<NavidromeClient>,
    pub listenbrainz: Option<Arc<ListenBrainzClient>>,
    pub lidarr: Option<Arc<LidarrClient>>,
    pub musicbrainz: Arc<MusicBrainzClient>,
    pub sync_service: Arc<SyncService>,
    pub recommendation_service: Arc<RecommendationService>,
    pub library_service: Arc<LibraryService>,
    pub playlist_service: Arc<PlaylistService>,
}

/// Query parameters for recommendations
#[derive(Debug, Deserialize)]
pub struct RecommendationQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    mood: Option<String>,
    genre: Option<String>,
    algorithm: Option<String>,
}

/// Request body for playlist generation
#[derive(Debug, Deserialize)]
pub struct PlaylistRequest {
    name: String,
    description: Option<String>,
    duration_minutes: Option<u32>,
    mood: Option<String>,
    genres: Option<Vec<String>>,
    include_favorites: Option<bool>,
}

/// Response for recommendation endpoints
#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    recommendations: Vec<TrackRecommendation>,
    total: usize,
    offset: usize,
    limit: usize,
    generated_at: DateTime<Utc>,
    algorithm: String,
    user_id: String,
}

/// Enhanced track recommendation with Navidrome data
#[derive(Debug, Serialize)]
pub struct TrackRecommendation {
    // Navidrome fields
    navidrome_id: String,
    title: String,
    artist: String,
    album: String,
    duration: u32,
    year: Option<u32>,
    genre: Option<String>,
    cover_art_url: Option<String>,

    // Recommendation fields
    score: f64,
    reason: String,
    recommendation_type: String,

    // Metadata
    play_count: u64,
    user_rating: Option<u32>,
    last_played: Option<DateTime<Utc>>,

    // Audio features (for content-based filtering)
    energy: Option<f64>,
    valence: Option<f64>,
    danceability: Option<f64>,
    tempo: Option<f64>,
}

/// Library statistics with real Navidrome data
#[derive(Debug, Serialize)]
pub struct LibraryStats {
    total_tracks: u32,
    total_albums: u32,
    total_artists: u32,
    total_users: u32,
    total_playlists: u32,
    total_listening_history: u64,
    library_size_gb: f64,
    last_sync: Option<DateTime<Utc>>,
    navidrome_status: String,
}

/// System information response
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    service: String,
    version: String,
    uptime_seconds: u64,
    navidrome_connected: bool,
    navidrome_url: String,
    database_status: String,
    active_users: u32,
    sync_status: String,
    last_sync: Option<DateTime<Utc>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stepheybot_music=info,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()?;

    info!(
        "🎵 Starting StepheyBot Music v{} - Navidrome Integration",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration
    let config = Arc::new(Config::load().context("Failed to load configuration")?);
    info!("✅ Configuration loaded");

    // Initialize database
    let database_url = config.database_url();

    // Create data directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&database_url.replace("sqlite:", "")).parent() {
        std::fs::create_dir_all(parent).context("Failed to create data directory")?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections as u32)
        .connect_timeout(Duration::from_secs(config.database.connection_timeout))
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let database = Arc::new(Database::new(pool));
    database
        .migrate()
        .await
        .context("Failed to run database migrations")?;
    info!("✅ Database initialized and migrated");

    // Initialize Navidrome client
    let navidrome = Arc::new(
        NavidromeClient::new(
            &config.navidrome_url(),
            &config.navidrome_username(),
            &config.navidrome_password(),
        )
        .context("Failed to create Navidrome client")?,
    );

    // Test Navidrome connection
    info!("🔍 Testing Navidrome connection...");
    match navidrome.health_check().await {
        Ok(_) => info!(
            "✅ Successfully connected to Navidrome at {}",
            config.navidrome_url()
        ),
        Err(e) => {
            error!("❌ Failed to connect to Navidrome: {}", e);
            warn!("⚠️  Continuing without Navidrome - some features will be limited");
        }
    }

    // Initialize optional services
    let listenbrainz = if !config.listenbrainz_token().is_empty() {
        match ListenBrainzClient::new(&config.listenbrainz_url(), &config.listenbrainz_token()) {
            Ok(client) => {
                info!("✅ ListenBrainz client initialized");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!("⚠️  Failed to initialize ListenBrainz client: {}", e);
                None
            }
        }
    } else {
        info!("ℹ️  ListenBrainz token not configured, skipping");
        None
    };

    let lidarr = if !config.lidarr_api_key().is_empty() {
        match LidarrClient::new(&config.lidarr_url(), &config.lidarr_api_key()) {
            Ok(client) => {
                info!("✅ Lidarr client initialized");
                Some(Arc::new(client))
            }
            Err(e) => {
                warn!("⚠️  Failed to initialize Lidarr client: {}", e);
                None
            }
        }
    } else {
        info!("ℹ️  Lidarr API key not configured, skipping");
        None
    };

    let musicbrainz = Arc::new(
        MusicBrainzClient::new(&config.musicbrainz_user_agent())
            .context("Failed to create MusicBrainz client")?,
    );
    info!("✅ MusicBrainz client initialized");

    // Initialize services
    let sync_service = Arc::new(
        SyncService::new(
            database.clone(),
            navidrome.clone(),
            listenbrainz
                .clone()
                .unwrap_or_else(|| Arc::new(ListenBrainzClient::new("", "").unwrap())),
        )
        .context("Failed to create sync service")?,
    );

    let recommendation_service = Arc::new(
        RecommendationService::new(database.clone(), config.clone())
            .context("Failed to create recommendation service")?,
    );

    let library_service = Arc::new(
        LibraryService::new(database.clone(), navidrome.clone())
            .context("Failed to create library service")?,
    );

    let playlist_service = Arc::new(
        PlaylistService::new(database.clone(), navidrome.clone())
            .context("Failed to create playlist service")?,
    );

    info!("✅ All services initialized");

    // Create application state
    let app_state = AppState {
        config: config.clone(),
        database: database.clone(),
        navidrome: navidrome.clone(),
        listenbrainz,
        lidarr,
        musicbrainz,
        sync_service: sync_service.clone(),
        recommendation_service,
        library_service,
        playlist_service,
    };

    // Perform initial sync if enabled
    if config.tasks.enable_background_tasks {
        info!("🔄 Performing initial sync with Navidrome...");
        let sync_state = app_state.clone();
        tokio::spawn(async move {
            match sync_state.sync_service.sync_all_users().await {
                Ok(result) => {
                    info!(
                        "✅ Initial sync completed: {}/{} users synced, {} listens",
                        result.successful_syncs,
                        result.total_users_processed,
                        result.total_listens_synced
                    );
                }
                Err(e) => {
                    error!("❌ Initial sync failed: {}", e);
                }
            }
        });
    }

    // Start background tasks
    if config.tasks.enable_background_tasks {
        start_background_tasks(app_state.clone()).await;
    }

    // Create router with all endpoints
    let app = create_router(app_state).await?;

    // Get server configuration
    let port = config.server_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 StepheyBot Music server starting on http://{}", addr);
    info!("📊 Health check: http://{}/health", addr);
    info!("🎵 API status: http://{}/api/v1/status", addr);
    info!(
        "🎧 Get recommendations: http://{}/api/v1/recommendations/user1",
        addr
    );
    info!("📱 Library stats: http://{}/api/v1/library/stats", addr);
    info!("🎶 Navidrome URL: {}", config.navidrome_url());

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 StepheyBot Music shutdown complete");
    Ok(())
}

/// Start background tasks for syncing and maintenance
async fn start_background_tasks(state: AppState) {
    let config = state.config.clone();

    // Sync task
    if config.tasks.sync_interval > 0 {
        let sync_state = state.clone();
        let interval = Duration::from_secs(config.tasks.sync_interval * 60);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                if let Err(e) = sync_state.sync_service.sync_all_users().await {
                    error!("Background sync failed: {}", e);
                }
            }
        });
        info!(
            "✅ Background sync task started (every {} minutes)",
            config.tasks.sync_interval
        );
    }

    // Recommendation update task
    if config.tasks.recommendation_interval > 0 {
        let rec_state = state.clone();
        let interval = Duration::from_secs(config.tasks.recommendation_interval * 60);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                // Update recommendations for all users
                info!("Updating recommendations for all users...");
                // Implementation would go here
            }
        });
        info!(
            "✅ Recommendation update task started (every {} minutes)",
            config.tasks.recommendation_interval
        );
    }
}

/// Create the application router with all endpoints
async fn create_router(state: AppState) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Ok(Router::new()
        // Health and status endpoints
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // API v1 endpoints
        .route("/api/v1/status", get(api_status))
        .route("/api/v1/system", get(system_info))
        // Music and recommendations
        .route(
            "/api/v1/recommendations/:user_id",
            get(get_user_recommendations),
        )
        .route("/api/v1/recommendations/trending", get(get_trending))
        .route("/api/v1/recommendations/discover", get(get_discovery))
        // Library management
        .route("/api/v1/library/stats", get(library_stats))
        .route("/api/v1/library/search", get(search_library))
        .route("/api/v1/library/sync", post(trigger_sync))
        // Playlist management
        .route("/api/v1/playlists", get(list_playlists))
        .route("/api/v1/playlists/generate", post(generate_playlist))
        // User management
        .route("/api/v1/users", get(list_users))
        .route("/api/v1/users/:user_id/history", get(get_user_history))
        // Admin endpoints (if enabled)
        .route("/admin/sync/status", get(sync_status))
        .route("/admin/sync/trigger", post(trigger_full_sync))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(cors),
        )
        .with_state(state))
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
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

    info!("🛑 Shutdown signal received, starting graceful shutdown...");
}

// =============================================================================
// API HANDLERS
// =============================================================================

/// Root endpoint with service information
async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "StepheyBot Music",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "AI-powered music recommendations with Navidrome integration",
        "status": "operational",
        "endpoints": {
            "health": "/health",
            "api_status": "/api/v1/status",
            "recommendations": "/api/v1/recommendations/{user_id}",
            "library_stats": "/api/v1/library/stats",
            "trending": "/api/v1/recommendations/trending"
        },
        "integration": {
            "navidrome": true,
            "ai_recommendations": true,
            "real_time_sync": true
        }
    }))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check database health
    if let Err(_) = state.database.health_check().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    // Check Navidrome health
    let navidrome_healthy = state.navidrome.health_check().await.is_ok();

    Ok(Json(serde_json::json!({
        "service": "stepheybot-music",
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "components": {
            "database": "healthy",
            "navidrome": if navidrome_healthy { "healthy" } else { "degraded" }
        }
    })))
}

/// Readiness check
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ready = state.database.health_check().await.is_ok();
    let navidrome_ready = state.navidrome.health_check().await.is_ok();

    if db_ready && navidrome_ready {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "timestamp": Utc::now()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Liveness check
async fn liveness_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now()
    }))
}

/// API status with detailed information
async fn api_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let navidrome_status = match state.navidrome.health_check().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(serde_json::json!({
        "api_version": "v1",
        "service": "StepheyBot Music",
        "version": env!("CARGO_PKG_VERSION"),
        "navidrome": {
            "status": navidrome_status,
            "url": state.config.navidrome_url()
        },
        "features": [
            "navidrome_integration",
            "ai_recommendations",
            "real_time_sync",
            "playlist_generation",
            "music_discovery"
        ],
        "timestamp": Utc::now()
    }))
}

/// Get system information
async fn system_info(State(state): State<AppState>) -> Json<SystemInfo> {
    let navidrome_connected = state.navidrome.health_check().await.is_ok();
    let database_status = if state.database.health_check().await.is_ok() {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    Json(SystemInfo {
        service: "StepheyBot Music".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Would need to track actual uptime
        navidrome_connected,
        navidrome_url: state.config.navidrome_url(),
        database_status,
        active_users: 0, // Would query from database
        sync_status: "operational".to_string(),
        last_sync: None, // Would query from sync service
    })
}

/// Get personalized recommendations for a user
async fn get_user_recommendations(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<RecommendationQuery>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    match state
        .recommendation_service
        .get_recommendations(&user_id, limit, offset)
        .await
    {
        Ok(recommendations) => Ok(Json(RecommendationResponse {
            recommendations,
            total: recommendations.len(),
            offset,
            limit,
            generated_at: Utc::now(),
            algorithm: "hybrid".to_string(),
            user_id,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get trending tracks
async fn get_trending(
    State(state): State<AppState>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    match state.recommendation_service.get_trending(20).await {
        Ok(recommendations) => Ok(Json(RecommendationResponse {
            recommendations,
            total: recommendations.len(),
            offset: 0,
            limit: 20,
            generated_at: Utc::now(),
            algorithm: "trending".to_string(),
            user_id: "global".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get discovery tracks
async fn get_discovery(
    State(state): State<AppState>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    match state.recommendation_service.get_discovery(20).await {
        Ok(recommendations) => Ok(Json(RecommendationResponse {
            recommendations,
            total: recommendations.len(),
            offset: 0,
            limit: 20,
            generated_at: Utc::now(),
            algorithm: "discovery".to_string(),
            user_id: "global".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get library statistics
async fn library_stats(State(state): State<AppState>) -> Result<Json<LibraryStats>, StatusCode> {
    match state.library_service.get_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Search music library
async fn search_library(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    match state.library_service.search(&query, limit).await {
        Ok(results) => Ok(Json(serde_json::json!({
            "query": query,
            "results": results,
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Trigger library sync
async fn trigger_sync(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sync_service.sync_all_users().await {
        Ok(result) => Ok(Json(serde_json::json!({
            "message": "Sync completed",
            "result": result,
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Generate smart playlist
async fn generate_playlist(
    State(state): State<AppState>,
    Json(request): Json<PlaylistRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .playlist_service
        .generate(&request.name, request.duration_minutes.unwrap_or(60))
        .await
    {
        Ok(playlist) => Ok(Json(serde_json::json!({
            "message": "Playlist generated successfully",
            "playlist": playlist,
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// List all playlists
async fn list_playlists(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.navidrome.get_playlists().await {
        Ok(playlists) => Ok(Json(serde_json::json!({
            "playlists": playlists,
            "count": playlists.len(),
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// List all users
async fn list_users(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.navidrome.get_users().await {
        Ok(users) => Ok(Json(serde_json::json!({
            "users": users,
            "count": users.len(),
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get user listening history
async fn get_user_history(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.library_service.get_user_history(&user_id).await {
        Ok(history) => Ok(Json(serde_json::json!({
            "user_id": user_id,
            "history": history,
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get sync status (admin endpoint)
async fn sync_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sync_service.get_sync_statistics().await {
        Ok(stats) => Ok(Json(serde_json::json!({
            "sync_status": stats,
            "timestamp": Utc::now()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Trigger full sync (admin endpoint)
async fn trigger_full_sync(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tokio::spawn(async move {
        if let Err(e) = state.sync_service.sync_all_users().await {
            error!("Full sync failed: {}", e);
        }
    });

    Ok(Json(serde_json::json!({
        "message": "Full sync initiated",
        "timestamp": Utc::now()
    })))
}
