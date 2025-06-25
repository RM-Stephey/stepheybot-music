# StepheyBot Music - Complete Project Documentation

## 🎵 Project Overview

StepheyBot Music is a private, self-hosted music streaming service with AI-powered recommendations, built as part of the StepheyBot ecosystem. It provides a Spotify-like experience with complete control over your music library and data.

### ✨ Key Features

- 🎧 **Music Streaming**: Direct integration with Navidrome for high-quality audio streaming
- 🤖 **AI Recommendations**: Intelligent music discovery based on listening history and preferences  
- 🎨 **Modern Web Interface**: Responsive Svelte-based frontend with real-time player controls
- 📚 **Library Management**: Automatic music discovery and downloading via Lidarr integration
- 🔐 **Secure Access**: OAuth2 authentication with SSO integration
- 🐳 **Containerized**: Full Docker deployment with microservices architecture
- 📱 **Mobile Responsive**: Works seamlessly across desktop and mobile devices

## 🏗️ Architecture

### System Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Backend       │    │   Services      │
│   (Svelte)      │◄──►│   (Rust/Axum)   │◄──►│   (Docker)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                    ┌─────────────────────────────┐
                    │     External Services       │
                    │  ┌─────────┐ ┌─────────┐   │
                    │  │Navidrome│ │ Lidarr  │   │
                    │  │(Stream) │ │(Library)│   │
                    │  └─────────┘ └─────────┘   │
                    └─────────────────────────────┘
```

### Technology Stack

**Backend (Rust)**
- Framework: Axum (async HTTP server)
- Database: SQLite with SQLx ORM
- Authentication: OAuth2 proxy integration
- Streaming: Direct Navidrome API integration
- Configuration: Environment-based with TOML support

**Frontend (JavaScript/Svelte)**
- Framework: SvelteKit with SSR
- Build Tool: Vite
- Styling: Custom CSS with responsive design
- State Management: Svelte stores
- Audio: HTML5 Audio API with custom controls

**Infrastructure**
- Containerization: Docker with multi-stage builds
- Reverse Proxy: OAuth2-proxy for authentication
- Databases: PostgreSQL (main), Redis (sessions), SQLite (app data)
- File Storage: Volume mounts for music library access

## 🚀 Current Features

### ✅ Working Features

**Music Playback**
- ✅ Stream music from Navidrome library
- ✅ Play/pause/skip controls
- ✅ Queue management (add, remove, reorder)
- ✅ Volume control and progress tracking
- ✅ Track metadata display (title, artist, album, duration)

**Music Discovery**
- ✅ Personalized recommendations based on library
- ✅ Random discovery tracks from collection
- ✅ Search functionality across music library
- ✅ Add tracks to queue from discovery page

**User Interface**
- ✅ Responsive dashboard with system statistics
- ✅ Dedicated discovery page for finding new music
- ✅ Persistent music player bar across all pages
- ✅ Real-time queue visualization
- ✅ Mobile-optimized interface

**System Integration**
- ✅ Navidrome API integration for music streaming
- ✅ Lidarr API integration for library management
- ✅ Health monitoring and status endpoints
- ✅ OAuth2 authentication ready

### 🚧 In Development

- Advanced recommendation algorithms (collaborative filtering)
- Playlist creation and management
- User preference learning
- Advanced search filters
- Social features (sharing, following)
- Offline playback support

## 📡 API Reference

### Core Endpoints

#### Health & Status
```http
GET /health                    # Basic health check
GET /health/ready             # Readiness probe
GET /health/live              # Liveness probe
GET /api/v1/status            # Detailed system status
```

#### Music Streaming
```http
GET /api/v1/stream/:track_id       # Stream audio file
GET /api/v1/tracks/search/:query   # Search music library
GET /api/v1/discover               # Get discovery tracks
```

#### Recommendations
```http
GET /api/v1/recommendations/:user_id   # Get personalized recommendations
```

#### Player Control
```http
GET    /api/v1/player/current         # Get current playing track
GET    /api/v1/player/queue           # Get player queue
POST   /api/v1/player/queue           # Update player queue
POST   /api/v1/player/play/:track_id  # Play specific track
POST   /api/v1/player/pause           # Pause playback
POST   /api/v1/player/next            # Skip to next track
POST   /api/v1/player/previous        # Go to previous track
```

#### Library Management
```http
GET  /api/v1/library/stats       # Get library statistics
POST /api/v1/library/scan        # Trigger library scan
```

#### External Integrations
```http
GET  /api/v1/navidrome/status    # Navidrome connection status
GET  /api/v1/navidrome/stats     # Navidrome library stats
GET  /api/v1/lidarr/status       # Lidarr connection status
GET  /api/v1/lidarr/artists      # Get monitored artists
POST /api/v1/lidarr/add          # Add artist to monitoring
```

### API Response Format

All API responses follow this structure:

```json
{
  "success": true,
  "data": { /* response data */ },
  "timestamp": "2025-06-25T00:00:00Z",
  "version": "0.1.0"
}
```

Error responses:
```json
{
  "success": false,
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": "2025-06-25T00:00:00Z"
}
```

## 🛠️ Development Setup

### Prerequisites

- Docker & Docker Compose
- Rust 1.70+ (for local development)
- Node.js 18+ (for frontend development)
- Access to Navidrome instance
- Access to Lidarr instance (optional)

### Quick Start

1. **Clone and Setup**
   ```bash
   git clone <repository>
   cd music-recommender
   ```

2. **Production Deployment**
   ```bash
   cd ../  # Go to docker-compose.yml location
   docker-compose build stepheybot-music
   docker-compose up -d stepheybot-music
   ```

3. **Access Application**
   - Main Interface: http://localhost:8083
   - OAuth2 Proxy: http://localhost:4186 (if using SSO)
   - Health Check: http://localhost:8083/health

### Development Workflow

**Backend Development**
```bash
# Install Rust dependencies
cargo build

