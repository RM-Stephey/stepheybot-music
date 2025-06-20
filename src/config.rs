//! Configuration management for StepheyBot Music
//!
//! Supports loading configuration from environment variables, configuration files,
//! and .env files with proper validation and defaults.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Navidrome integration settings
    pub navidrome: NavidromeConfig,

    /// ListenBrainz integration settings
    pub listenbrainz: ListenBrainzConfig,

    /// Lidarr integration settings
    pub lidarr: LidarrConfig,

    /// MusicBrainz integration settings
    pub musicbrainz: MusicBrainzConfig,

    /// File system paths
    pub paths: PathsConfig,

    /// Background task settings
    pub tasks: TasksConfig,

    /// Recommendation engine settings
    pub recommendations: RecommendationConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    #[serde(default = "default_server_address")]
    pub address: String,

    /// Server port
    #[serde(default = "default_server_port")]
    pub port: u16,

    /// Enable admin API endpoints
    #[serde(default = "default_true")]
    pub enable_admin_api: bool,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL (SQLite file path)
    #[serde(default = "default_database_url")]
    pub url: String,

    /// Maximum number of database connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Database connection timeout in seconds
    #[serde(default = "default_db_timeout")]
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavidromeConfig {
    /// Navidrome server URL
    pub url: String,

    /// Username for Navidrome
    pub username: String,

    /// Password for Navidrome
    pub password: String,

    /// API timeout in seconds
    #[serde(default = "default_api_timeout")]
    pub timeout: u64,

    /// Enable SSL verification
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenBrainzConfig {
    /// ListenBrainz server URL
    #[serde(default = "default_listenbrainz_url")]
    pub url: String,

    /// ListenBrainz user token (optional)
    pub token: Option<String>,

    /// API timeout in seconds
    #[serde(default = "default_api_timeout")]
    pub timeout: u64,

    /// Rate limit: requests per minute
    #[serde(default = "default_listenbrainz_rate_limit")]
    pub rate_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LidarrConfig {
    /// Lidarr server URL
    pub url: String,

    /// Lidarr API key
    pub api_key: String,

    /// API timeout in seconds
    #[serde(default = "default_api_timeout")]
    pub timeout: u64,

    /// Enable automatic downloads
    #[serde(default = "default_true")]
    pub enable_downloads: bool,

    /// Quality profile ID
    #[serde(default = "default_quality_profile")]
    pub quality_profile_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicBrainzConfig {
    /// User agent for MusicBrainz API
    #[serde(default = "default_musicbrainz_user_agent")]
    pub user_agent: String,

    /// API timeout in seconds
    #[serde(default = "default_api_timeout")]
    pub timeout: u64,

    /// Rate limit: requests per second
    #[serde(default = "default_musicbrainz_rate_limit")]
    pub rate_limit: f64,

    /// Enable cover art fetching
    #[serde(default = "default_true")]
    pub enable_cover_art: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Music library root path
    pub music_path: PathBuf,

    /// Download directory path
    pub download_path: PathBuf,

    /// Cache directory path
    #[serde(default = "default_cache_path")]
    pub cache_path: PathBuf,

    /// Database file path
    #[serde(default = "default_db_path")]
    pub database_path: PathBuf,

    /// Log file directory
    #[serde(default = "default_log_path")]
    pub log_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    /// Sync interval in seconds
    #[serde(default = "default_sync_interval")]
    pub sync_interval: u64,

    /// Recommendation generation interval in seconds (daily = 86400)
    #[serde(default = "default_recommendation_interval")]
    pub recommendation_interval: u64,

    /// Library scan interval in seconds
    #[serde(default = "default_library_scan_interval")]
    pub library_scan_interval: u64,

    /// Cleanup interval in seconds
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval: u64,

    /// Enable background tasks
    #[serde(default = "default_true")]
    pub enable_background_tasks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConfig {
    /// Number of recommendations to generate per user
    #[serde(default = "default_recommendations_count")]
    pub count: u32,

    /// Minimum listening history required for recommendations
    #[serde(default = "default_min_listening_history")]
    pub min_listening_history: u32,

    /// Weight for collaborative filtering (0.0-1.0)
    #[serde(default = "default_collaborative_weight")]
    pub collaborative_weight: f64,

    /// Weight for content-based filtering (0.0-1.0)
    #[serde(default = "default_content_weight")]
    pub content_weight: f64,

    /// Weight for popularity-based filtering (0.0-1.0)
    #[serde(default = "default_popularity_weight")]
    pub popularity_weight: f64,

    /// Enable discovery mode (find new music)
    #[serde(default = "default_true")]
    pub enable_discovery: bool,

    /// Discovery ratio (0.0-1.0) - percentage of recommendations that should be new music
    #[serde(default = "default_discovery_ratio")]
    pub discovery_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable JSON logging
    #[serde(default = "default_false")]
    pub json: bool,

    /// Enable file logging
    #[serde(default = "default_true")]
    pub file: bool,

    /// Log file rotation size in MB
    #[serde(default = "default_log_rotation_size")]
    pub rotation_size: u64,

    /// Number of log files to keep
    #[serde(default = "default_log_retention")]
    pub retention: u32,
}

impl Config {
    /// Load configuration from environment variables and configuration files
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let mut config_builder = config::Config::builder()
            // Start with default configuration
            .set_default("server.address", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.enable_admin_api", true)?
            .set_default("server.request_timeout", 30)?
            .set_default("database.url", "sqlite:data/stepheybot-music.db")?
            .set_default("database.max_connections", 10)?
            .set_default("database.connection_timeout", 30)?
            .set_default("listenbrainz.url", "https://api.listenbrainz.org")?
            .set_default("listenbrainz.timeout", 30)?
            .set_default("listenbrainz.rate_limit", 60)?
            .set_default(
                "musicbrainz.user_agent",
                "StepheyBot-Music/1.0 (https://stepheybot.dev)",
            )?
            .set_default("musicbrainz.timeout", 30)?
            .set_default("musicbrainz.rate_limit", 1.0)?
            .set_default("musicbrainz.enable_cover_art", true)?
            .set_default("paths.cache_path", "data/cache")?
            .set_default("paths.database_path", "data/stepheybot-music.db")?
            .set_default("paths.log_path", "data/logs")?
            .set_default("tasks.sync_interval", 3600)? // 1 hour
            .set_default("tasks.recommendation_interval", 86400)? // 1 day
            .set_default("tasks.library_scan_interval", 1800)? // 30 minutes
            .set_default("tasks.cleanup_interval", 43200)? // 12 hours
            .set_default("tasks.enable_background_tasks", true)?
            .set_default("recommendations.count", 50)?
            .set_default("recommendations.min_listening_history", 10)?
            .set_default("recommendations.collaborative_weight", 0.4)?
            .set_default("recommendations.content_weight", 0.4)?
            .set_default("recommendations.popularity_weight", 0.2)?
            .set_default("recommendations.enable_discovery", true)?
            .set_default("recommendations.discovery_ratio", 0.3)?
            .set_default("logging.level", "info")?
            .set_default("logging.json", false)?
            .set_default("logging.file", true)?
            .set_default("logging.rotation_size", 100)?
            .set_default("logging.retention", 7)?;

        // Try to load from configuration file
        if let Ok(config_path) = std::env::var("STEPHEYBOT_CONFIG_FILE") {
            config_builder =
                config_builder.add_source(config::File::with_name(&config_path).required(false));
        } else {
            // Try common configuration file locations
            config_builder = config_builder
                .add_source(config::File::with_name("config/config").required(false))
                .add_source(config::File::with_name("config").required(false))
                .add_source(config::File::with_name("stepheybot-music").required(false));
        }

        // Override with environment variables
        config_builder = config_builder.add_source(
            config::Environment::with_prefix("STEPHEYBOT")
                .separator("__")
                .try_parsing(true),
        );

        let config = config_builder
            .build()
            .context("Failed to build configuration")?;
        let config: Config = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Validate configuration
        config
            .validate()
            .context("Configuration validation failed")?;

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            anyhow::bail!("Server port cannot be 0");
        }

        // Validate URLs
        url::Url::parse(&self.navidrome.url).context("Invalid Navidrome URL")?;
        url::Url::parse(&self.listenbrainz.url).context("Invalid ListenBrainz URL")?;
        url::Url::parse(&self.lidarr.url).context("Invalid Lidarr URL")?;

        // Validate paths exist or can be created
        for path in [
            &self.paths.music_path,
            &self.paths.download_path,
            &self.paths.cache_path,
        ] {
            if !path.exists() {
                std::fs::create_dir_all(path)
                    .with_context(|| format!("Failed to create directory: {}", path.display()))?;
            }
        }

        // Validate recommendation weights sum to approximately 1.0
        let total_weight = self.recommendations.collaborative_weight
            + self.recommendations.content_weight
            + self.recommendations.popularity_weight;
        if (total_weight - 1.0).abs() > 0.1 {
            anyhow::bail!(
                "Recommendation weights should sum to approximately 1.0, got {}",
                total_weight
            );
        }

        Ok(())
    }

    /// Get the database URL
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Get the server port
    pub fn server_port(&self) -> u16 {
        self.server.port
    }

    /// Get Navidrome URL
    pub fn navidrome_url(&self) -> &str {
        &self.navidrome.url
    }

    /// Get Navidrome admin credentials
    pub fn navidrome_username(&self) -> &str {
        &self.navidrome.username
    }

    pub fn navidrome_password(&self) -> &str {
        &self.navidrome.password
    }

    /// Get ListenBrainz URL
    pub fn listenbrainz_url(&self) -> &str {
        &self.listenbrainz.url
    }

    /// Get ListenBrainz token
    pub fn listenbrainz_token(&self) -> Option<&str> {
        self.listenbrainz.token.as_deref()
    }

    /// Get Lidarr URL and API key
    pub fn lidarr_url(&self) -> &str {
        &self.lidarr.url
    }

    pub fn lidarr_api_key(&self) -> &str {
        &self.lidarr.api_key
    }

    /// Get MusicBrainz user agent
    pub fn musicbrainz_user_agent(&self) -> &str {
        &self.musicbrainz.user_agent
    }

    /// Get file paths
    pub fn music_path(&self) -> &PathBuf {
        &self.paths.music_path
    }

    pub fn download_path(&self) -> &PathBuf {
        &self.paths.download_path
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.paths.cache_path
    }
}

// Default value functions
fn default_server_address() -> String {
    "0.0.0.0".to_string()
}
fn default_server_port() -> u16 {
    8080
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_request_timeout() -> u64 {
    30
}
fn default_database_url() -> String {
    "sqlite:data/stepheybot-music.db".to_string()
}
fn default_max_connections() -> u32 {
    10
}
fn default_db_timeout() -> u64 {
    30
}
fn default_api_timeout() -> u64 {
    30
}
fn default_listenbrainz_url() -> String {
    "https://api.listenbrainz.org".to_string()
}
fn default_listenbrainz_rate_limit() -> u32 {
    60
}
fn default_quality_profile() -> u32 {
    1
}
fn default_musicbrainz_user_agent() -> String {
    "StepheyBot-Music/1.0 (https://stepheybot.dev)".to_string()
}
fn default_musicbrainz_rate_limit() -> f64 {
    1.0
}
fn default_cache_path() -> PathBuf {
    PathBuf::from("data/cache")
}
fn default_db_path() -> PathBuf {
    PathBuf::from("data/stepheybot-music.db")
}
fn default_log_path() -> PathBuf {
    PathBuf::from("data/logs")
}
fn default_sync_interval() -> u64 {
    3600
}
fn default_recommendation_interval() -> u64 {
    86400
}
fn default_library_scan_interval() -> u64 {
    1800
}
fn default_cleanup_interval() -> u64 {
    43200
}
fn default_recommendations_count() -> u32 {
    50
}
fn default_min_listening_history() -> u32 {
    10
}
fn default_collaborative_weight() -> f64 {
    0.4
}
fn default_content_weight() -> f64 {
    0.4
}
fn default_popularity_weight() -> f64 {
    0.2
}
fn default_discovery_ratio() -> f64 {
    0.3
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_rotation_size() -> u64 {
    100
}
fn default_log_retention() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            server: ServerConfig {
                address: "0.0.0.0".to_string(),
                port: 8080,
                enable_admin_api: true,
                request_timeout: 30,
            },
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(),
                max_connections: 10,
                connection_timeout: 30,
            },
            navidrome: NavidromeConfig {
                url: "http://localhost:4533".to_string(),
                admin_user: "admin".to_string(),
                admin_password: "password".to_string(),
                timeout: 30,
                verify_ssl: true,
            },
            listenbrainz: ListenBrainzConfig {
                url: "https://api.listenbrainz.org".to_string(),
                token: None,
                timeout: 30,
                rate_limit: 60,
            },
            lidarr: LidarrConfig {
                url: "http://localhost:8686".to_string(),
                api_key: "test_key".to_string(),
                timeout: 30,
                enable_downloads: true,
                quality_profile_id: 1,
            },
            musicbrainz: MusicBrainzConfig {
                user_agent: "StepheyBot-Music/1.0".to_string(),
                timeout: 30,
                rate_limit: 1.0,
                enable_cover_art: true,
            },
            paths: PathsConfig {
                music_path: PathBuf::from("/tmp/music"),
                download_path: PathBuf::from("/tmp/downloads"),
                cache_path: PathBuf::from("/tmp/cache"),
                database_path: PathBuf::from("/tmp/db.sqlite"),
                log_path: PathBuf::from("/tmp/logs"),
            },
            tasks: TasksConfig {
                sync_interval: 3600,
                recommendation_interval: 86400,
                library_scan_interval: 1800,
                cleanup_interval: 43200,
                enable_background_tasks: true,
            },
            recommendations: RecommendationConfig {
                count: 50,
                min_listening_history: 10,
                collaborative_weight: 0.4,
                content_weight: 0.4,
                popularity_weight: 0.2,
                enable_discovery: true,
                discovery_ratio: 0.3,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                json: false,
                file: true,
                rotation_size: 100,
                retention: 7,
            },
        };

        // This should pass validation
        assert!(config.validate().is_ok());

        // Test invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
        config.server.port = 8080;

        // Test invalid URL
        config.navidrome.url = "not-a-url".to_string();
        assert!(config.validate().is_err());
        config.navidrome.url = "http://localhost:4533".to_string();

        // Test invalid recommendation weights
        config.recommendations.collaborative_weight = 0.8;
        assert!(config.validate().is_err());
    }
}
