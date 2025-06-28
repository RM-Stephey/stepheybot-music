# StepheyBot Music - User Profile System Implementation

## 🎵 Overview

This document outlines the comprehensive implementation strategy for adding multi-user support to StepheyBot Music, enabling personalized experiences, social features, and advanced recommendation capabilities while maintaining the existing high-performance architecture.

## 🏗️ System Architecture

### Current Foundation
- **Backend**: Rust-based API server with service-oriented architecture
- **Authentication**: Keycloak SSO integration (already configured)
- **Database**: SQLite with NVME caching optimization
- **Music Library**: 1,447 professionally organized tracks in Artist/Album/Track hierarchy
- **Frontend**: Svelte-based responsive web application

### Multi-User Enhancement Strategy
The user profile system extends the existing architecture without disrupting current functionality:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Keycloak SSO  │────│  Rust Backend    │────│  SQLite + Cache │
│                 │    │                  │    │                 │
│ • User Auth     │    │ • User Services  │    │ • User Tables   │
│ • JWT Tokens    │    │ • Recommendations│    │ • Music Library │
│ • Role Management│   │ • Integrations   │    │ • Preferences   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                    ┌───────────┼───────────┐
                    │           │           │
            ┌───────▼────┐ ┌────▼─────┐ ┌──▼──────┐
            │ListenBrainz│ │ Spotify  │ │Frontend │
            │Integration │ │ Playlists│ │Multi-User│
            └────────────┘ └──────────┘ └─────────┘
```

## 🗄️ Database Schema Design

### User Management Tables

```sql
-- Core user identity linked to Keycloak
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

-- External service integrations
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
```

### Music Interaction Tables

```sql
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

-- Playlist track associations
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
```

### Recommendation and Social Tables

```sql
-- User taste profiles for recommendations
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

-- Generated recommendations
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
```

### Performance Indexes

```sql
-- User and session indexes
CREATE INDEX idx_users_keycloak_id ON users(keycloak_id);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_last_active ON users(last_active);

-- Listening session indexes
CREATE INDEX idx_listening_sessions_user_id ON listening_sessions(user_id);
CREATE INDEX idx_listening_sessions_track_path ON listening_sessions(track_path);
CREATE INDEX idx_listening_sessions_started_at ON listening_sessions(started_at);
CREATE INDEX idx_listening_sessions_user_track ON listening_sessions(user_id, track_path);

-- Playlist indexes
CREATE INDEX idx_user_playlists_user_id ON user_playlists(user_id);
CREATE INDEX idx_user_playlists_public ON user_playlists(is_public);
CREATE INDEX idx_playlist_tracks_playlist_id ON playlist_tracks(playlist_id);

-- Recommendation indexes
CREATE INDEX idx_user_recommendations_user_id ON user_recommendations(user_id);
CREATE INDEX idx_user_recommendations_type ON user_recommendations(recommendation_type);
CREATE INDEX idx_user_recommendations_generated_at ON user_recommendations(generated_at);

-- Social indexes
CREATE INDEX idx_user_follows_follower ON user_follows(follower_id);
CREATE INDEX idx_user_follows_following ON user_follows(following_id);
```

## 🔐 Authentication & Authorization

### Keycloak Integration

The system leverages the existing Keycloak SSO configuration with enhanced user management:

```rust
// JWT Token validation middleware
pub struct AuthMiddleware {
    keycloak_config: KeycloakConfig,
    user_service: Arc<UserService>,
}

impl AuthMiddleware {
    pub async fn validate_token(&self, token: &str) -> Result<AuthenticatedUser> {
        // 1. Validate JWT signature and expiration
        let claims = jsonwebtoken::decode::<Claims>(token, &self.keycloak_config.public_key, &validation)?;
        
        // 2. Extract user information
        let keycloak_id = claims.claims.sub;
        let username = claims.claims.preferred_username;
        let email = claims.claims.email;
        
        // 3. Ensure user exists in our database (create if first login)
        let user = self.user_service.ensure_user_exists(keycloak_id, username, email).await?;
        
        Ok(AuthenticatedUser {
            id: user.id,
            keycloak_id,
            username,
            email,
            roles: claims.claims.realm_access.roles,
        })
    }
}
```

### Role-Based Access Control

```rust
#[derive(Debug, Clone)]
pub enum UserRole {
    Admin,      // Full system access
    User,       // Standard user access
    Guest,      // Read-only library access
}

