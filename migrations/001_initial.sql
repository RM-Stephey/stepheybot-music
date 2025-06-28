-- Migration: User Profile System Foundation
-- Creates all tables and indexes needed for multi-user support in StepheyBot Music
-- Compatible with SQLite and optimized for performance

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- ============================================================================
-- CORE USER MANAGEMENT TABLES
-- ============================================================================

-- Core user identity linked to Keycloak SSO
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    keycloak_id TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    last_active INTEGER DEFAULT (strftime('%s', 'now')),
    is_active BOOLEAN DEFAULT 1,
    preferences_json TEXT DEFAULT '{}' -- JSON blob for user preferences
);

-- User profile information and settings
CREATE TABLE user_profiles (
    user_id INTEGER PRIMARY KEY,
    bio TEXT,
    avatar_url TEXT,
    location TEXT,
    website TEXT,
    privacy_level INTEGER DEFAULT 1, -- 0=private, 1=friends, 2=public
    share_listening_history BOOLEAN DEFAULT 0,
    share_playlists BOOLEAN DEFAULT 1,
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- External service integrations (ListenBrainz, Spotify, etc.)
CREATE TABLE user_integrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    service_name TEXT NOT NULL, -- 'listenbrainz', 'spotify', etc.
    service_user_id TEXT, -- External service username/ID
    api_token_encrypted TEXT, -- Encrypted API token/credentials
    refresh_token_encrypted TEXT, -- For OAuth services
    enabled BOOLEAN DEFAULT 1,
    last_sync INTEGER,
    sync_settings_json TEXT DEFAULT '{}',
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    UNIQUE(user_id, service_name)
);

-- ============================================================================
-- MUSIC INTERACTION TABLES
-- ============================================================================

-- User listening sessions and scrobbles
CREATE TABLE listening_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    track_path TEXT NOT NULL, -- References tracks.file_path
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    duration_played INTEGER, -- Seconds actually played
    completed BOOLEAN DEFAULT 0, -- Did user listen to >80%?
    source TEXT, -- 'web', 'mobile', 'api'
    scrobbled_listenbrainz BOOLEAN DEFAULT 0,
    client_info_json TEXT DEFAULT '{}', -- User agent, app version, etc.
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- User favorites and ratings
CREATE TABLE user_favorites (
    user_id INTEGER NOT NULL,
    track_path TEXT NOT NULL,
    rating INTEGER CHECK(rating >= 1 AND rating <= 5), -- 1-5 star rating
    favorited_at INTEGER DEFAULT (strftime('%s', 'now')),
    notes TEXT, -- User's personal notes about the track
    PRIMARY KEY (user_id, track_path),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- User-created playlists
CREATE TABLE user_playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT 0,
    is_collaborative BOOLEAN DEFAULT 0,
    cover_image_url TEXT,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),
    track_count INTEGER DEFAULT 0,
    total_duration INTEGER DEFAULT 0, -- Total duration in seconds
    source_service TEXT, -- 'local', 'spotify_import', etc.
    external_id TEXT, -- Original ID from external service
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Playlist track associations with position tracking
CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL,
    track_path TEXT NOT NULL,
    position INTEGER NOT NULL,
    added_at INTEGER DEFAULT (strftime('%s', 'now')),
    added_by_user_id INTEGER, -- For collaborative playlists
    PRIMARY KEY (playlist_id, position),
    FOREIGN KEY (playlist_id) REFERENCES user_playlists (id) ON DELETE CASCADE,
    FOREIGN KEY (added_by_user_id) REFERENCES users (id) ON DELETE SET NULL
);

-- ============================================================================
-- RECOMMENDATION AND SOCIAL TABLES
-- ============================================================================

-- User taste profiles for personalized recommendations
CREATE TABLE user_taste_profiles (
    user_id INTEGER PRIMARY KEY,
    favorite_genres_json TEXT DEFAULT '[]', -- JSON array of genre preferences
    favorite_artists_json TEXT DEFAULT '[]', -- JSON array of artist preferences
    audio_features_json TEXT DEFAULT '{}', -- Preferred audio characteristics
    listening_patterns_json TEXT DEFAULT '{}', -- Time-based listening patterns
    recommendation_settings_json TEXT DEFAULT '{}', -- User's recommendation preferences
    last_updated INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Generated recommendations with tracking
CREATE TABLE user_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    track_path TEXT NOT NULL,
    recommendation_type TEXT NOT NULL, -- 'similar_tracks', 'discover_weekly', etc.
    confidence_score REAL DEFAULT 0.0, -- 0.0 to 1.0
    reason_json TEXT DEFAULT '{}', -- Why this was recommended
    generated_at INTEGER DEFAULT (strftime('%s', 'now')),
    viewed_at INTEGER,
    clicked_at INTEGER,
    dismissed_at INTEGER,
    feedback_rating INTEGER, -- User feedback: -1, 0, 1
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- User similarity matrix for collaborative filtering
CREATE TABLE user_similarity (
    user_id_1 INTEGER NOT NULL,
    user_id_2 INTEGER NOT NULL,
    similarity_score REAL NOT NULL, -- 0.0 to 1.0
    based_on TEXT NOT NULL, -- 'listening_history', 'favorites', 'playlists'
    calculated_at INTEGER DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (user_id_1, user_id_2),
    FOREIGN KEY (user_id_1) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id_2) REFERENCES users (id) ON DELETE CASCADE,
    CHECK (user_id_1 < user_id_2) -- Ensure consistent ordering
);

