# 🏗️ StepheyBot Music - System Architecture

> **Comprehensive architecture documentation for StepheyBot Music**

## 📋 Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Component Architecture](#component-architecture)
- [Data Architecture](#data-architecture)
- [API Architecture](#api-architecture)
- [Deployment Architecture](#deployment-architecture)
- [Integration Architecture](#integration-architecture)
- [Security Architecture](#security-architecture)
- [Performance Architecture](#performance-architecture)
- [Scalability Considerations](#scalability-considerations)

## Overview

StepheyBot Music is designed as a microservice-oriented, containerized music recommendation platform that prioritizes performance, privacy, and extensibility. The architecture follows modern cloud-native principles while maintaining the ability to run on single-node deployments.

### Design Principles

- **Privacy First**: All data processing occurs locally, no external data sharing
- **Performance Oriented**: Sub-millisecond response times for core operations
- **Modular Design**: Clean separation of concerns with well-defined interfaces
- **Container Native**: Designed for containerized deployment from the ground up
- **Type Safety**: Leverages Rust's type system for runtime reliability
- **Async First**: Non-blocking operations throughout the stack

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    StepheyBot Music Platform                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────┐  │
│  │   Web Client    │    │  Mobile Client  │    │  API Client │  │
│  │  (React/Vue)    │    │    (Flutter)    │    │   (Rust)    │  │
│  └─────────────────┘    └─────────────────┘    └─────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                        API Gateway                              │
│                    (Nginx/Traefik)                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              StepheyBot Music Core                      │    │
│  │                  (Rust + Axum)                         │    │
│  │                                                         │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │    │
│  │  │Recommendation│  │   Library   │  │    User     │    │    │
│  │  │   Engine     │  │   Service   │  │   Service   │    │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘    │    │
│  │                                                         │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │    │
│  │  │  Playlist   │  │    Sync     │  │   Audio     │    │    │
│  │  │  Service    │  │   Service   │  │  Analysis   │    │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   SQLite    │  │    Redis    │  │    Cache    │            │
│  │  Database   │  │   Cache     │  │   Layer     │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │  Navidrome  │  │    Lidarr   │  │ MusicBrainz │            │
│  │   Server    │  │   Client    │  │   Client    │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

### Architecture Layers

#### 1. **Presentation Layer**
- **Web Client**: React/Vue SPA with neon theme
- **Mobile Client**: Flutter cross-platform application
- **API Client**: Rust SDK for programmatic access

#### 2. **API Gateway Layer**
- **Load Balancing**: Request distribution across instances
- **Rate Limiting**: DDoS protection and fair usage
- **SSL Termination**: HTTPS encryption handling
- **Request Routing**: Path-based routing to services

#### 3. **Application Layer**
- **Core Services**: Business logic and orchestration
- **Recommendation Engine**: AI-powered music suggestions
- **Library Management**: Music metadata and organization
- **User Management**: Authentication and preferences

#### 4. **Data Layer**
- **Primary Database**: SQLite for relational data
- **Cache Layer**: Redis for session and computation caching
- **File Storage**: Local filesystem for music files

#### 5. **Integration Layer**
- **Music Servers**: Navidrome integration
- **Acquisition**: Lidarr for music downloads
- **Metadata**: MusicBrainz for enrichment

## Component Architecture

### Core Components

#### 1. **HTTP Server (Axum)**
```rust
// Simplified component structure
struct StepheyBotServer {
    router: Router,
    state: AppState,
    middleware: MiddlewareStack,
}

struct AppState {
    database: Arc<SqlitePool>,
    recommendation_engine: Arc<RecommendationService>,
    library_service: Arc<LibraryService>,
    cache: Arc<CacheService>,
}
```

**Responsibilities**:
- HTTP request/response handling
- Middleware orchestration (logging, CORS, compression)
- Route management and path matching
- Error handling and response formatting

#### 2. **Recommendation Engine**
```rust
struct RecommendationService {
    algorithms: Vec<Box<dyn RecommendationAlgorithm>>,
    cache: Arc<RwLock<HashMap<String, CachedRecommendations>>>,
    config: RecommendationConfig,
}

trait RecommendationAlgorithm {
    async fn generate(&self, user_id: &str, context: &Context) -> Result<Vec<Recommendation>>;
    fn algorithm_type(&self) -> AlgorithmType;
    fn weight(&self) -> f64;
}
```

**Algorithms**:
- **Collaborative Filtering**: User similarity-based recommendations
- **Content-Based**: Audio feature matching
- **Popularity-Based**: Trending and popular tracks
- **Discovery**: Hidden gems algorithm
- **Temporal**: Time-based preference analysis

#### 3. **Database Layer (SQLx + SQLite)**
```rust
struct DatabaseService {
    pool: SqlitePool,
    migrations: MigrationRunner,
    query_cache: QueryCache,
}

// Schema design
struct Schema {
    users: UsersTable,
    tracks: TracksTable,
    albums: AlbumsTable,
    artists: ArtistsTable,
    listening_history: ListeningHistoryTable,
    recommendations: RecommendationsTable,
}
```

**Features**:
- Connection pooling with configurable limits
- Automatic migration management
- Query optimization with prepared statements
- Transaction management for data consistency

#### 4. **External Clients**
```rust
struct NavidromeClient {
    client: reqwest::Client,
    base_url: String,
    auth: AuthenticationManager,
}

struct LidarrClient {
    client: reqwest::Client,
    api_key: String,
    rate_limiter: RateLimiter,
}
```

**Integration Points**:
- Navidrome API for music streaming
- Lidarr API for music acquisition
- MusicBrainz API for metadata enrichment
- ListenBrainz API for scrobbling (optional)

## Data Architecture

### Database Schema

#### Entity Relationship Diagram (Text)
```
Users ||--o{ ListeningHistory
Users ||--o{ Recommendations
Artists ||--o{ Albums
Artists ||--o{ Tracks
Albums ||--o{ Tracks
Tracks ||--o{ ListeningHistory
Tracks ||--o{ Recommendations
```

#### Core Tables

**Users Table**:
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT,
    created_at TEXT NOT NULL,
    last_active TEXT,
    preferences JSON  -- User preferences and settings
);
```

**Tracks Table**:
```sql
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    artist_id TEXT NOT NULL REFERENCES artists(id),
    album_id TEXT NOT NULL REFERENCES albums(id),
    duration_seconds INTEGER,
    -- Audio Features
    energy REAL,           -- 0.0-1.0 energy level
    valence REAL,          -- 0.0-1.0 positivity
    danceability REAL,     -- 0.0-1.0 danceability
    acousticness REAL,     -- 0.0-1.0 acoustic vs electric
    instrumentalness REAL, -- 0.0-1.0 instrumental content
    bpm INTEGER,           -- Beats per minute
    -- Metadata
    genre TEXT,
    play_count INTEGER DEFAULT 0,
    average_rating REAL,
    created_at TEXT NOT NULL
);
```

**Listening History Table**:
```sql
CREATE TABLE listening_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    track_id TEXT NOT NULL REFERENCES tracks(id),
    played_at TEXT NOT NULL,
    duration_played INTEGER,    -- Seconds played
    completed BOOLEAN DEFAULT FALSE,
    skip_reason TEXT,          -- Why was it skipped?
    context JSON               -- Playlist, radio, etc.
);
```

### Data Flow Architecture

#### Write Path
```
User Action → API Endpoint → Service Layer → Database Transaction → Cache Invalidation
```

#### Read Path
```
API Request → Cache Check → Database Query → Data Transform → Cache Update → Response
```

#### Recommendation Pipeline
```
User Request → History Analysis → Algorithm Execution → Score Aggregation → Cache Storage → Response
```

## API Architecture

### RESTful Design

#### Resource Organization
```
/api/v1/
├── users/
│   ├── {user_id}/
│   │   ├── history
│   │   ├── preferences
│   │   └── statistics
├── recommendations/
│   ├── {user_id}
│   ├── trending
│   └── discover
├── library/
│   ├── search
│   ├── stats
│   └── metadata
├── playlists/
│   ├── generate
│   └── {playlist_id}
└── admin/
    ├── system
    ├── database
    └── analytics
```

#### Response Standards
```rust
#[derive(Serialize)]
struct ApiResponse<T> {
    status: String,
    data: Option<T>,
    error: Option<ApiError>,
    meta: ResponseMetadata,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct ResponseMetadata {
    version: String,
    request_id: String,
    processing_time_ms: u64,
    cache_hit: bool,
}
```

### WebSocket Architecture (Planned)

#### Real-time Events
```rust
enum WebSocketEvent {
    PlaybackUpdate { track_id: String, position: u64 },
    RecommendationUpdate { user_id: String, recommendations: Vec<Recommendation> },
    LibraryUpdate { event_type: LibraryEventType, metadata: serde_json::Value },
    UserActivity { user_id: String, activity: ActivityType },
}
```

## Deployment Architecture

### Container Architecture

#### Single Node Deployment
```yaml
version: '3.8'
services:
  stepheybot-music:
    image: stepheybot-music:latest
    ports: [8083:8083]
    volumes:
      - ./data:/data          # Database persistence
      - ./music:/music:ro     # Music library (read-only)
      - ./cache:/cache        # Application cache
    environment:
      - DATABASE_URL=sqlite:/data/stepheybot.db
      - RUST_LOG=info
```

#### Multi-Service Deployment
```yaml
services:
  stepheybot-api:
    image: stepheybot-music:latest
    replicas: 3
    
  nginx:
    image: nginx:alpine
    ports: [80:80, 443:443]
    
  redis:
    image: redis:alpine
    
  navidrome:
    image: deluan/navidrome:latest
    
  lidarr:
    image: linuxserver/lidarr:latest
```

### Infrastructure Components

#### Load Balancer Configuration
```nginx
upstream stepheybot_backend {
    least_conn;
    server stepheybot-api-1:8083;
    server stepheybot-api-2:8083;
    server stepheybot-api-3:8083;
}

server {
    listen 443 ssl http2;
    server_name music.stepheybot.dev;
    
    location /api/ {
        proxy_pass http://stepheybot_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Integration Architecture

### External Service Integration

#### Navidrome Integration Flow
```
StepheyBot → Subsonic API → Navidrome → Music Files
     ↓           ↓              ↓
   Cache    Rate Limiting   Authentication
```

#### Music Acquisition Flow
```
User Request → Recommendation → Lidarr API → Download → Navidrome Scan → Library Update
```

### Authentication Flow (Planned)
```
Client → OAuth2 Provider → JWT Token → API Gateway → Service Authorization
```

## Security Architecture

### Security Layers

#### 1. **Network Security**
- HTTPS/TLS 1.3 for all external communication
- Internal service mesh with mTLS
- Firewall rules for port restriction
- VPN access for administrative functions

#### 2. **Application Security**
- Input validation and sanitization
- SQL injection prevention via prepared statements
- XSS protection with content security policies
- Rate limiting to prevent abuse

#### 3. **Data Security**
- Database encryption at rest
- Secure key management
- User data anonymization options
- GDPR compliance for EU users

#### 4. **Authentication & Authorization**
```rust
struct SecurityContext {
    user_id: Option<String>,
    permissions: HashSet<Permission>,
    rate_limit_quota: RateLimitQuota,
    request_metadata: RequestMetadata,
}

enum Permission {
    ReadLibrary,
    WritePlaylist,
    AdminAccess,
    UserManagement,
}
```

## Performance Architecture

### Performance Optimizations

#### 1. **Database Performance**
- Strategic indexing on query patterns
- Connection pooling with optimal sizing
- Read replicas for scaling (future)
- Query optimization and analysis

#### 2. **Caching Strategy**
```rust
enum CacheLayer {
    L1Memory(Duration),      // In-process cache
    L2Redis(Duration),       // Distributed cache
    L3Database(Duration),    // Database query cache
}

struct CacheKey {
    namespace: String,
    identifier: String,
    version: u64,
}
```

#### 3. **Async Processing**
- Background job processing for heavy operations
- Streaming responses for large datasets
- Connection pooling for external APIs
- Non-blocking I/O throughout the stack

### Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| API Response Time | < 100ms | ~50ms |
| Database Query Time | < 10ms | ~5ms |
| Recommendation Generation | < 500ms | ~200ms |
| Memory Usage | < 256MB | ~128MB |
| CPU Usage (idle) | < 5% | ~2% |

## Scalability Considerations

### Horizontal Scaling

#### Stateless Design
- No server-side session storage
- Database for all persistent state
- Cache for performance, not correctness
- Load balancer friendly architecture

#### Database Scaling
```rust
enum DatabaseTopology {
    SingleNode {
        database: SqlitePool,
        backup_strategy: BackupStrategy,
    },
    
    Distributed {
        primary: DatabaseNode,
        replicas: Vec<DatabaseNode>,
        sharding_strategy: ShardingStrategy,
    },
}
```

### Vertical Scaling
- Multi-core CPU utilization
- Memory-efficient data structures
- Optimized async runtime configuration
- Resource monitoring and alerting

### Future Scalability Plans
- **Phase 1**: Single node optimization (current)
- **Phase 2**: Multi-node deployment with load balancing
- **Phase 3**: Microservices decomposition
- **Phase 4**: Kubernetes orchestration
- **Phase 5**: Multi-region deployment

---

## Conclusion

The StepheyBot Music architecture is designed to be:

- **Robust**: Type-safe Rust implementation with comprehensive error handling
- **Scalable**: Stateless design ready for horizontal scaling
- **Maintainable**: Clean separation of concerns with well-defined interfaces
- **Performant**: Optimized for sub-100ms response times
- **Secure**: Multi-layered security approach with privacy focus
- **Extensible**: Plugin architecture for future enhancements

This architecture provides a solid foundation for building a world-class music recommendation platform while maintaining the flexibility to evolve with changing requirements.

**Architecture Version**: 1.0  
**Last Updated**: 2025-06-20  
**Next Review**: 2025-07-20