pub struct AuthorizedRequest {
    pub user: AuthenticatedUser,
    pub required_role: UserRole,
}

// API endpoint protection
#[derive(Debug)]
pub struct RequireAuth {
    pub role: UserRole,
}

impl RequireAuth {
    pub fn admin() -> Self { Self { role: UserRole::Admin } }
    pub fn user() -> Self { Self { role: UserRole::User } }
    pub fn guest() -> Self { Self { role: UserRole::Guest } }
}
```

## 📡 API Endpoints

### User Management

```http
# Authentication
POST   /api/v1/auth/login              # Redirect to Keycloak
GET    /api/v1/auth/callback           # Handle Keycloak callback
POST   /api/v1/auth/logout             # Logout and invalidate session
GET    /api/v1/auth/profile            # Get current user profile

# User Profile Management
GET    /api/v1/user/profile            # Get user profile
PUT    /api/v1/user/profile            # Update user profile
GET    /api/v1/user/preferences        # Get user preferences
PUT    /api/v1/user/preferences        # Update user preferences
POST   /api/v1/user/avatar             # Upload avatar image
DELETE /api/v1/user/account            # Delete user account (GDPR)
GET    /api/v1/user/export             # Export user data (GDPR)
```

### Music Interaction

```http
# Listening Tracking
POST   /api/v1/listen/start            # Start listening session
POST   /api/v1/listen/progress         # Update listening progress
POST   /api/v1/listen/complete         # Complete listening session
GET    /api/v1/listen/history          # Get listening history
GET    /api/v1/listen/stats            # Get listening statistics

# Favorites and Ratings
GET    /api/v1/favorites               # Get user favorites
POST   /api/v1/favorites               # Add to favorites
DELETE /api/v1/favorites/{track_id}    # Remove from favorites
PUT    /api/v1/ratings/{track_id}      # Rate a track

# Playlist Management
GET    /api/v1/playlists               # Get user playlists
POST   /api/v1/playlists               # Create new playlist
GET    /api/v1/playlists/{id}          # Get playlist details
PUT    /api/v1/playlists/{id}          # Update playlist
DELETE /api/v1/playlists/{id}          # Delete playlist
POST   /api/v1/playlists/{id}/tracks   # Add tracks to playlist
DELETE /api/v1/playlists/{id}/tracks   # Remove tracks from playlist
```

### Recommendations

```http
# Recommendation Engine
GET    /api/v1/recommendations         # Get personalized recommendations
GET    /api/v1/recommendations/discover # Get discovery recommendations
POST   /api/v1/recommendations/feedback # Provide feedback on recommendations
GET    /api/v1/recommendations/similar/{track_id} # Get similar tracks
PUT    /api/v1/taste-profile           # Update taste profile
```

### Social Features

```http
# User Discovery
GET    /api/v1/users/search            # Search for users
GET    /api/v1/users/{username}        # Get public user profile
GET    /api/v1/users/{username}/playlists # Get public playlists

# Following System
GET    /api/v1/social/following        # Get users I'm following
GET    /api/v1/social/followers        # Get my followers
POST   /api/v1/social/follow/{user_id} # Follow a user
DELETE /api/v1/social/follow/{user_id} # Unfollow a user

# Activity Feed
GET    /api/v1/social/feed             # Get activity feed
GET    /api/v1/social/activity         # Get my activity
```

### External Integrations

```http
# ListenBrainz Integration
GET    /api/v1/integrations/listenbrainz/status    # Check connection status
POST   /api/v1/integrations/listenbrainz/connect   # Connect account
DELETE /api/v1/integrations/listenbrainz/disconnect # Disconnect account
POST   /api/v1/integrations/listenbrainz/sync      # Force sync
GET    /api/v1/integrations/listenbrainz/stats     # Get ListenBrainz stats

