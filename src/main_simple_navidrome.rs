//! StepheyBot Music - Simple Navidrome Integration
//!
//! This version adds basic Navidrome connectivity to the existing application
//! while maintaining full backward compatibility with sample data.

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
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{debug, error, info, warn};

// Internal modules
mod clients;
mod config;
mod database;
mod models;
mod services;
mod utils;

use clients::NavidromeClient;

/// Enhanced application state with optional Navidrome integration
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub navidrome: Option<Arc<NavidromeClient>>,
    pub navidrome_connected: Arc<tokio::sync::RwLock<bool>>,
}

/// Query parameters for recommendations
#[derive(Debug, Deserialize)]
pub struct RecommendationQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    mood: Option<String>,
    genre: Option<String>,
}

/// Request body for playlist generation
#[derive(Debug, Deserialize)]
pub struct PlaylistRequest {
    name: String,
    description: Option<String>,
    duration_minutes: Option<u32>,
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
    source: String, // "navidrome" or "sample_data"
}

/// Track recommendation with optional Navidrome data
#[derive(Debug, Serialize)]
pub struct TrackRecommendation {
    track_id: String,
    title: String,
    artist: String,
    album: String,
    score: f64,
    reason: String,
    recommendation_type: String,
    duration: u32,
    year: Option<u32>,
    genre: Option<String>,
    play_count: u64,
    user_rating: Option<u32>,
    // Audio features
    energy: Option<f64>,
    valence: Option<f64>,
    danceability: Option<f64>,
}

/// Library statistics
#[derive(Debug, Serialize)]
pub struct LibraryStats {
    total_tracks: u32,
    total_albums: u32,
    total_artists: u32,
    total_users: u32,
    total_listening_history: u64,
    database_size_mb: f64,
    last_updated: DateTime<Utc>,
    source: String,
    navidrome_status: Option<String>,
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
        "🎵 Starting StepheyBot Music v{} - Simple Navidrome Integration",
        env!("CARGO_PKG_VERSION")
    );

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/stepheybot-music.db".to_string());

    // Create data directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(&database_url.replace("sqlite:", "")).parent() {
        std::fs::create_dir_all(parent).context("Failed to create data directory")?;
    }

    // Create database if it doesn't exist
    if !Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        info!("Creating database: {}", database_url);
        Sqlite::create_database(&database_url).await?;
    }

    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("Failed to run database migrations")?;

    info!("✅ Database initialized at {}", database_url);

    // Initialize Navidrome client (optional)
    let navidrome_url = std::env::var("NAVIDROME_URL").unwrap_or_default();
    let navidrome_username = std::env::var("NAVIDROME_USERNAME").unwrap_or_default();
    let navidrome_password = std::env::var("NAVIDROME_PASSWORD").unwrap_or_default();

    let (navidrome_client, navidrome_connected) = if !navidrome_url.is_empty()
        && !navidrome_username.is_empty()
        && !navidrome_password.is_empty()
    {
        info!("🔗 Initializing Navidrome client...");
        match NavidromeClient::new(&navidrome_url, &navidrome_username, &navidrome_password) {
            Ok(client) => {
                info!("🔍 Testing Navidrome connection...");
                let connected = match client.health_check().await {
                    Ok(_) => {
                        info!(
                            "✅ Successfully connected to Navidrome at {}",
                            navidrome_url
                        );
                        true
                    }
                    Err(e) => {
                        error!("❌ Failed to connect to Navidrome: {}", e);
                        false
                    }
                };
                (
                    Some(Arc::new(client)),
                    Arc::new(tokio::sync::RwLock::new(connected)),
                )
            }
            Err(e) => {
                error!("❌ Failed to create Navidrome client: {}", e);
                (None, Arc::new(tokio::sync::RwLock::new(false)))
            }
        }
    } else {
        info!("ℹ️  Navidrome not configured - using sample data only");
        (None, Arc::new(tokio::sync::RwLock::new(false)))
    };

    // Populate sample data
    let populate_sample = std::env::var("POPULATE_SAMPLE_DATA")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if populate_sample {
        info!("📝 Populating sample data...");
        populate_sample_data(&db)
            .await
            .context("Failed to populate sample data")?;
        info!("✅ Sample data populated");
    }

    // Create application state
    let app_state = AppState {
        db,
        navidrome: navidrome_client,
        navidrome_connected,
    };

    // Create router with all endpoints
    let app = create_router(app_state).await?;

    // Get server configuration
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8083".to_string())
        .parse::<u16>()
        .unwrap_or(8083);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 StepheyBot Music server starting on http://{}", addr);
    info!("📊 Health check: http://{}/health", addr);
    info!("🎵 API status: http://{}/api/v1/status", addr);
    info!(
        "🎧 Get recommendations: http://{}/api/v1/recommendations/user1",
        addr
    );
    info!("📱 Library stats: http://{}/api/v1/library/stats", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 StepheyBot Music shutdown complete");
    Ok(())
}

