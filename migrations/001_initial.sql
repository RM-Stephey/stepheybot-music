-- Initial migration for StepheyBot Music
-- Creates the core database schema for music streaming, recommendations, and user data

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Users table - synchronized from Navidrome
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    navidrome_id TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    display_name TEXT,
    email TEXT,
    is_admin BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_seen_at DATETIME,
    listening_time_total INTEGER DEFAULT 0, -- Total listening time in seconds
    track_count_total INTEGER DEFAULT 0     -- Total tracks played
);

-- Artists table - music artists with MusicBrainz integration
CREATE TABLE artists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_name TEXT,
    musicbrainz_id TEXT UNIQUE,
    biography TEXT,
    country TEXT,
    formed_year INTEGER,
    disbanded_year INTEGER,
    artist_type TEXT, -- person, group, orchestra, choir, character, other
    gender TEXT,      -- male, female, other (for person type)
    image_url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    play_count INTEGER DEFAULT 0,
    last_played_at DATETIME
);

-- Albums table - music albums
CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    sort_title TEXT,
    artist_id TEXT NOT NULL,
    musicbrainz_id TEXT UNIQUE,
    release_date TEXT, -- ISO date format
    release_year INTEGER,
    album_type TEXT,   -- album, single, ep, compilation, soundtrack, etc.
    track_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0, -- Total duration in seconds
    genre TEXT,
    artwork_url TEXT,
    artwork_path TEXT, -- Local artwork file path
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    play_count INTEGER DEFAULT 0,
    last_played_at DATETIME,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE
);

-- Tracks table - individual songs
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    sort_title TEXT,
    artist_id TEXT NOT NULL,
    album_id TEXT,
    musicbrainz_id TEXT UNIQUE,
    track_number INTEGER,
    disc_number INTEGER DEFAULT 1,
    duration INTEGER, -- Duration in seconds
    file_path TEXT,   -- Path to the actual audio file
    file_size INTEGER, -- File size in bytes
    bitrate INTEGER,   -- Audio bitrate
    sample_rate INTEGER, -- Audio sample rate
    channels INTEGER,  -- Number of audio channels
    format TEXT,       -- Audio format (mp3, flac, etc.)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    play_count INTEGER DEFAULT 0,
    last_played_at DATETIME,
    love_count INTEGER DEFAULT 0, -- Number of users who loved this track
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE SET NULL
);

-- Genres table - music genres
CREATE TABLE genres (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    parent_genre_id INTEGER,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_genre_id) REFERENCES genres(id) ON DELETE SET NULL
);

-- Track genres - many-to-many relationship
CREATE TABLE track_genres (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id TEXT NOT NULL,
    genre_id INTEGER NOT NULL,
    weight REAL DEFAULT 1.0, -- Genre weight/relevance (0.0-1.0)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE,
    UNIQUE(track_id, genre_id)
);

-- Playlists table - user playlists
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    user_id TEXT NOT NULL,
    navidrome_id TEXT, -- Corresponding playlist ID in Navidrome
    is_public BOOLEAN DEFAULT FALSE,
    is_smart BOOLEAN DEFAULT FALSE, -- Auto-generated playlist
    smart_criteria TEXT, -- JSON criteria for smart playlists
    track_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0, -- Total duration in seconds
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played_at DATETIME,
    play_count INTEGER DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Playlist tracks - tracks in playlists
CREATE TABLE playlist_tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    added_by_user_id TEXT,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    FOREIGN KEY (added_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE(playlist_id, position)
);

-- Listening history - user play history
CREATE TABLE listening_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    play_duration INTEGER, -- How long the track was played (seconds)
    completion_percentage REAL, -- Percentage of track completed (0.0-1.0)
    source TEXT, -- navidrome, recommendation, playlist, etc.
    source_id TEXT, -- ID of the source (playlist_id, etc.)
    client_name TEXT, -- Client application name
    client_version TEXT,
    ip_address TEXT,
    user_agent TEXT,
    skip_reason TEXT, -- Why the track was skipped (if applicable)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- Recommendations table - generated recommendations
CREATE TABLE recommendations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    recommendation_type TEXT NOT NULL, -- collaborative, content_based, popularity, etc.
    score REAL NOT NULL, -- Recommendation confidence score (0.0-1.0)
    reason TEXT, -- Human-readable reason for recommendation
    metadata TEXT, -- JSON metadata about the recommendation
    is_consumed BOOLEAN DEFAULT FALSE, -- Has the user played this recommendation
    consumed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME, -- When this recommendation expires
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- User preferences - user settings and preferences
CREATE TABLE user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    preference_key TEXT NOT NULL,
    preference_value TEXT NOT NULL,
    preference_type TEXT DEFAULT 'string', -- string, integer, float, boolean, json
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, preference_key)
);