# Spotify Integration  
GET    /api/v1/integrations/spotify/auth           # Start OAuth flow
GET    /api/v1/integrations/spotify/callback       # Handle OAuth callback
GET    /api/v1/integrations/spotify/playlists      # Get Spotify playlists
POST   /api/v1/integrations/spotify/import         # Import playlists
GET    /api/v1/integrations/spotify/import/status  # Check import status
```

## 🔗 External Service Integrations

### ListenBrainz Integration

```rust
pub struct ListenBrainzService {
    client: reqwest::Client,
    base_url: String,
}

impl ListenBrainzService {
    pub async fn submit_listen(
        &self,
        user_token: &str,
        listen: &ListenData,
    ) -> Result<()> {
        let payload = json!({
            "listen_type": "single",
            "payload": [{
                "listened_at": listen.timestamp,
                "track_metadata": {
                    "artist_name": listen.artist,
                    "track_name": listen.track,
                    "release_name": listen.album,
                    "additional_info": {
                        "duration_ms": listen.duration_ms,
                        "listening_from": "StepheyBot Music"
                    }
                }
            }]
        });

        self.client
            .post(&format!("{}/1/submit-listens", self.base_url))
            .bearer_auth(user_token)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_user_listens(
        &self,
        username: &str,
        count: u32,
    ) -> Result<Vec<Listen>> {
        let response = self.client
            .get(&format!("{}/1/user/{}/listens", self.base_url, username))
            .query(&[("count", count)])
            .send()
            .await?;

        let data: ListenBrainzResponse = response.json().await?;
        Ok(data.payload.listens)
    }
}
```

### Spotify Integration

```rust
pub struct SpotifyService {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl SpotifyService {
    pub async fn get_authorization_url(&self, user_id: i64) -> Result<String> {
        let state = format!("user:{}", user_id);
        let scope = "playlist-read-private playlist-read-collaborative user-read-private";
        
        Ok(format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(scope),
            urlencoding::encode(&state)
        ))
    }

    pub async fn exchange_code_for_token(&self, code: &str) -> Result<SpotifyTokens> {
        let payload = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self.client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&payload)
            .send()
            .await?;

        let tokens: SpotifyTokens = response.json().await?;
        Ok(tokens)
    }

    pub async fn import_playlists(
        &self,
        access_token: &str,
        user_id: i64,
    ) -> Result<Vec<ImportedPlaylist>> {
        // Get user's playlists
        let playlists = self.get_user_playlists(access_token).await?;
        let mut imported = Vec::new();

        for playlist in playlists {
            let tracks = self.get_playlist_tracks(access_token, &playlist.id).await?;
            
            // Match Spotify tracks to local library
            let matched_tracks = self.match_tracks_to_library(&tracks).await?;
            
            imported.push(ImportedPlaylist {
                name: playlist.name,
                description: playlist.description,
                tracks: matched_tracks,
                total_tracks: tracks.len(),
                matched_tracks: matched_tracks.len(),
            });
        }

        Ok(imported)
    }
}
```

## 🤖 Recommendation System

### Algorithm Architecture

```rust
pub trait RecommendationAlgorithm: Send + Sync {
    async fn generate_recommendations(
        &self,
        user_id: i64,
        count: usize,
        context: &RecommendationContext,
    ) -> Result<Vec<Recommendation>>;
    
    fn algorithm_name(&self) -> &'static str;
    fn weight(&self) -> f64; // For hybrid approaches
}

pub struct HybridRecommendationEngine {
    algorithms: Vec<Box<dyn RecommendationAlgorithm>>,
    user_service: Arc<UserService>,
    library_service: Arc<LibraryService>,
}

impl HybridRecommendationEngine {
    pub async fn generate_recommendations(
        &self,
        user_id: i64,
        count: usize,
    ) -> Result<Vec<Recommendation>> {
        let context = self.build_context(user_id).await?;
        let mut all_recommendations = Vec::new();

        // Run all algorithms
        for algorithm in &self.algorithms {
            let recs = algorithm
                .generate_recommendations(user_id, count * 2, &context)
                .await?;
            
            // Weight and tag recommendations by algorithm
            for mut rec in recs {
                rec.confidence_score *= algorithm.weight();
                rec.source_algorithm = algorithm.algorithm_name().to_string();
                all_recommendations.push(rec);
            }
        }

        // Deduplicate and sort by confidence
        self.deduplicate_and_rank(all_recommendations, count)
    }
}
```

### Collaborative Filtering

```rust
pub struct CollaborativeFilteringAlgorithm {
    database: Arc<Database>,
}