# Run backend only
export DATABASE_URL="sqlite:data/stepheybot-music.db"
export PORT="8083"
cargo run

# Run tests
cargo test
```

**Frontend Development**
```bash
cd frontend

# Install dependencies
npm install --legacy-peer-deps

# Development server (with backend proxy)
npm run dev

# Build for production
npm run build
```

**Full Development Stack**
```bash
# Use the provided development script
./start-dev.sh
```

## 🐳 Docker Configuration

### Main Service (docker-compose.yml)

```yaml
stepheybot-music:
  build:
    context: ./music-recommender
    dockerfile: Dockerfile
  container_name: stepheybot_music_brain
  restart: unless-stopped
  depends_on:
    - navidrome
    - lidarr
  ports:
    - "8083:8083"
  environment:
    # Database configuration
    - DATABASE_URL=sqlite:/data/stepheybot-music.db
    # Navidrome integration
    - NAVIDROME_URL=http://stepheybot_music_navidrome:4533
    - NAVIDROME_USERNAME=${NAVIDROME_USERNAME}
    - NAVIDROME_PASSWORD=${NAVIDROME_PASSWORD}
    # Lidarr integration
    - LIDARR_URL=http://stepheybot_music_lidarr:8686
    - LIDARR_API_KEY=${LIDARR_API_KEY}
  volumes:
    - stepheybot_music_data:/data
    - stepheybot_music_cache:/cache
    - /mnt/nvme/music:/music:ro
  networks:
    - nextcloud_net
```

### OAuth2 Proxy Integration

```yaml
oauth2-proxy-stepheybot:
  image: quay.io/oauth2-proxy/oauth2-proxy:v7.4.0
  container_name: oauth2-proxy-stepheybot
  command:
    - --upstream=http://stepheybot_music_brain:8083
    - --provider=oidc
    - --oidc-issuer-url=https://sso.axiomethica.io/realms/stepheybot
    # ... additional OAuth2 configuration
  ports:
    - "4186:4180"
```

## 📁 Project Structure

```
music-recommender/
├── src/                           # Rust backend source
│   ├── main.rs                   # Main application entry point
│   ├── navidrome_addon.rs        # Navidrome API integration
│   └── lidarr_addon.rs           # Lidarr API integration
├── frontend/                      # Svelte frontend
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +layout.svelte    # Main layout with music player
│   │   │   ├── +page.svelte      # Dashboard/home page
│   │   │   └── discover/
│   │   │       └── +page.svelte  # Music discovery page
│   │   └── lib/
│   │       ├── components/
│   │       │   ├── MusicPlayer.svelte    # Main music player component
│   │       │   └── MusicDiscovery.svelte # Discovery interface
│   │       └── stores/
│   │           └── musicPlayer.js        # Svelte store for player state
│   ├── package.json              # Frontend dependencies
│   ├── vite.config.js            # Vite build configuration
│   └── svelte.config.js          # SvelteKit configuration
├── migrations/                    # Database migrations
├── Dockerfile                     # Multi-stage Docker build
├── Cargo.toml                    # Rust dependencies and metadata
└── docs/                         # Project documentation
```

## 🔧 Configuration

### Environment Variables

**Required:**
```env
# Database
DATABASE_URL=sqlite:/data/stepheybot-music.db

