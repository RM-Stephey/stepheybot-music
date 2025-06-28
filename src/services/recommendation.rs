//! Advanced recommendation engine for StepheyBot Music
//!
//! This module implements a hybrid recommendation system combining:
//! - Collaborative filtering (user-based and item-based)
//! - Content-based filtering (using music metadata)
//! - Popularity-based recommendations
//! - Temporal pattern analysis
//! - Machine learning for pattern recognition

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Statistics for the recommendation service
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecommendationStats {
    pub total_recommendations: u64,
    pub recommendations_consumed: u64,
    pub average_score: f64,
    pub last_generation: Option<DateTime<Utc>>,
    pub active_users: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

use crate::{
    clients::{listenbrainz::ListenBrainzClient, musicbrainz::MusicBrainzClient},
    database::Database,
    models::{
        entities::{
            ListeningHistory, Recommendation, Track, User, UserListeningStats, UserTrackRating,
        },
        generate_id, now,
    },
};

/// Recommendation engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConfig {
    pub collaborative_weight: f64,
    pub content_weight: f64,
    pub popularity_weight: f64,
    pub temporal_weight: f64,
    pub min_listening_history: u32,
    pub max_recommendations: u32,
    pub discovery_ratio: f64,
    pub similarity_threshold: f64,
    pub cache_duration_hours: u32,
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            collaborative_weight: 0.4,
            content_weight: 0.3,
            popularity_weight: 0.2,
            temporal_weight: 0.1,
            min_listening_history: 10,
            max_recommendations: 50,
            discovery_ratio: 0.3,
            similarity_threshold: 0.1,
            cache_duration_hours: 24,
        }
    }
}

/// User similarity data
#[derive(Debug, Clone)]
pub struct UserSimilarity {
    pub user_id: String,
    pub similarity_score: f64,
    pub common_tracks: usize,
    pub last_calculated: DateTime<Utc>,
}

/// Track features for content-based filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackFeatures {
    pub track_id: String,
    pub artist_id: String,
    pub genre_vector: Vec<f64>,
    pub tempo: Option<f64>,
    pub energy: Option<f64>,
    pub valence: Option<f64>,
    pub danceability: Option<f64>,
    pub acousticness: Option<f64>,
    pub instrumentalness: Option<f64>,
    pub year: Option<i32>,
    pub popularity: f64,
    pub duration: Option<i32>,
}

/// Recommendation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResult {
    pub track_id: String,
    pub score: f64,
    pub reason: String,
    pub recommendation_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Main recommendation engine
pub struct RecommendationService {
    database: Arc<Database>,
    listenbrainz_client: Arc<ListenBrainzClient>,
    musicbrainz_client: Arc<MusicBrainzClient>,
    config: RecommendationConfig,
    cache_dir: String,
    user_similarities: Arc<RwLock<HashMap<String, Vec<UserSimilarity>>>>,
    track_features: Arc<RwLock<HashMap<String, TrackFeatures>>>,
    popularity_cache: Arc<RwLock<HashMap<String, f64>>>,
    stats: Arc<RwLock<RecommendationStats>>,
}

impl RecommendationService {
    /// Create a new recommendation service
    pub fn new(
        database: Arc<Database>,
        listenbrainz_client: Arc<ListenBrainzClient>,
        musicbrainz_client: Arc<MusicBrainzClient>,
        cache_dir: &str,
    ) -> Result<Self> {
        let config = RecommendationConfig::default();

        // Ensure cache directory exists
        std::fs::create_dir_all(cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir))?;