impl RecommendationAlgorithm for CollaborativeFilteringAlgorithm {
    async fn generate_recommendations(
        &self,
        user_id: i64,
        count: usize,
        context: &RecommendationContext,
    ) -> Result<Vec<Recommendation>> {
        // Find similar users based on listening history
        let similar_users = self.find_similar_users(user_id, 20).await?;
        
        // Get tracks loved by similar users but not yet heard by target user
        let mut recommendations = Vec::new();
        
        for similar_user in similar_users {
            let their_favorites = self.get_user_favorites(similar_user.id).await?;
            
            for track in their_favorites {
                if !context.user_history.contains(&track.path) {
                    recommendations.push(Recommendation {
                        track_path: track.path,
                        confidence_score: similar_user.similarity * track.rating,
                        reason: format!("Users with similar taste rated this highly"),
                        source_algorithm: self.algorithm_name().to_string(),
                    });
                }
            }
        }

        Ok(recommendations.into_iter().take(count).collect())
    }

    fn algorithm_name(&self) -> &'static str {
        "collaborative_filtering"
    }

    fn weight(&self) -> f64 {
        0.4 // 40% weight in hybrid system
    }
}
```

### Content-Based Filtering

```rust
pub struct ContentBasedAlgorithm {
    database: Arc<Database>,
}

impl RecommendationAlgorithm for ContentBasedAlgorithm {
    async fn generate_recommendations(
        &self,
        user_id: i64,
        count: usize,
        context: &RecommendationContext,
    ) -> Result<Vec<Recommendation>> {
        // Analyze user's favorite tracks for audio features
        let user_taste_profile = self.analyze_user_taste(user_id).await?;
        
        // Find tracks with similar audio characteristics
        let candidate_tracks = self.find_similar_audio_features(&user_taste_profile).await?;
        
        let mut recommendations = Vec::new();
        
        for track in candidate_tracks {
            if !context.user_history.contains(&track.file_path) {
                let similarity = self.calculate_audio_similarity(&user_taste_profile, &track);
                
                recommendations.push(Recommendation {
                    track_path: track.file_path,
                    confidence_score: similarity,
                    reason: format!("Similar audio characteristics to your favorites"),
                    source_algorithm: self.algorithm_name().to_string(),
                });
            }
        }

        Ok(recommendations.into_iter().take(count).collect())
    }