-- Social following relationships
CREATE TABLE user_follows (
    follower_id INTEGER NOT NULL,
    following_id INTEGER NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (follower_id, following_id),
    FOREIGN KEY (follower_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (following_id) REFERENCES users (id) ON DELETE CASCADE,
    CHECK (follower_id != following_id)
);

-- ============================================================================
-- PERFORMANCE INDEXES
-- ============================================================================

-- User and identity indexes
CREATE INDEX idx_users_keycloak_id ON users(keycloak_id);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_last_active ON users(last_active);
CREATE INDEX idx_users_is_active ON users(is_active);

-- User profile indexes
CREATE INDEX idx_user_profiles_privacy_level ON user_profiles(privacy_level);
CREATE INDEX idx_user_profiles_updated_at ON user_profiles(updated_at);

-- Integration indexes
CREATE INDEX idx_user_integrations_user_id ON user_integrations(user_id);
CREATE INDEX idx_user_integrations_service_name ON user_integrations(service_name);
CREATE INDEX idx_user_integrations_enabled ON user_integrations(enabled);
CREATE INDEX idx_user_integrations_last_sync ON user_integrations(last_sync);

-- Listening session indexes for performance
CREATE INDEX idx_listening_sessions_user_id ON listening_sessions(user_id);
CREATE INDEX idx_listening_sessions_track_path ON listening_sessions(track_path);
CREATE INDEX idx_listening_sessions_started_at ON listening_sessions(started_at);
CREATE INDEX idx_listening_sessions_user_track ON listening_sessions(user_id, track_path);
CREATE INDEX idx_listening_sessions_completed ON listening_sessions(completed);
CREATE INDEX idx_listening_sessions_source ON listening_sessions(source);

-- Favorites indexes
CREATE INDEX idx_user_favorites_user_id ON user_favorites(user_id);
CREATE INDEX idx_user_favorites_track_path ON user_favorites(track_path);
CREATE INDEX idx_user_favorites_rating ON user_favorites(rating);
CREATE INDEX idx_user_favorites_favorited_at ON user_favorites(favorited_at);

-- Playlist indexes
CREATE INDEX idx_user_playlists_user_id ON user_playlists(user_id);
CREATE INDEX idx_user_playlists_is_public ON user_playlists(is_public);
CREATE INDEX idx_user_playlists_is_collaborative ON user_playlists(is_collaborative);
CREATE INDEX idx_user_playlists_created_at ON user_playlists(created_at);
CREATE INDEX idx_user_playlists_source_service ON user_playlists(source_service);

-- Playlist tracks indexes
CREATE INDEX idx_playlist_tracks_playlist_id ON playlist_tracks(playlist_id);
CREATE INDEX idx_playlist_tracks_track_path ON playlist_tracks(track_path);
CREATE INDEX idx_playlist_tracks_added_by_user_id ON playlist_tracks(added_by_user_id);
CREATE INDEX idx_playlist_tracks_added_at ON playlist_tracks(added_at);

-- Recommendation indexes
CREATE INDEX idx_user_recommendations_user_id ON user_recommendations(user_id);
CREATE INDEX idx_user_recommendations_track_path ON user_recommendations(track_path);
CREATE INDEX idx_user_recommendations_type ON user_recommendations(recommendation_type);
CREATE INDEX idx_user_recommendations_generated_at ON user_recommendations(generated_at);
CREATE INDEX idx_user_recommendations_confidence_score ON user_recommendations(confidence_score);
CREATE INDEX idx_user_recommendations_feedback_rating ON user_recommendations(feedback_rating);

-- Taste profile indexes
CREATE INDEX idx_user_taste_profiles_last_updated ON user_taste_profiles(last_updated);

-- Social indexes
CREATE INDEX idx_user_follows_follower ON user_follows(follower_id);
CREATE INDEX idx_user_follows_following ON user_follows(following_id);
CREATE INDEX idx_user_follows_created_at ON user_follows(created_at);

-- Similarity indexes
CREATE INDEX idx_user_similarity_user_1 ON user_similarity(user_id_1);
CREATE INDEX idx_user_similarity_user_2 ON user_similarity(user_id_2);
CREATE INDEX idx_user_similarity_score ON user_similarity(similarity_score);
CREATE INDEX idx_user_similarity_calculated_at ON user_similarity(calculated_at);

-- ============================================================================
-- VIEWS FOR COMMON QUERIES
-- ============================================================================

-- View for user profile information with stats
CREATE VIEW v_user_profiles_with_stats AS
SELECT
    u.id,
    u.keycloak_id,
    u.username,
    u.email,
    u.display_name,
    u.created_at,
    u.last_active,
    u.is_active,
    up.bio,
    up.avatar_url,
    up.location,
    up.website,
    up.privacy_level,
    up.share_listening_history,
    up.share_playlists,
    (SELECT COUNT(*) FROM user_playlists WHERE user_id = u.id) as playlist_count,
    (SELECT COUNT(*) FROM user_favorites WHERE user_id = u.id) as favorite_count,
    (SELECT COUNT(*) FROM listening_sessions WHERE user_id = u.id) as total_listens,
    (SELECT COUNT(*) FROM user_follows WHERE follower_id = u.id) as following_count,
    (SELECT COUNT(*) FROM user_follows WHERE following_id = u.id) as follower_count
FROM users u
LEFT JOIN user_profiles up ON u.id = up.user_id;

-- View for public playlists with user information
CREATE VIEW v_public_playlists AS
SELECT
    p.id,
    p.user_id,
    u.username,
    u.display_name,
    p.name,
    p.description,
    p.cover_image_url,
    p.created_at,
    p.updated_at,
    p.track_count,
    p.total_duration,
    p.is_collaborative
FROM user_playlists p
JOIN users u ON p.user_id = u.id
WHERE p.is_public = 1;

-- View for listening statistics
CREATE VIEW v_user_listening_stats AS
SELECT
    user_id,
    COUNT(*) as total_sessions,
    SUM(duration_played) as total_listening_time,
    COUNT(DISTINCT track_path) as unique_tracks_played,
    COUNT(CASE WHEN completed = 1 THEN 1 END) as completed_sessions,
    AVG(duration_played) as avg_session_duration,
    MAX(started_at) as last_listen_time
FROM listening_sessions
GROUP BY user_id;

-- ============================================================================
-- TRIGGERS FOR DATA CONSISTENCY
-- ============================================================================

-- Update playlist track count when tracks are added/removed
CREATE TRIGGER tr_playlist_tracks_insert
    AFTER INSERT ON playlist_tracks
BEGIN
    UPDATE user_playlists
    SET track_count = track_count + 1,
        updated_at = strftime('%s', 'now')
    WHERE id = NEW.playlist_id;
END;

CREATE TRIGGER tr_playlist_tracks_delete
    AFTER DELETE ON playlist_tracks
BEGIN
    UPDATE user_playlists
    SET track_count = track_count - 1,
        updated_at = strftime('%s', 'now')
    WHERE id = OLD.playlist_id;
END;

-- Update user last_active when they perform actions
CREATE TRIGGER tr_update_user_last_active_listening
    AFTER INSERT ON listening_sessions
BEGIN
    UPDATE users
    SET last_active = strftime('%s', 'now')
    WHERE id = NEW.user_id;
END;

CREATE TRIGGER tr_update_user_last_active_favorites
    AFTER INSERT ON user_favorites
BEGIN
    UPDATE users
    SET last_active = strftime('%s', 'now')
    WHERE id = NEW.user_id;
END;

CREATE TRIGGER tr_update_user_last_active_playlists
    AFTER INSERT ON user_playlists
BEGIN
    UPDATE users
    SET last_active = strftime('%s', 'now')
    WHERE id = NEW.user_id;
END;

-- Update user profile updated_at timestamp
CREATE TRIGGER tr_user_profiles_updated_at
    AFTER UPDATE ON user_profiles
BEGIN
    UPDATE user_profiles
    SET updated_at = strftime('%s', 'now')
    WHERE user_id = NEW.user_id;
END;

-- ============================================================================
-- INITIAL DATA AND CONFIGURATION
-- ============================================================================

-- Insert default admin user (will be updated with real Keycloak data)
INSERT INTO users (keycloak_id, username, email, display_name, is_active)
VALUES ('admin-placeholder', 'admin', 'admin@stepheybot.dev', 'StepheyBot Admin', 1);

-- Create default profile for admin user
INSERT INTO user_profiles (user_id, bio, privacy_level, share_listening_history, share_playlists)
VALUES (1, 'StepheyBot Music System Administrator', 2, 1, 1);

-- Create default taste profile for admin user
INSERT INTO user_taste_profiles (user_id, favorite_genres_json, favorite_artists_json)
VALUES (1, '["Electronic", "Trance", "Progressive House"]', '["Armin van Buuren"]');

-- ============================================================================
-- MIGRATION COMPLETION
-- ============================================================================

-- Migration completed successfully
-- SQLx will automatically track this migration in _sqlx_migrations table