# Server
PORT=8083
HOST=0.0.0.0

# Navidrome Integration
NAVIDROME_URL=http://navidrome:4533
NAVIDROME_USERNAME=your_username
NAVIDROME_PASSWORD=your_password

# Lidarr Integration (Optional)
LIDARR_URL=http://lidarr:8686
LIDARR_API_KEY=your_api_key
```

**Optional:**
```env
# Logging
RUST_LOG=stepheybot_music=info,tower_http=debug
RUST_BACKTRACE=1

# Performance
RUST_MIN_THREADS=4
RUST_MAX_THREADS=16

# Features
ENABLE_DISCOVERY=true
ENABLE_RECOMMENDATIONS=true
ENABLE_LIDARR_INTEGRATION=true
```

## 🔍 Troubleshooting

### Common Issues

**1. Music Not Playing from Discovery Page**
- **Symptom**: Tracks add to queue but won't play, shows "undefined" in audio src
- **Solution**: Ensure discover API returns `stream_url` field for each track
- **Fixed**: Recent patch added stream_url generation to discover endpoint

**2. OAuth2 Authentication Loops**
- **Symptom**: Redirects continuously between app and SSO
- **Solution**: Check cookie domain and redirect URL configuration
- **Check**: Ensure OAuth2 proxy `--cookie-domain` matches your domain

**3. Navidrome Connection Failed**
- **Symptom**: No music recommendations or streaming
- **Solution**: Verify Navidrome credentials and network connectivity
- **Debug**: Check `/api/v1/navidrome/status` endpoint

**4. Frontend Build Errors**
- **Symptom**: Vite build fails or JavaScript errors
- **Solution**: Use `npm install --legacy-peer-deps` for dependency resolution
- **Alternative**: Clear node_modules and package-lock.json, reinstall

### Debug Endpoints

- `/health` - Basic health status
- `/api/v1/navidrome/debug` - Detailed Navidrome connection info
- `/api/v1/lidarr/status` - Lidarr integration status
- `/api/v1/stats` - System statistics and performance

### Logs

**Backend Logs:**
```bash
docker logs stepheybot_music_brain -f
```

**Frontend Logs (Development):**
```bash
cd frontend && npm run dev
```

## 🚀 Recent Updates

### v0.1.0 - Current Release

**✅ Fixed Issues:**
- Music playback from discovery page (missing stream_url)
- Frontend store integration for music player
- OAuth2 proxy configuration for production
- Responsive design improvements

**🆕 New Features:**
- Complete music player with queue management
- Discovery page with search functionality
- Real-time player state synchronization
- Mobile-responsive interface

**🔧 Technical Improvements:**
- Multi-stage Docker build optimization
- Improved error handling and logging
- Clean separation between development and production
- Comprehensive API documentation

## 📋 TODO / Roadmap

### High Priority
- [ ] Implement proper user authentication and sessions
- [ ] Add playlist creation and management
- [ ] Implement advanced recommendation algorithms
- [ ] Add user preference learning
- [ ] Create admin interface for system management

### Medium Priority
- [ ] Add social features (sharing, following)
- [ ] Implement offline playback support
- [ ] Add lyrics integration
- [ ] Create mobile app (React Native)
- [ ] Add scrobbling to Last.fm/ListenBrainz

### Low Priority
- [ ] Add equalizer and audio effects
- [ ] Implement smart playlists
- [ ] Add podcast support
- [ ] Create API rate limiting
- [ ] Add comprehensive analytics

## 🤝 Contributing

1. **Development Setup**: Follow the development setup guide above
2. **Code Style**: Run `cargo fmt` for Rust, `npm run format` for frontend
3. **Testing**: Ensure all tests pass with `cargo test`
4. **Documentation**: Update this documentation for any architectural changes
5. **Pull Requests**: Create feature branches and submit PRs with detailed descriptions

## 📄 License

This project is part of the StepheyBot ecosystem. See LICENSE file for details.

---

**Project Status**: ✅ **Active Development**  
**Last Updated**: June 25, 2025  
**Version**: 0.1.0  
**Maintainer**: Stephey <stephey@stepheybot.dev>