-- Artist relationships - related artists (similar, member of, etc.)
CREATE TABLE artist_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    artist_id TEXT NOT NULL,
    related_artist_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL, -- similar, member_of, collaboration, etc.
    strength REAL DEFAULT 1.0, -- Relationship strength (0.0-1.0)
    source TEXT, -- musicbrainz, lastfm, computed, etc.
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    FOREIGN KEY (related_artist_id) REFERENCES artists(id) ON DELETE CASCADE,
    UNIQUE(artist_id, related_artist_id, relationship_type)
);

-- User track ratings - user ratings and favorites
CREATE TABLE user_track_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5), -- 1-5 star rating
    is_loved BOOLEAN DEFAULT FALSE,
    is_banned BOOLEAN DEFAULT FALSE, -- User doesn't want to hear this track
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
    UNIQUE(user_id, track_id)
);

-- Download requests - tracks requested for download
CREATE TABLE download_requests (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    artist_name TEXT NOT NULL,
    track_title TEXT NOT NULL,
    album_title TEXT,
    status TEXT DEFAULT 'pending', -- pending, downloading, completed, failed
    source_url TEXT,
    error_message TEXT,
    requested_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME,
    track_id TEXT, -- Set when download is completed and track is added
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE SET NULL
);

-- Create indexes for performance
CREATE INDEX idx_users_navidrome_id ON users(navidrome_id);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);

CREATE INDEX idx_artists_name ON artists(name);
CREATE INDEX idx_artists_musicbrainz_id ON artists(musicbrainz_id);
CREATE INDEX idx_artists_play_count ON artists(play_count DESC);

CREATE INDEX idx_albums_title ON albums(title);
CREATE INDEX idx_albums_artist_id ON albums(artist_id);
CREATE INDEX idx_albums_release_year ON albums(release_year);
CREATE INDEX idx_albums_musicbrainz_id ON albums(musicbrainz_id);

CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_artist_id ON tracks(artist_id);
CREATE INDEX idx_tracks_album_id ON tracks(album_id);
CREATE INDEX idx_tracks_musicbrainz_id ON tracks(musicbrainz_id);
CREATE INDEX idx_tracks_play_count ON tracks(play_count DESC);
CREATE INDEX idx_tracks_file_path ON tracks(file_path);

CREATE INDEX idx_playlists_user_id ON playlists(user_id);
CREATE INDEX idx_playlists_name ON playlists(name);
CREATE INDEX idx_playlists_is_smart ON playlists(is_smart);

CREATE INDEX idx_playlist_tracks_playlist_id ON playlist_tracks(playlist_id);
CREATE INDEX idx_playlist_tracks_track_id ON playlist_tracks(track_id);
CREATE INDEX idx_playlist_tracks_position ON playlist_tracks(playlist_id, position);

CREATE INDEX idx_listening_history_user_id ON listening_history(user_id);
CREATE INDEX idx_listening_history_track_id ON listening_history(track_id);
CREATE INDEX idx_listening_history_played_at ON listening_history(played_at DESC);
CREATE INDEX idx_listening_history_user_played ON listening_history(user_id, played_at DESC);

CREATE INDEX idx_recommendations_user_id ON recommendations(user_id);
CREATE INDEX idx_recommendations_track_id ON recommendations(track_id);
CREATE INDEX idx_recommendations_score ON recommendations(score DESC);
CREATE INDEX idx_recommendations_created_at ON recommendations(created_at DESC);
CREATE INDEX idx_recommendations_expires_at ON recommendations(expires_at);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

CREATE INDEX idx_track_genres_track_id ON track_genres(track_id);
CREATE INDEX idx_track_genres_genre_id ON track_genres(genre_id);

CREATE INDEX idx_artist_relationships_artist_id ON artist_relationships(artist_id);
CREATE INDEX idx_artist_relationships_related ON artist_relationships(related_artist_id);
CREATE INDEX idx_artist_relationships_type ON artist_relationships(relationship_type);

CREATE INDEX idx_user_track_ratings_user_id ON user_track_ratings(user_id);
CREATE INDEX idx_user_track_ratings_track_id ON user_track_ratings(track_id);
CREATE INDEX idx_user_track_ratings_loved ON user_track_ratings(is_loved);
CREATE INDEX idx_user_track_ratings_banned ON user_track_ratings(is_banned);

CREATE INDEX idx_download_requests_user_id ON download_requests(user_id);
CREATE INDEX idx_download_requests_status ON download_requests(status);
CREATE INDEX idx_download_requests_requested_at ON download_requests(requested_at DESC);