    fn algorithm_name(&self) -> &'static str {
        "content_based"
    }

    fn weight(&self) -> f64 {
        0.3 // 30% weight in hybrid system
    }
}
```

## 🚀 Implementation Phases

### Phase 1: Foundation (Weeks 1-3)

**Database Setup**
- [ ] Extend SQLite schema with user tables
- [ ] Create database migration system
- [ ] Implement database seeding for development
- [ ] Add comprehensive indexes for performance

**Authentication Infrastructure**
- [ ] Implement JWT validation middleware
- [ ] Create user management service
- [ ] Add user session handling
- [ ] Implement role-based access control

**Basic User Management**
- [ ] User registration/login flow
- [ ] User profile CRUD operations
- [ ] User preferences management
- [ ] Basic user settings page

### Phase 2: Core Features (Weeks 4-6)

**Listening Tracking**
- [ ] Implement listening session tracking
- [ ] Add scrobbling to local database
- [ ] Create listening history endpoints
- [ ] Build listening statistics dashboard

**Playlist Management**
- [ ] User playlist CRUD operations
- [ ] Playlist sharing and privacy controls
- [ ] Collaborative playlist features
- [ ] Playlist import/export functionality

**Favorites and Ratings**
- [ ] Track favorites system
- [ ] 5-star rating system
- [ ] User library management
- [ ] Personal music dashboard

### Phase 3: External Integrations (Weeks 7-9)

**ListenBrainz Integration**
- [ ] OAuth flow for ListenBrainz
- [ ] Real-time scrobbling
- [ ] Historical data import
- [ ] Sync status and error handling

**Spotify Integration**
- [ ] Spotify OAuth implementation
- [ ] Playlist import wizard
- [ ] Track matching algorithm
- [ ] Import progress tracking

**Data Synchronization**
- [ ] Background sync jobs
- [ ] Conflict resolution
- [ ] Error handling and retry logic
- [ ] Sync status monitoring

### Phase 4: Recommendations (Weeks 10-12)

**Recommendation Engine**
- [ ] Algorithm framework implementation
- [ ] Collaborative filtering algorithm
- [ ] Content-based filtering algorithm
- [ ] Hybrid recommendation system

**Taste Profile System**
- [ ] Automatic taste profile generation
- [ ] Manual preference configuration
- [ ] Profile evolution over time
- [ ] Taste similarity calculations

**Recommendation UI**
- [ ] Personalized discovery page
- [ ] Recommendation explanations
- [ ] Feedback collection system
- [ ] Recommendation tuning interface

### Phase 5: Social Features (Weeks 13-15)

**User Discovery**
- [ ] User search functionality
- [ ] Public profile pages
- [ ] Following/followers system
- [ ] Privacy controls

**Social Interactions**
- [ ] Activity feed
- [ ] Shared playlists
- [ ] Music recommendations between users
- [ ] Social listening statistics

**Community Features**
- [ ] User-generated content
- [ ] Reviews and comments
- [ ] Community playlists
- [ ] Social discovery algorithms

## 📱 Frontend Implementation

### User Interface Enhancements

**Navigation Updates**
```svelte
<!-- UserNavigation.svelte -->
<script>
  import { user, isAuthenticated } from '$lib/stores/auth';
  import UserAvatar from './UserAvatar.svelte';
  import UserMenu from './UserMenu.svelte';
</script>

{#if $isAuthenticated}
  <div class="user-nav">
    <UserAvatar user={$user} size="small" />
    <span class="username">{$user.display_name || $user.username}</span>
    <UserMenu />
  </div>
{:else}
  <a href="/auth/login" class="login-btn neon-button">
    Login
  </a>
{/if}
```

**Personal Dashboard**
```svelte
<!-- PersonalDashboard.svelte -->
<script>
  import ListeningStats from './ListeningStats.svelte';
  import RecommendationFeed from './RecommendationFeed.svelte';
  import RecentActivity from './RecentActivity.svelte';
  import QuickActions from './QuickActions.svelte';
</script>

<div class="dashboard-grid">
  <div class="stats-section">
    <ListeningStats />
  </div>
  
  <div class="recommendations-section">
    <h2>Discover New Music</h2>
    <RecommendationFeed />
  </div>
  
  <div class="activity-section">
    <RecentActivity />
  </div>
  
  <div class="actions-section">
    <QuickActions />
  </div>
</div>

<style>
  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    grid-template-rows: auto auto;
    gap: 2rem;
    padding: 2rem;
  }
  
  @media (max-width: 768px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
      grid-template-rows: auto;
    }
  }
</style>
```

### Mobile Responsiveness

**Touch-Friendly Controls**
```css
/* Mobile-first playlist management */
.playlist-item {
  padding: 1rem;
  border-radius: 0.5rem;
  margin-bottom: 0.5rem;
  background: var(--surface-color);
  border: 1px solid var(--neon-pink);
  transition: all 0.3s ease;
}

.playlist-item:hover,
.playlist-item:focus {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--neon-glow);
}

@media (max-width: 768px) {
  .playlist-item {
    padding: 1.5rem;
    margin-bottom: 1rem;
  }
  
  .playlist-controls {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-top: 1rem;
  }
  
  .playlist-controls button {
    flex: 1;
    padding: 0.75rem;
    font-size: 1rem;
  }
}
```

## 🔒 Security Implementation

### Authentication Security

```rust
// JWT Token validation with security checks
pub struct JwtValidator {
    keycloak_public_key: DecodingKey,
    validation: Validation,
}

impl JwtValidator {
    pub fn new(keycloak_public_key: &str) -> Result<Self> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["stepheybot-music"]);