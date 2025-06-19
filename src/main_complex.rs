//! StepheyBot Music - Private Spotify-like music streaming service
//!
//! This application provides music streaming, recommendations, and library management
//! for personal use, integrating with Navidrome, MusicBrainz, and other services.

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Timelike;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    signal,
    sync::broadcast,
    time::{interval, sleep},
};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod clients;
mod config;
mod database;
mod models;
mod services;
mod utils;

use clients::{
    lidarr::LidarrClient, listenbrainz::ListenBrainzClient, musicbrainz::MusicBrainzClient,
    navidrome::NavidromeClient,
};
use config::Config;
use database::Database;
use services::{
    library::LibraryService, playlist::PlaylistService, recommendation::RecommendationService,
    sync::SyncService,
};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: Arc<Database>,
    pub navidrome_client: Arc<NavidromeClient>,
    pub listenbrainz_client: Arc<ListenBrainzClient>,
    pub lidarr_client: Arc<LidarrClient>,
    pub musicbrainz_client: Arc<MusicBrainzClient>,
    pub library_service: Arc<LibraryService>,
    pub playlist_service: Arc<PlaylistService>,
    pub recommendation_service: Arc<RecommendationService>,
    pub sync_service: Arc<SyncService>,
    pub shutdown_tx: broadcast::Sender<()>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing()?;

    info!(
        "🎵 Starting StepheyBot Music v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration
    let config = Arc::new(Config::load().context("Failed to load configuration")?);
    info!("✅ Configuration loaded successfully");

    // Initialize database
    let database = Arc::new(
        Database::new(&config.database_url)
            .await
            .context("Failed to initialize database")?,
    );
    info!("✅ Database initialized successfully");

    // Run database migrations
    database
        .migrate()
        .await
        .context("Failed to run database migrations")?;
    info!("✅ Database migrations completed");

    // Initialize external service clients
    let navidrome_client = Arc::new(NavidromeClient::new(
        &config.navidrome_url,
        &config.navidrome_admin_user,
        &config.navidrome_admin_password,
    )?);

    let listenbrainz_client = Arc::new(ListenBrainzClient::new(
        &config.listenbrainz_url,
        config.listenbrainz_token.as_deref(),
    )?);

    let lidarr_client = Arc::new(LidarrClient::new(
        &config.lidarr_url,
        &config.lidarr_api_key,
    )?);

    let musicbrainz_client = Arc::new(MusicBrainzClient::new(&config.musicbrainz_user_agent)?);

    info!("✅ External service clients initialized");

    // Initialize core services
    let library_service = Arc::new(LibraryService::new(
        database.clone(),
        &config.music_path,
        &config.download_path,
    )?);

    let playlist_service = Arc::new(PlaylistService::new(
        database.clone(),
        navidrome_client.clone(),
    )?);

    let recommendation_service = Arc::new(RecommendationService::new(
        database.clone(),
        listenbrainz_client.clone(),
        musicbrainz_client.clone(),
        &config.cache_dir,
    )?);

    let sync_service = Arc::new(SyncService::new(
        database.clone(),
        navidrome_client.clone(),
        listenbrainz_client.clone(),
    )?);

    info!("✅ Core services initialized");

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel(1);

    // Build application state
    let app_state = AppState {
        config: config.clone(),
        database,
        navidrome_client,
        listenbrainz_client,
        lidarr_client,
        musicbrainz_client,
        library_service,
        playlist_service,
        recommendation_service,
        sync_service,
        shutdown_tx: shutdown_tx.clone(),
    };

    // Build HTTP router
    let app = create_router(app_state.clone());

    // Start background tasks
    let background_handle = tokio::spawn(run_background_tasks(
        app_state.clone(),
        shutdown_tx.subscribe(),
    ));

    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("🚀 Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|e| error!("Server error: {}", e));
    });

    info!("🎵 StepheyBot Music is ready!");

    // Wait for shutdown signal
    shutdown_signal().await;
    info!("🛑 Shutdown signal received, gracefully shutting down...");

    // Send shutdown signal to background tasks
    let _ = shutdown_tx.send(());

    // Wait for background tasks to complete
    if let Err(e) = background_handle.await {
        warn!("Background task join error: {}", e);
    }

    // Wait for server to shutdown
    if let Err(e) = server_handle.await {
        warn!("Server shutdown error: {}", e);
    }

    info!("👋 StepheyBot Music shutdown complete");
    Ok(())
}