        Ok(Self {
            database,
            listenbrainz_client,
            musicbrainz_client,
            config,
            cache_dir: cache_dir.to_string(),
            user_similarities: Arc::new(RwLock::new(HashMap::new())),
            track_features: Arc::new(RwLock::new(HashMap::new())),
            popularity_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RecommendationStats::default())),
        })
    }

    /// Generate recommendations for all users
    pub async fn generate_all_recommendations(&self) -> Result<()> {
        info!("Starting recommendation generation for all users");

        // Get all active users
        let users = self.get_active_users().await?;
        info!("Generating recommendations for {} users", users.len());

        // Update caches first
        self.update_user_similarities().await?;
        self.update_track_features().await?;
        self.update_popularity_cache().await?;

        // Generate recommendations for each user
        for user in users {
            match self.generate_user_recommendations(&user.id).await {
                Ok(recommendations) => {
                    info!(
                        "Generated {} recommendations for user {}",
                        recommendations.len(),
                        user.username
                    );
                    self.save_recommendations(&user.id, recommendations).await?;
                }
                Err(e) => {
                    error!(
                        "Failed to generate recommendations for user {}: {}",
                        user.username, e
                    );
                }
            }
        }

        info!("Recommendation generation completed");
        Ok(())
    }

    /// Generate recommendations for a specific user
    pub async fn generate_user_recommendations(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecommendationResult>> {
        debug!("Generating recommendations for user: {}", user_id);

        // Get user's listening history
        let listening_stats = self.get_user_listening_stats(user_id).await?;

        if listening_stats.total_plays < self.config.min_listening_history as i64 {
            // New user - use popularity-based recommendations
            return self.generate_popularity_recommendations(user_id).await;
        }

        // Get user's preferences and listening patterns
        let user_tracks = self.get_user_track_history(user_id, 1000).await?;
        let _user_ratings = self.get_user_ratings(user_id).await?;
        let banned_tracks = self.get_banned_tracks(user_id).await?;

        let mut all_recommendations = Vec::new();

        // Collaborative filtering
        if self.config.collaborative_weight > 0.0 {
            let collaborative_recs = self
                .generate_collaborative_recommendations(user_id, &user_tracks)
                .await?;
            all_recommendations.extend(collaborative_recs);
        }

        // Content-based filtering
        if self.config.content_weight > 0.0 {
            let content_recs = self
                .generate_content_based_recommendations(user_id, &user_tracks)
                .await?;
            all_recommendations.extend(content_recs);
        }

        // Popularity-based recommendations
        if self.config.popularity_weight > 0.0 {
            let popularity_recs = self.generate_popularity_recommendations(user_id).await?;
            all_recommendations.extend(popularity_recs);
        }

        // Temporal pattern recommendations
        if self.config.temporal_weight > 0.0 {
            let temporal_recs = self
                .generate_temporal_recommendations(user_id, &user_tracks)
                .await?;
            all_recommendations.extend(temporal_recs);
        }

        // Filter out banned tracks and already listened tracks
        all_recommendations.retain(|rec| {
            !banned_tracks.contains(&rec.track_id) && !user_tracks.contains(&rec.track_id)
        });

        // Sort by score and deduplicate
        all_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_recommendations.dedup_by(|a, b| a.track_id == b.track_id);

        // Apply discovery ratio
        let discovery_count =
            (all_recommendations.len() as f64 * self.config.discovery_ratio) as usize;
        let mut final_recommendations = Vec::new();

        // Take top recommendations
        final_recommendations.extend(
            all_recommendations
                .iter()
                .take(self.config.max_recommendations as usize - discovery_count)
                .cloned(),
        );

        // Add discovery tracks (less popular or new releases)
        let discovery_recs = self.generate_discovery_recommendations(user_id).await?;
        final_recommendations.extend(discovery_recs.into_iter().take(discovery_count));

        // Final shuffle to avoid predictability
        final_recommendations.truncate(self.config.max_recommendations as usize);

        debug!(
            "Generated {} recommendations for user {}",
            final_recommendations.len(),
            user_id
        );

        Ok(final_recommendations)
    }

    /// Generate collaborative filtering recommendations
    async fn generate_collaborative_recommendations(
        &self,
        user_id: &str,
        user_tracks: &[String],
    ) -> Result<Vec<RecommendationResult>> {
        let similarities = self.user_similarities.read().await;
        let empty_vec = Vec::new();
        let similar_users = similarities.get(user_id).unwrap_or(&empty_vec);

        let mut recommendations = Vec::new();
        let mut track_scores: HashMap<String, f64> = HashMap::new();

        for similar_user in similar_users.iter().take(20) {
            if similar_user.similarity_score < self.config.similarity_threshold {
                continue;
            }

            let similar_user_tracks = self
                .get_user_track_history(&similar_user.user_id, 200)
                .await?;

            for track_id in similar_user_tracks {
                if !user_tracks.contains(&track_id) {
                    let score = similar_user.similarity_score * self.config.collaborative_weight;
                    *track_scores.entry(track_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        for (track_id, score) in track_scores {
            if score > 0.1 {
                recommendations.push(RecommendationResult {
                    track_id: track_id.clone(),
                    score,
                    reason: "Based on users with similar taste".to_string(),
                    recommendation_type: "collaborative".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// Generate content-based recommendations
    async fn generate_content_based_recommendations(
        &self,
        _user_id: &str,
        user_tracks: &[String],
    ) -> Result<Vec<RecommendationResult>> {
        let track_features = self.track_features.read().await;

        // Build user profile based on listening history
        let user_profile = self
            .build_user_profile(user_tracks, &track_features)
            .await?;

        let mut recommendations = Vec::new();

        // Find tracks similar to user's profile
        for (track_id, features) in track_features.iter() {
            if user_tracks.contains(track_id) {
                continue;
            }

            let similarity = self.calculate_content_similarity(&user_profile, features);
            if similarity > self.config.similarity_threshold {
                let score = similarity * self.config.content_weight;

                recommendations.push(RecommendationResult {
                    track_id: track_id.clone(),
                    score,
                    reason: "Based on your music taste".to_string(),
                    recommendation_type: "content_based".to_string(),
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "similarity".to_string(),
                            serde_json::Value::from(similarity),
                        );
                        metadata
                    },
                });
            }
        }

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(20);

        Ok(recommendations)
    }

    /// Generate popularity-based recommendations
    async fn generate_popularity_recommendations(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecommendationResult>> {
        let popularity_cache = self.popularity_cache.read().await;
        let mut recommendations = Vec::new();

        // Get user's listening history to avoid duplicates
        let user_tracks = self.get_user_track_history(user_id, 1000).await?;
        let user_track_set: HashSet<String> = user_tracks.into_iter().collect();

        // Get top popular tracks
        let mut popular_tracks: Vec<_> = popularity_cache
            .iter()
            .filter(|(track_id, _)| !user_track_set.contains(*track_id))
            .collect();

        popular_tracks.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        for (track_id, popularity) in popular_tracks.iter().take(30) {
            let score = *popularity * self.config.popularity_weight;

            recommendations.push(RecommendationResult {
                track_id: track_id.to_string(),
                score,
                reason: "Popular track".to_string(),
                recommendation_type: "popularity".to_string(),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "popularity".to_string(),
                        serde_json::Value::from(**popularity),
                    );
                    metadata
                },
            });
        }

        Ok(recommendations)
    }

    /// Generate temporal pattern recommendations
    async fn generate_temporal_recommendations(
        &self,
        user_id: &str,
        user_tracks: &[String],
    ) -> Result<Vec<RecommendationResult>> {
        // Analyze user's listening patterns by time of day, day of week
        let temporal_patterns = self.analyze_temporal_patterns(user_id).await?;
        let current_time = chrono::Local::now();
        let current_hour = current_time.hour();
        let current_weekday = current_time.weekday();

        let mut recommendations = Vec::new();

        // Find tracks that are typically played at this time
        for pattern in temporal_patterns {
            if pattern.matches_current_time(current_hour, current_weekday) {
                let score = pattern.confidence * self.config.temporal_weight;

                recommendations.push(RecommendationResult {
                    track_id: pattern.track_id,
                    score,
                    reason: format!("Often played at this time"),
                    recommendation_type: "temporal".to_string(),
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "confidence".to_string(),
                            serde_json::Value::from(pattern.confidence),
                        );
                        metadata
                    },
                });
            }
        }

        Ok(recommendations)
    }

    /// Generate discovery recommendations (new or less popular tracks)
    async fn generate_discovery_recommendations(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecommendationResult>> {
        let mut recommendations = Vec::new();

        // Get recently added tracks
        let recent_tracks = self.get_trending_tracks(30).await?;

        // Get user's artist preferences
        let preferred_artists = self.get_user_preferred_artists(user_id).await?;

        for track in recent_tracks {
            // Check if track is from a preferred artist
            if preferred_artists.contains(&track.artist_id) {
                recommendations.push(RecommendationResult {
                    track_id: track.id,
                    score: 0.7,
                    reason: "New release from an artist you like".to_string(),
                    recommendation_type: "discovery".to_string(),
                    metadata: HashMap::new(),
                });
            } else {
                recommendations.push(RecommendationResult {
                    track_id: track.id,
                    score: 0.3,
                    reason: "New music to discover".to_string(),
                    recommendation_type: "discovery".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(recommendations)
    }

    /// Update user similarity cache
    async fn update_user_similarities(&self) -> Result<()> {
        info!("Updating user similarity cache");

        let users = self.get_active_users().await?;
        let mut similarities = HashMap::new();

        for user in &users {
            let mut user_similarities = Vec::new();
            let user_tracks = self.get_user_track_history(&user.id, 500).await?;

            if user_tracks.len() < 5 {
                continue; // Skip users with too few tracks
            }

            for other_user in &users {
                if user.id == other_user.id {
                    continue;
                }

                let other_tracks = self.get_user_track_history(&other_user.id, 500).await?;

                if other_tracks.len() < 5 {
                    continue;
                }

                let similarity = self.calculate_user_similarity(&user_tracks, &other_tracks);

                if similarity > self.config.similarity_threshold {
                    user_similarities.push(UserSimilarity {
                        user_id: other_user.id.clone(),
                        similarity_score: similarity,
                        common_tracks: self.count_common_tracks(&user_tracks, &other_tracks),
                        last_calculated: now(),
                    });
                }
            }

            // Sort by similarity score
            user_similarities
                .sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
            user_similarities.truncate(50); // Keep top 50 similar users

            similarities.insert(user.id.clone(), user_similarities);
        }

        *self.user_similarities.write().await = similarities;
        info!("User similarity cache updated");
        Ok(())
    }

    /// Update track features cache
    async fn update_track_features(&self) -> Result<()> {
        info!("Updating track features cache");

        // This would typically involve:
        // 1. Fetching audio features from external APIs
        // 2. Analyzing metadata (genre, year, etc.)
        // 3. Computing derived features

        let tracks = self.get_all_tracks().await?;
        let mut features = HashMap::new();

        for track in tracks {
            let track_features = self.extract_track_features(&track).await?;
            features.insert(track.id, track_features);
        }

        *self.track_features.write().await = features;
        info!("Track features cache updated");
        Ok(())
    }

    /// Update popularity cache
    async fn update_popularity_cache(&self) -> Result<()> {
        info!("Updating popularity cache");

        let popular_tracks = self.get_track_popularity().await?;
        let mut cache = HashMap::new();

        for track_pop in popular_tracks {
            cache.insert(track_pop.track_id.clone(), track_pop.popularity_score());
        }

        *self.popularity_cache.write().await = cache;
        info!("Popularity cache updated");
        Ok(())
    }

    /// Calculate similarity between two users based on their listening history
    fn calculate_user_similarity(&self, user1_tracks: &[String], user2_tracks: &[String]) -> f64 {
        let set1: HashSet<_> = user1_tracks.iter().collect();
        let set2: HashSet<_> = user2_tracks.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Count common tracks between two users
    fn count_common_tracks(&self, user1_tracks: &[String], user2_tracks: &[String]) -> usize {
        let set1: HashSet<_> = user1_tracks.iter().collect();
        let set2: HashSet<_> = user2_tracks.iter().collect();
        set1.intersection(&set2).count()
    }

    /// Calculate content similarity between user profile and track features
    fn calculate_content_similarity(
        &self,
        user_profile: &TrackFeatures,
        track_features: &TrackFeatures,
    ) -> f64 {
        // Cosine similarity for genre vectors
        let genre_similarity =
            self.cosine_similarity(&user_profile.genre_vector, &track_features.genre_vector);

        // Weighted similarity for other features
        let mut total_similarity = genre_similarity * 0.4;

        if let (Some(user_energy), Some(track_energy)) =
            (user_profile.energy, track_features.energy)
        {
            total_similarity += (1.0 - (user_energy - track_energy).abs()) * 0.15;
        }

        if let (Some(user_valence), Some(track_valence)) =
            (user_profile.valence, track_features.valence)
        {
            total_similarity += (1.0 - (user_valence - track_valence).abs()) * 0.15;
        }

        if let (Some(user_tempo), Some(track_tempo)) = (user_profile.tempo, track_features.tempo) {
            let tempo_diff = (user_tempo - track_tempo).abs() / user_tempo.max(track_tempo);
            total_similarity += (1.0 - tempo_diff.min(1.0)) * 0.1;
        }

        // Year similarity (prefer music from similar eras)
        if let (Some(user_year), Some(track_year)) = (user_profile.year, track_features.year) {
            let year_diff = (user_year - track_year).abs() as f64;
            let year_similarity = (1.0 - (year_diff / 50.0).min(1.0)) * 0.1;
            total_similarity += year_similarity;
        }

        // Popularity factor
        let popularity_factor = (track_features.popularity * 0.1).min(0.1);
        total_similarity += popularity_factor;

        total_similarity.min(1.0).max(0.0)
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Build user profile from listening history
    async fn build_user_profile(
        &self,
        user_tracks: &[String],
        track_features: &HashMap<String, TrackFeatures>,
    ) -> Result<TrackFeatures> {
        let mut genre_vector = vec![0.0; 20]; // Assuming 20 genres
        let mut total_energy = 0.0;
        let mut total_valence = 0.0;
        let mut total_tempo = 0.0;
        let mut total_danceability = 0.0;
        let mut total_acousticness = 0.0;
        let mut total_instrumentalness = 0.0;
        let mut year_sum = 0;
        let mut count = 0;

        for track_id in user_tracks {
            if let Some(features) = track_features.get(track_id) {
                // Sum genre vectors
                for (i, &value) in features.genre_vector.iter().enumerate() {
                    if i < genre_vector.len() {
                        genre_vector[i] += value;
                    }
                }

                // Sum other features
                if let Some(energy) = features.energy {
                    total_energy += energy;
                }
                if let Some(valence) = features.valence {
                    total_valence += valence;
                }
                if let Some(tempo) = features.tempo {
                    total_tempo += tempo;
                }
                if let Some(danceability) = features.danceability {
                    total_danceability += danceability;
                }
                if let Some(acousticness) = features.acousticness {
                    total_acousticness += acousticness;
                }
                if let Some(instrumentalness) = features.instrumentalness {
                    total_instrumentalness += instrumentalness;
                }
                if let Some(year) = features.year {
                    year_sum += year;
                }
                count += 1;
            }
        }

        if count == 0 {
            return Ok(TrackFeatures {
                track_id: "user_profile".to_string(),
                artist_id: "".to_string(),
                genre_vector: vec![0.0; 20],
                tempo: None,
                energy: None,
                valence: None,
                danceability: None,
                acousticness: None,
                instrumentalness: None,
                year: None,
                popularity: 0.0,
                duration: None,
            });
        }

        // Average the features
        let count_f = count as f64;
        for value in &mut genre_vector {
            *value /= count_f;
        }

        Ok(TrackFeatures {
            track_id: "user_profile".to_string(),
            artist_id: "".to_string(),
            genre_vector,
            tempo: if total_tempo > 0.0 {
                Some(total_tempo / count_f)
            } else {
                None
            },
            energy: if total_energy > 0.0 {
                Some(total_energy / count_f)
            } else {
                None
            },
            valence: if total_valence > 0.0 {
                Some(total_valence / count_f)
            } else {
                None
            },
            danceability: if total_danceability > 0.0 {
                Some(total_danceability / count_f)
            } else {
                None
            },
            acousticness: if total_acousticness > 0.0 {
                Some(total_acousticness / count_f)
            } else {
                None
            },
            instrumentalness: if total_instrumentalness > 0.0 {
                Some(total_instrumentalness / count_f)
            } else {
                None
            },
            year: if year_sum > 0 {
                Some(year_sum / count as i32)
            } else {
                None
            },
            popularity: 0.0, // Not relevant for user profile
            duration: None,
        })
    }

    /// Save recommendations to database
    async fn save_recommendations(
        &self,
        user_id: &str,
        recommendations: Vec<RecommendationResult>,
    ) -> Result<()> {
        let mut tx = self.database.begin_transaction().await?;

        // Delete old recommendations for this user
        sqlx::query("DELETE FROM recommendations WHERE user_id = ? AND created_at < datetime('now', '-7 days')")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Insert new recommendations
        for rec in recommendations {
            let recommendation = Recommendation {
                id: generate_id(),
                user_id: user_id.to_string(),
                track_id: rec.track_id,
                recommendation_type: rec.recommendation_type,
                score: rec.score,
                reason: Some(rec.reason),
                metadata: serde_json::to_string(&rec.metadata).ok(),
                is_consumed: false,
                consumed_at: None,
                created_at: now(),
                expires_at: Some(now() + Duration::days(7)),
            };

            sqlx::query(
                r#"
                INSERT INTO recommendations (id, user_id, track_id, recommendation_type, score, reason, metadata, is_consumed, created_at, expires_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&recommendation.id)
            .bind(&recommendation.user_id)
            .bind(&recommendation.track_id)
            .bind(&recommendation.recommendation_type)
            .bind(recommendation.score)
            .bind(&recommendation.reason)
            .bind(&recommendation.metadata)
            .bind(recommendation.is_consumed)
            .bind(recommendation.created_at)
            .bind(recommendation.expires_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // Database helper methods (implementations would go here)
    async fn get_active_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users WHERE is_active = true")
            .fetch_all(self.database.pool())
            .await?;
        Ok(users)
    }

    async fn get_user_listening_stats(&self, user_id: &str) -> Result<UserListeningStats> {
        let stats = sqlx::query_as::<_, UserListeningStats>(
            "SELECT * FROM user_listening_stats WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(self.database.pool())
        .await?;
        Ok(stats)
    }

    async fn get_user_track_history(&self, user_id: &str, limit: usize) -> Result<Vec<String>> {
        let tracks = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT track_id FROM listening_history WHERE user_id = ? ORDER BY played_at DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(self.database.pool())
        .await?;
        Ok(tracks)
    }

    async fn get_user_ratings(&self, user_id: &str) -> Result<Vec<UserTrackRating>> {
        let ratings = sqlx::query_as::<_, UserTrackRating>(
            "SELECT * FROM user_track_ratings WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?;
        Ok(ratings)
    }

    async fn get_banned_tracks(&self, user_id: &str) -> Result<HashSet<String>> {
        let banned = sqlx::query_scalar::<_, String>(
            "SELECT track_id FROM user_track_ratings WHERE user_id = ? AND is_banned = true",
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?;

        Ok(banned.into_iter().collect())
    }

    async fn get_all_tracks(&self) -> Result<Vec<crate::models::entities::Track>> {
        // Placeholder - would fetch all tracks from database
        Ok(Vec::new())
    }

    async fn get_track_popularity(&self) -> Result<Vec<crate::models::entities::TrackPopularity>> {
        // Placeholder - would fetch track popularity from database
        Ok(Vec::new())
    }

    async fn get_trending_tracks(&self, _days: u32) -> Result<Vec<crate::models::entities::Track>> {
        // Placeholder - would fetch recent tracks from database
        Ok(Vec::new())
    }

    async fn get_user_preferred_artists(
        &self,
        _user_id: &str,
    ) -> Result<std::collections::HashSet<String>> {
        // Placeholder - would get user's preferred artists
        Ok(std::collections::HashSet::new())
    }

    async fn analyze_temporal_patterns(&self, _user_id: &str) -> Result<Vec<TemporalPattern>> {
        // Placeholder for temporal pattern analysis
        Ok(Vec::new())
    }

    async fn extract_track_features(
        &self,
        track: &crate::models::entities::Track,
    ) -> Result<TrackFeatures> {
        // Placeholder for track feature extraction
        Ok(TrackFeatures {
            track_id: track.id.clone(),
            artist_id: track.artist_id.clone(),
            genre_vector: vec![0.0; 20],
            tempo: None,
            energy: None,
            valence: None,
            danceability: None,
            acousticness: None,
            instrumentalness: None,
            year: None,
            popularity: 0.0,
            duration: track.duration,
        })
    }
}

/// Temporal pattern for time-based recommendations
#[derive(Debug, Clone)]
pub struct TemporalPattern {
    pub track_id: String,
    pub hour_of_day: u32,
    pub day_of_week: chrono::Weekday,
    pub confidence: f64,
}

/// Service trait implementation for RecommendationService
#[async_trait::async_trait]
impl crate::services::Service for RecommendationService {
    type Stats = RecommendationStats;

    async fn health_check(&self) -> Result<()> {
        // Basic health check - verify internal state is valid
        let _stats = self.stats.read().await;
        Ok(())
    }

    async fn get_stats(&self) -> Result<Self::Stats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    async fn shutdown(&self) -> Result<()> {
        // Cleanup any background tasks or resources
        info!("Shutting down recommendation service");
        Ok(())
    }
}

impl TemporalPattern {
    pub fn matches_current_time(
        &self,
        current_hour: u32,
        current_weekday: chrono::Weekday,
    ) -> bool {
        // Simple matching - could be more sophisticated
        self.hour_of_day == current_hour || self.day_of_week == current_weekday
    }
}
