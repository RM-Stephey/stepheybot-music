//! Database module for StepheyBot Music
//!
//! Provides database connection management, migrations, and common database operations
//! using SQLx with SQLite for reliable music data storage.

use anyhow::{Context, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool, Row,
};
use std::{path::Path, str::FromStr, time::Duration};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Database connection pool and management
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Initializing database connection: {}", database_url);

        // Parse connection options
        let connect_options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(30))
            .pragma("cache_size", "-64000") // 64MB cache
            .pragma("temp_store", "memory")
            .pragma("mmap_size", "268435456") // 256MB mmap
            .pragma("optimize", "0x10002"); // Enable query planner optimizations

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect_with(connect_options)
            .await
            .context("Failed to create database pool")?;

        debug!("Database pool created successfully");

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .context("Database health check failed")?;

        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM users) as user_count,
                (SELECT COUNT(*) FROM tracks) as track_count,
                (SELECT COUNT(*) FROM albums) as album_count,
                (SELECT COUNT(*) FROM artists) as artist_count,
                (SELECT COUNT(*) FROM playlists) as playlist_count,
                (SELECT COUNT(*) FROM listening_history) as listening_history_count,
                (SELECT COUNT(*) FROM recommendations) as recommendation_count
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            user_count: row.get::<i64, _>("user_count") as u64,
            track_count: row.get::<i64, _>("track_count") as u64,
            album_count: row.get::<i64, _>("album_count") as u64,
            artist_count: row.get::<i64, _>("artist_count") as u64,
            playlist_count: row.get::<i64, _>("playlist_count") as u64,
            listening_history_count: row.get::<i64, _>("listening_history_count") as u64,
            recommendation_count: row.get::<i64, _>("recommendation_count") as u64,
        })
    }

    /// Clean up old data based on retention policies
    pub async fn cleanup_old_data(&self, retention_days: u32) -> Result<CleanupResult> {
        let mut tx = self.pool.begin().await?;

        // Clean up old listening history
        let listening_history_deleted = sqlx::query(
            "DELETE FROM listening_history WHERE created_at < datetime('now', '-' || ? || ' days')"
        )
        .bind(retention_days)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Clean up old recommendations
        let recommendations_deleted = sqlx::query(
            "DELETE FROM recommendations WHERE created_at < datetime('now', '-' || ? || ' days')"
        )
        .bind(retention_days)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Clean up orphaned playlist entries
        let playlist_entries_deleted = sqlx::query(
            r#"
            DELETE FROM playlist_tracks
            WHERE playlist_id NOT IN (SELECT id FROM playlists)
            OR track_id NOT IN (SELECT id FROM tracks)
            "#
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Clean up orphaned tracks (tracks without files)
        let orphaned_tracks_deleted = sqlx::query(
            r#"
            DELETE FROM tracks
            WHERE id NOT IN (
                SELECT DISTINCT track_id FROM playlist_tracks
            ) AND file_path IS NULL
            "#
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        tx.commit().await?;

        let result = CleanupResult {
            listening_history_deleted,
            recommendations_deleted,
            playlist_entries_deleted,
            orphaned_tracks_deleted,
        };

        info!("Database cleanup completed: {:?}", result);
        Ok(result)
    }

    /// Vacuum the database to reclaim space
    pub async fn vacuum(&self) -> Result<()> {
        info!("Running database vacuum");

        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await
            .context("Failed to vacuum database")?;

        info!("Database vacuum completed");
        Ok(())
    }

    /// Analyze database for query optimization
    pub async fn analyze(&self) -> Result<()> {
        info!("Running database analysis");

        sqlx::query("ANALYZE")
            .execute(&self.pool)
            .await
            .context("Failed to analyze database")?;

        info!("Database analysis completed");
        Ok(())
    }

    /// Get database file size information
    pub async fn get_size_info(&self) -> Result<DatabaseSizeInfo> {
        let row = sqlx::query(
            r#"
            SELECT
                page_count * page_size as total_size,
                freelist_count * page_size as free_size,
                (page_count - freelist_count) * page_size as used_size
            FROM pragma_page_count(), pragma_page_size(), pragma_freelist_count()
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseSizeInfo {
            total_size: row.get::<i64, _>("total_size") as u64,
            used_size: row.get::<i64, _>("used_size") as u64,
            free_size: row.get::<i64, _>("free_size") as u64,
        })
    }

    /// Create a new database transaction
    pub async fn begin_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>> {
        self.pool.begin().await.context("Failed to begin transaction")
    }

    /// Execute a query with retries for handling busy database
    pub async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> futures::future::BoxFuture<'_, Result<T>> + Send + Sync,
        T: Send,
    {
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(e);
                    }

                    // Check if it's a busy/locked error
                    if let Some(db_err) = e.downcast_ref::<sqlx::Error>() {
                        match db_err {
                            sqlx::Error::Database(db_err) if db_err.code() == Some("5".into()) => {
                                // SQLite busy error, retry with exponential backoff
                                let delay = Duration::from_millis(100 * (2_u64.pow(attempts - 1)));
                                warn!("Database busy, retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                            _ => return Err(e),
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub user_count: u64,
    pub track_count: u64,
    pub album_count: u64,
    pub artist_count: u64,
    pub playlist_count: u64,
    pub listening_history_count: u64,
    pub recommendation_count: u64,
}

/// Database cleanup results
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub listening_history_deleted: u64,
    pub recommendations_deleted: u64,
    pub playlist_entries_deleted: u64,
    pub orphaned_tracks_deleted: u64,
}

/// Database size information
#[derive(Debug, Clone)]
pub struct DatabaseSizeInfo {
    pub total_size: u64,
    pub used_size: u64,
    pub free_size: u64,
}

impl DatabaseSizeInfo {
    /// Get the database usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used_size as f64 / self.total_size as f64) * 100.0
        }
    }

    /// Get the total size in MB
    pub fn total_size_mb(&self) -> f64 {
        self.total_size as f64 / 1_048_576.0
    }

    /// Get the used size in MB
    pub fn used_size_mb(&self) -> f64 {
        self.used_size as f64 / 1_048_576.0
    }

    /// Get the free size in MB
    pub fn free_size_mb(&self) -> f64 {
        self.free_size as f64 / 1_048_576.0
    }
}

/// Utility functions for database operations
pub mod utils {
    use super::*;

    /// Generate a new UUID string
    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get current timestamp in ISO format
    pub fn current_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    /// Convert duration to seconds
    pub fn duration_to_seconds(duration: Duration) -> f64 {
        duration.as_secs_f64()
    }

    /// Create a database backup
    pub async fn backup_database(source_path: &Path, backup_path: &Path) -> Result<()> {
        info!("Creating database backup: {} -> {}", source_path.display(), backup_path.display());

        if let Some(parent) = backup_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::copy(source_path, backup_path).await?;

        info!("Database backup completed successfully");
        Ok(())
    }

    /// Restore database from backup
    pub async fn restore_database(backup_path: &Path, target_path: &Path) -> Result<()> {
        info!("Restoring database from backup: {} -> {}", backup_path.display(), target_path.display());

        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::copy(backup_path, target_path).await?;

        info!("Database restoration completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());

        let db = Database::new(&db_url).await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_database_stats() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().display());

        let db = Database::new(&db_url).await.unwrap();
        db.migrate().await.unwrap();

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.user_count, 0);
        assert_eq!(stats.track_count, 0);
    }

    #[test]
    fn test_database_size_info() {
        let size_info = DatabaseSizeInfo {
            total_size: 1_048_576, // 1 MB
            used_size: 524_288,    // 512 KB
            free_size: 524_288,    // 512 KB
        };

        assert_eq!(size_info.usage_percentage(), 50.0);
        assert_eq!(size_info.total_size_mb(), 1.0);
        assert_eq!(size_info.used_size_mb(), 0.5);
        assert_eq!(size_info.free_size_mb(), 0.5);
    }

    #[test]
    fn test_utils() {
        let id = utils::generate_id();
        assert!(uuid::Uuid::parse_str(&id).is_ok());

        let timestamp = utils::current_timestamp();
        assert!(chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok());

        let duration = Duration::from_secs(60);
        assert_eq!(utils::duration_to_seconds(duration), 60.0);
    }
}