/// Initialize tracing/logging system
fn init_tracing() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stepheybot_music=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()
        .context("Failed to initialize tracing")?;

    Ok(())
}

/// Create the main HTTP router
fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        // API routes
        .route("/api/v1/sync", post(trigger_sync))
        .route("/api/v1/recommendations/:user_id", get(get_recommendations))
        .route("/api/v1/playlists/generate", post(generate_playlist))
        .route("/api/v1/library/scan", post(scan_library))
        .route("/api/v1/stats", get(get_stats))
        // Admin routes (will be protected by OAuth2 proxy)
        .route("/admin/users", get(list_users))
        .route("/admin/system", get(system_info))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

/// Background task runner
async fn run_background_tasks(
    state: AppState,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    info!("🔄 Starting background tasks");

    // Sync task - runs every hour
    let sync_state = state.clone();
    let sync_task = tokio::spawn(async move {
        let mut sync_interval = interval(Duration::from_secs(3600)); // 1 hour
        loop {
            tokio::select! {
                _ = sync_interval.tick() => {
                    if let Err(e) = sync_state.sync_service.sync_all_users().await {
                        error!("Sync task error: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("🔄 Sync task shutting down");
                    break;
                }
            }
        }
    });

    // Recommendation generation task - runs daily at 3 AM
    let rec_state = state.clone();
    let mut rec_shutdown_rx = state.shutdown_tx.subscribe();
    let recommendation_task = tokio::spawn(async move {
        loop {
            // Calculate seconds until 3 AM
            let now = chrono::Local::now();
            let next_3am = if now.hour() < 3 {
                now.date_naive().and_hms_opt(3, 0, 0).unwrap()
            } else {
                (now.date_naive() + chrono::Duration::days(1))
                    .and_hms_opt(3, 0, 0)
                    .unwrap()
            };
            let duration_until_3am = (next_3am - now.naive_local())
                .to_std()
                .unwrap_or(Duration::from_secs(3600));

            tokio::select! {
                _ = sleep(duration_until_3am) => {
                    if let Err(e) = rec_state.recommendation_service.generate_all_recommendations().await {
                        error!("Recommendation generation error: {}", e);
                    }
                }
                _ = rec_shutdown_rx.recv() => {
                    info!("🎯 Recommendation task shutting down");
                    break;
                }
            }
        }
    });

    // Wait for all tasks to complete
    let _ = tokio::try_join!(sync_task, recommendation_task);

    info!("🔄 Background tasks completed");
    Ok(())
}

/// Wait for shutdown signal
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
}

// HTTP Handlers

/// Basic health check
async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "stepheybot-music",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check - ensures all dependencies are available
async fn readiness_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Check database connection
    if let Err(_) = state.database.health_check().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    // Check Navidrome connection
    if let Err(_) = state.navidrome_client.health_check().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(Json(json!({
        "status": "ready",
        "checks": {
            "database": "ok",
            "navidrome": "ok"
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

/// Trigger manual sync
async fn trigger_sync(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    match state.sync_service.sync_all_users().await {
        Ok(_) => Ok(Json(json!({"status": "sync_triggered"}))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get recommendations for a user
async fn get_recommendations(
    State(_state): State<AppState>,
    axum::extract::Path(_user_id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement recommendations endpoint
    Ok(Json(json!({
        "recommendations": [],
        "message": "Not implemented yet"
    })))
}

/// Generate playlist
async fn generate_playlist(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement playlist generation
    Ok(Json(json!({
        "playlist_id": "todo",
        "message": "Not implemented yet"
    })))
}

/// Scan music library
async fn scan_library(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement library scan
    Ok(Json(json!({
        "status": "scan_triggered",
        "message": "Not implemented yet"
    })))
}

/// Get system statistics
async fn get_stats(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement stats endpoint
    Ok(Json(json!({
        "stats": {},
        "message": "Not implemented yet"
    })))
}

/// List all users (admin endpoint)
async fn list_users(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement user listing
    Ok(Json(json!({
        "users": [],
        "message": "Not implemented yet"
    })))
}

/// Get system information (admin endpoint)
async fn system_info(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement system info
    Ok(Json(json!({
        "system": {},
        "message": "Not implemented yet"
    })))
}