-- Insert some default genres
INSERT INTO genres (name, description) VALUES
('Rock', 'Rock music genre'),
('Pop', 'Popular music genre'),
('Jazz', 'Jazz music genre'),
('Classical', 'Classical music genre'),
('Electronic', 'Electronic music genre'),
('Hip Hop', 'Hip hop music genre'),
('R&B', 'Rhythm and blues genre'),
('Country', 'Country music genre'),
('Folk', 'Folk music genre'),
('Blues', 'Blues music genre'),
('Reggae', 'Reggae music genre'),
('Punk', 'Punk rock genre'),
('Metal', 'Heavy metal genre'),
('Alternative', 'Alternative rock genre'),
('Indie', 'Independent music genre'),
('World', 'World music genre');

-- Create a view for user listening statistics
CREATE VIEW user_listening_stats AS
SELECT
    u.id as user_id,
    u.username,
    COUNT(lh.id) as total_plays,
    SUM(lh.play_duration) as total_listening_time,
    COUNT(DISTINCT lh.track_id) as unique_tracks_played,
    COUNT(DISTINCT t.artist_id) as unique_artists_played,
    AVG(lh.completion_percentage) as avg_completion_rate,
    MAX(lh.played_at) as last_played_at
FROM users u
LEFT JOIN listening_history lh ON u.id = lh.user_id
LEFT JOIN tracks t ON lh.track_id = t.id
GROUP BY u.id, u.username;

-- Create a view for track popularity
CREATE VIEW track_popularity AS
SELECT
    t.id as track_id,
    t.title,
    a.name as artist_name,
    al.title as album_title,
    COUNT(lh.id) as play_count,
    COUNT(DISTINCT lh.user_id) as unique_listeners,
    AVG(lh.completion_percentage) as avg_completion_rate,
    COUNT(utr.id) as rating_count,
    AVG(CAST(utr.rating AS REAL)) as avg_rating,
    SUM(CASE WHEN utr.is_loved THEN 1 ELSE 0 END) as love_count
FROM tracks t
JOIN artists a ON t.artist_id = a.id
LEFT JOIN albums al ON t.album_id = al.id
LEFT JOIN listening_history lh ON t.id = lh.track_id
LEFT JOIN user_track_ratings utr ON t.id = utr.track_id
GROUP BY t.id, t.title, a.name, al.title;

-- Triggers to update timestamps
CREATE TRIGGER update_users_timestamp
    AFTER UPDATE ON users
    FOR EACH ROW
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_artists_timestamp
    AFTER UPDATE ON artists
    FOR EACH ROW
BEGIN
    UPDATE artists SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_albums_timestamp
    AFTER UPDATE ON albums
    FOR EACH ROW
BEGIN
    UPDATE albums SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_tracks_timestamp
    AFTER UPDATE ON tracks
    FOR EACH ROW
BEGIN
    UPDATE tracks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_playlists_timestamp
    AFTER UPDATE ON playlists
    FOR EACH ROW
BEGIN
    UPDATE playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- Trigger to update playlist track count
CREATE TRIGGER update_playlist_track_count_insert
    AFTER INSERT ON playlist_tracks
    FOR EACH ROW
BEGIN
    UPDATE playlists
    SET track_count = (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = NEW.playlist_id),
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.playlist_id;
END;

CREATE TRIGGER update_playlist_track_count_delete
    AFTER DELETE ON playlist_tracks
    FOR EACH ROW
BEGIN
    UPDATE playlists
    SET track_count = (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = OLD.playlist_id),
        updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.playlist_id;
END;

-- Trigger to update play counts
CREATE TRIGGER update_play_counts
    AFTER INSERT ON listening_history
    FOR EACH ROW
    WHEN NEW.completion_percentage > 0.5  -- Only count as a play if >50% completed
BEGIN
    UPDATE tracks SET
        play_count = play_count + 1,
        last_played_at = NEW.played_at
    WHERE id = NEW.track_id;

    UPDATE artists SET
        play_count = play_count + 1,
        last_played_at = NEW.played_at
    WHERE id = (SELECT artist_id FROM tracks WHERE id = NEW.track_id);

    UPDATE albums SET
        play_count = play_count + 1,
        last_played_at = NEW.played_at
    WHERE id = (SELECT album_id FROM tracks WHERE id = NEW.track_id);

    UPDATE users SET
        listening_time_total = listening_time_total + COALESCE(NEW.play_duration, 0),
        track_count_total = track_count_total + 1,
        last_seen_at = NEW.played_at
    WHERE id = NEW.user_id;
END;