/// Populate database with sample data
async fn populate_sample_data(db: &SqlitePool) -> Result<()> {
    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS artists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            bio TEXT,
            genre TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS albums (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            year INTEGER,
            track_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (artist_id) REFERENCES artists (id)
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tracks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            album_id TEXT NOT NULL,
            duration INTEGER NOT NULL,
            track_number INTEGER,
            year INTEGER,
            genre TEXT,
            energy REAL,
            valence REAL,
            danceability REAL,
            play_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (artist_id) REFERENCES artists (id),
            FOREIGN KEY (album_id) REFERENCES albums (id)
        )
        "#,
    )
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(db)
    .await?;

    // Check if we already have data
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM artists")
        .fetch_one(db)
        .await?;

    if count.0 > 0 {
        debug!("Sample data already exists, skipping population");
        return Ok(());
    }

    // Sample artists
    let artists = [
        (
            "artist1",
            "Neon Synthetics",
            "Pioneering synthwave artist",
            "Synthwave",
        ),
        (
            "artist2",
            "Digital Dreams",
            "Electronic music producer",
            "Electronic",
        ),
        ("artist3", "Cyber Pulse", "High-energy synthpop", "Synthpop"),
        (
            "artist4",
            "Midnight Circuit",
            "Dark ambient electronic",
            "Dark Ambient",
        ),
        ("artist5", "Neon City", "Upbeat synthwave", "Synthwave"),
    ];

    for (id, name, bio, genre) in artists {
        sqlx::query("INSERT OR IGNORE INTO artists (id, name, bio, genre) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(bio)
            .bind(genre)
            .execute(db)
            .await?;
    }

    // Sample albums
    let albums = [
        ("album1", "Neon Nights", "artist1", 2023, 8),
        ("album2", "Digital Horizons", "artist2", 2022, 10),
        ("album3", "Retro Future", "artist3", 2023, 9),
        ("album4", "Dark Circuits", "artist4", 2021, 7),
        ("album5", "City Lights", "artist5", 2023, 11),
    ];

    for (id, title, artist_id, year, track_count) in albums {
        sqlx::query("INSERT OR IGNORE INTO albums (id, title, artist_id, year, track_count) VALUES (?, ?, ?, ?, ?)")
            .bind(id)
            .bind(title)
            .bind(artist_id)
            .bind(year)
            .bind(track_count)
            .execute(db)
            .await?;
    }

    // Sample tracks
    let tracks = [
        (
            "track1",
            "Midnight Drive",
            "artist1",
            "album1",
            245,
            1,
            2023,
            "Synthwave",
            0.8,
            0.7,
            0.6,
        ),
        (
            "track2",
            "Neon Glow",
            "artist1",
            "album1",
            198,
            2,
            2023,
            "Synthwave",
            0.9,
            0.8,
            0.7,
        ),
        (
            "track3",
            "Digital Rain",
            "artist2",
            "album2",
            312,
            1,
            2022,
            "Electronic",
            0.6,
            0.5,
            0.4,
        ),
        (
            "track4",
            "Virtual Dreams",
            "artist2",
            "album2",
            278,
            2,
            2022,
            "Electronic",
            0.5,
            0.6,
            0.3,
        ),
        (
            "track5",
            "Cyber Love",
            "artist3",
            "album3",
            203,
            1,
            2023,
            "Synthpop",
            0.9,
            0.9,
            0.8,
        ),
        (
            "track6",
            "Electric Pulse",
            "artist3",
            "album3",
            189,
            2,
            2023,
            "Synthpop",
            0.95,
            0.85,
            0.9,
        ),
        (
            "track7",
            "Dark Protocol",
            "artist4",
            "album4",
            356,
            1,
            2021,
            "Dark Ambient",
            0.3,
            0.2,
            0.2,
        ),
        (
            "track8",
            "System Override",
            "artist4",
            "album4",
            289,
            2,
            2021,
            "Dark Ambient",
            0.4,
            0.3,
            0.3,
        ),
        (
            "track9",
            "Neon Skyline",
            "artist5",
            "album5",
            234,
            1,
            2023,
            "Synthwave",
            0.85,
            0.75,
            0.65,
        ),
        (
            "track10",
            "City Pulse",
            "artist5",
            "album5",
            212,
            2,
            2023,
            "Synthwave",
            0.9,
            0.8,
            0.75,
        ),
    ];

    for (
        id,
        title,
        artist_id,
        album_id,
        duration,
        track_number,
        year,
        genre,
        energy,
        valence,
        danceability,
    ) in tracks
    {
        sqlx::query(
            "INSERT OR IGNORE INTO tracks (id, title, artist_id, album_id, duration, track_number, year, genre, energy, valence, danceability, play_count) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(title)
        .bind(artist_id)
        .bind(album_id)
        .bind(duration)
        .bind(track_number)
        .bind(year)
        .bind(genre)
        .bind(energy)
        .bind(valence)
        .bind(danceability)
        .bind(rand::random::<u32>() % 100)
        .execute(db)
        .await?;
    }

    // Sample users
    let users = [
        ("user1", "stephey", "stephey@stepheybot.dev"),
        ("user2", "musiclover", "music@example.com"),
        ("user3", "synthfan", "synth@example.com"),
    ];

    for (id, username, email) in users {
        sqlx::query("INSERT OR IGNORE INTO users (id, username, email) VALUES (?, ?, ?)")
            .bind(id)
            .bind(username)
            .bind(email)
            .execute(db)
            .await?;
    }

    Ok(())
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
        // Playlist management
        .route("/api/v1/playlists/generate", post(generate_playlist))
        // User management
        .route("/api/v1/users", get(list_users))
        .route("/api/v1/users/:user_id/history", get(get_user_history))
        // Navidrome specific endpoints
        .route("/api/v1/navidrome/status", get(navidrome_status))
        .route("/api/v1/navidrome/test", get(navidrome_test))
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
async fn root(State(state): State<AppState>) -> Json<serde_json::Value> {
    let navidrome_enabled = state.navidrome.is_some();
    let navidrome_connected = *state.navidrome_connected.read().await;

    Json(serde_json::json!({
        "service": "StepheyBot Music",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "AI-powered music recommendations with optional Navidrome integration",
        "status": "operational",
        "integration": {
            "navidrome_enabled": navidrome_enabled,
            "navidrome_connected": navidrome_connected,
            "ai_recommendations": true,
            "sample_data": true
        },
        "endpoints": {
            "health": "/health",
            "api_status": "/api/v1/status",
            "recommendations": "/api/v1/recommendations/{user_id}",
            "library_stats": "/api/v1/library/stats",
            "navidrome_status": "/api/v1/navidrome/status"
        }
    }))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check database health
    let db_healthy = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    if !db_healthy {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let navidrome_status = if let Some(client) = &state.navidrome {
        match client.health_check().await {
            Ok(_) => "connected",
            Err(_) => "disconnected",
        }
    } else {
        "disabled"
    };

    Ok(Json(serde_json::json!({
        "service": "stepheybot-music",
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "components": {
            "database": if db_healthy { "healthy" } else { "unhealthy" },
            "navidrome": navidrome_status
        }
    })))
}

/// Readiness check
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ready = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    if db_ready {
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
    let navidrome_connected = *state.navidrome_connected.read().await;

    Json(serde_json::json!({
        "api_version": "v1",
        "service": "StepheyBot Music",
        "version": env!("CARGO_PKG_VERSION"),
        "navidrome": {
            "enabled": state.navidrome.is_some(),
            "connected": navidrome_connected
        },
        "features": [
            "health_checks",
            "basic_routing",
            "recommendations",
            "library_management",
            "playlist_generation",
            if state.navidrome.is_some() { "navidrome_integration" } else { "sample_data_mode" }
        ],
        "timestamp": Utc::now()
    }))
}

/// Get system information
async fn system_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    let navidrome_connected = *state.navidrome_connected.read().await;

    Json(serde_json::json!({
        "service": "StepheyBot Music",
        "version": env!("CARGO_PKG_VERSION"),
        "navidrome": {
            "enabled": state.navidrome.is_some(),
            "connected": navidrome_connected
        },
        "database": {
            "type": "SQLite",
            "status": "healthy"
        },
        "timestamp": Utc::now()
    }))
}

/// Get personalized recommendations for a user
async fn get_user_recommendations(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<RecommendationQuery>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(10).min(50);
    let offset = params.offset.unwrap_or(0);

    // For now, always use sample data but indicate if Navidrome is available
    match get_sample_recommendations(&state.db, &user_id, limit, offset).await {
        Ok(recommendations) => {
            let source = if state.navidrome.is_some() && *state.navidrome_connected.read().await {
                "sample_with_navidrome_available"
            } else {
                "sample_data"
            };

            Ok(Json(RecommendationResponse {
                recommendations,
                total: recommendations.len(),
                offset,
                limit,
                generated_at: Utc::now(),
                algorithm: "sample_based".to_string(),
                source: source.to_string(),
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get recommendations from sample data
async fn get_sample_recommendations(
    db: &SqlitePool,
    _user_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<TrackRecommendation>> {
    let tracks = sqlx::query!(
        "SELECT t.id, t.title, a.name as artist, al.title as album, t.duration, t.year, t.genre,
                t.energy, t.valence, t.danceability, t.play_count
         FROM tracks t
         JOIN artists a ON t.artist_id = a.id
         JOIN albums al ON t.album_id = al.id
         ORDER BY t.energy DESC, t.valence DESC
         LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    let mut recommendations = Vec::new();
    for (i, track) in tracks.into_iter().enumerate() {
        recommendations.push(TrackRecommendation {
            track_id: track.id,
            title: track.title,
            artist: track.artist,
            album: track.album,
            duration: track.duration as u32,
            year: track.year.map(|y| y as u32),
            genre: track.genre,
            score: 0.95 - (i as f64 * 0.05),
            reason: format!(
                "High energy track with {:.1}% energy rating",
                track.energy.unwrap_or(0.0) * 100.0
            ),
            recommendation_type: "content_based".to_string(),
            play_count: track.play_count.unwrap_or(0) as u64,
            user_rating: None,
            energy: track.energy,
            valence: track.valence,
            danceability: track.danceability,
        });
    }

    Ok(recommendations)
}

/// Get trending tracks
async fn get_trending(
    State(state): State<AppState>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    match get_sample_recommendations(&state.db, "trending", 20, 0).await {
        Ok(recommendations) => Ok(Json(RecommendationResponse {
            recommendations,
            total: recommendations.len(),
            offset: 0,
            limit: 20,
            generated_at: Utc::now(),
            algorithm: "sample_trending".to_string(),
            source: "sample_data".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get discovery tracks
async fn get_discovery(
    State(state): State<AppState>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    match sqlx::query!(
        "SELECT t.id, t.title, a.name as artist, al.title as album, t.duration, t.year, t.genre,
                t.energy, t.valence, t.danceability, t.play_count
         FROM tracks t
         JOIN artists a ON t.artist_id = a.id
         JOIN albums al ON t.album_id = al.id
         WHERE t.play_count < 50
         ORDER BY t.valence DESC
         LIMIT 20"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(tracks) => {
            let recommendations = tracks
                .into_iter()
                .enumerate()
                .map(|(i, track)| TrackRecommendation {
                    track_id: track.id,
                    title: track.title,
                    artist: track.artist,
                    album: track.album,
                    duration: track.duration as u32,
                    year: track.year.map(|y| y as u32),
                    genre: track.genre,
                    score: 0.8 - (i as f64 * 0.02),
                    reason: "Underplayed gem with high mood rating".to_string(),
                    recommendation_type: "discovery".to_string(),
                    play_count: track.play_count.unwrap_or(0) as u64,
                    user_rating: None,
                    energy: track.energy,
                    valence: track.valence,
                    danceability: track.danceability,
                })
                .collect();

            Ok(Json(RecommendationResponse {
                recommendations,
                total: recommendations.len(),
                offset: 0,
                limit: 20,
                generated_at: Utc::now(),
                algorithm: "sample_discovery".to_string(),
                source: "sample_data".to_string(),
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get library statistics
async fn library_stats(State(state): State<AppState>) -> Result<Json<LibraryStats>, StatusCode> {
    // Always use sample data stats for now
    let artists_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM artists")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let albums_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM albums")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tracks_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tracks")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let navidrome_status = if let Some(_) = &state.navidrome {
        if *state.navidrome_connected.read().await {
            Some("connected".to_string())
        } else {
            Some("disconnected".to_string())
        }
    } else {
        Some("disabled".to_string())
    };

    Ok(Json(LibraryStats {
        total_tracks: tracks_count.0 as u32,
        total_albums: albums_count.0 as u32,
        total_artists: artists_count.0 as u32,
        total_users: users_count.0 as u32,
        total_listening_history: 0,
        database_size_mb: 0.1, // Small for sample data
        last_updated: Utc::now(),
        source: "sample_data".to_string(),
        navidrome_status,
    }))
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

    // Use sample data search
    let tracks = sqlx::query!(
        "SELECT t.id, t.title, a.name as artist, al.title as album
         FROM tracks t
         JOIN artists a ON t.artist_id = a.id
         JOIN albums al ON t.album_id = al.id
         WHERE t.title LIKE ? OR a.name LIKE ? OR al.title LIKE ?
         LIMIT ?",
        format!("%{}%", query),
        format!("%{}%", query),
        format!("%{}%", query),
        limit
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "query": query,
        "source": "sample_data",
        "results": {
            "tracks": tracks
        },
        "timestamp": Utc::now()
    })))
}

/// Generate smart playlist
async fn generate_playlist(
    State(state): State<AppState>,
    Json(request): Json<PlaylistRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let duration_minutes = request.duration_minutes.unwrap_or(60);
    let target_duration = duration_minutes * 60; // Convert to seconds

    // Generate playlist from sample data
    let tracks = sqlx::query!(
        "SELECT t.id, t.title, a.name as artist, t.duration
         FROM tracks t
         JOIN artists a ON t.artist_id = a.id
         ORDER BY RANDOM()
         LIMIT 20"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut playlist_tracks = Vec::new();
    let mut total_duration = 0;

    for track in tracks {
        if total_duration + track.duration < target_duration as i64 {
            total_duration += track.duration;
            playlist_tracks.push(serde_json::json!({
                "id": track.id,
                "title": track.title,
                "artist": track.artist,
                "duration": track.duration
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "message": "Smart playlist generated",
        "playlist": {
            "name": request.name,
            "description": request.description,
            "tracks": playlist_tracks,
            "total_duration": total_duration,
            "track_count": playlist_tracks.len(),
            "source": "sample_data"
        },
        "timestamp": Utc::now()
    })))
}

/// List all users
async fn list_users(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Use sample data users
    let users = sqlx::query!("SELECT id, username, email FROM users")
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "users": users,
        "count": users.len(),
        "source": "sample_data",
        "timestamp": Utc::now()
    })))
}

/// Get user listening history
async fn get_user_history(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return empty history as we don't have this implemented yet
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "history": [],
        "message": "Listening history not yet implemented",
        "timestamp": Utc::now()
    })))
}

/// Get Navidrome connection status
async fn navidrome_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let enabled = state.navidrome.is_some();
    let connected = *state.navidrome_connected.read().await;

    Json(serde_json::json!({
        "navidrome": {
            "enabled": enabled,
            "connected": connected,
            "url": std::env::var("NAVIDROME_URL").unwrap_or_default(),
            "status": if !enabled {
                "disabled"
            } else if connected {
                "connected"
            } else {
                "disconnected"
            }
        },
        "timestamp": Utc::now()
    }))
}

/// Test Navidrome connection
async fn navidrome_test(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(client) = &state.navidrome {
        match client.health_check().await {
            Ok(_) => {
                // Update connection status
                *state.navidrome_connected.write().await = true;
                Json(serde_json::json!({
                    "status": "success",
                    "message": "Navidrome connection test successful",
                    "connected": true,
                    "timestamp": Utc::now()
                }))
            }
            Err(e) => {
                // Update connection status
                *state.navidrome_connected.write().await = false;
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Navidrome connection failed: {}", e),
                    "connected": false,
                    "timestamp": Utc::now()
                }))
            }
        }
    } else {
        Json(serde_json::json!({
            "status": "disabled",
            "message": "Navidrome integration not configured",
            "connected": false,
            "timestamp": Utc::now()
        }))
    }
}
