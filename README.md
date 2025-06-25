# 🎵 StepheyBot Music

> **Private Spotify-like music streaming service with AI-powered recommendations**

StepheyBot Music is a self-hosted, privacy-focused music streaming platform that provides intelligent music recommendations, playlist generation, and seamless integration with your personal music library.

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg?style=flat-square&logo=docker)](https://www.docker.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg?style=flat-square)](https://github.com/stephey/stepheybot-music)

## ✨ Features

- 🎶 **Smart Music Recommendations** - AI-powered suggestions based on your listening habits
- 🎵 **Personal Music Library** - Stream from your own collection with metadata enrichment
- 📱 **Neon-Themed Web Interface** - Beautiful Svelte frontend with cyberpunk aesthetics (cyan, pink, purple neon effects)
- 🔒 **Privacy First** - Your data stays on your server
- 🐳 **Docker Ready** - Easy deployment with multi-architecture support
- 🎧 **Navidrome Integration** - Seamless compatibility with existing setups
- 📊 **Music Analytics** - Detailed insights into your listening patterns
- 🎛️ **Playlist Management** - Smart playlist generation and curation

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Neon Frontend  │───▶│  StepheyBot     │───▶│   Navidrome     │
│   (Svelte 4)    │    │  Music (Rust)   │    │ (Music Server)  │
│  Port: 5173     │    │  Port: 8083     │    │   Port: 4533    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                       ┌─────────────────┐    ┌─────────────────┐
                       │    Database     │    │     Lidarr      │
                       │   (SQLite)      │    │ (Music Manager) │
                       └─────────────────┘    └─────────────────┘
```
</end_text>

<old_text>
### From Source

```bash
# Prerequisites: Rust 1.80+, SQLite
git clone https://github.com/stephey/stepheybot-music.git
cd stepheybot-music

# Build and run
cargo build --release
./target/release/stepheybot-music
```

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/stephey/stepheybot-music.git
cd stepheybot-music

# Run with Docker
docker run -d \
  --name stepheybot-music \
  -p 8083:8083 \
  -v ./data:/data \
  -v ./music:/music \
  -e STEPHEYBOT__DATABASE__URL=sqlite:/data/stepheybot-music.db \
  stepheybot-music:latest
```

### From Source

```bash
# Prerequisites: Rust 1.80+, SQLite
git clone https://github.com/stephey/stepheybot-music.git
cd stepheybot-music

# Build and run
cargo build --release
./target/release/stepheybot-music
```

## 🔧 Configuration

### Environment Variables

```bash
# Server Configuration
STEPHEYBOT__SERVER__PORT=8083
STEPHEYBOT__SERVER__ADDRESS=0.0.0.0

# Database
STEPHEYBOT__DATABASE__URL=sqlite:/data/stepheybot-music.db

# External Services
NAVIDROME_URL=http://localhost:4533
NAVIDROME_USERNAME=admin
NAVIDROME_PASSWORD=your_password

LIDARR_URL=http://localhost:8686
LIDARR_API_KEY=your_api_key

# Paths
STEPHEYBOT__PATHS__MUSIC_PATH=/music
STEPHEYBOT__PATHS__CACHE_PATH=/cache
STEPHEYBOT__PATHS__DOWNLOAD_PATH=/downloads

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
```

### Configuration File

Create `config/local.toml`:

```toml
[server]
port = 8083
address = "0.0.0.0"

[database]
url = "sqlite:data/stepheybot-music.db"

[navidrome]
url = "http://localhost:4533"
username = "admin"
password = "your_password"

[lidarr]
url = "http://localhost:8686"
api_key = "your_api_key"

[recommendations]
enabled = true
model = "collaborative_filtering"
update_interval = "24h"
```

## 📚 Documentation

Complete project documentation is available in the [`docs/`](docs/) directory:

### 📖 Core Documentation
- **[Project Overview](docs/README.md)** - Comprehensive project documentation
- **[Development Guide](docs/setup/development.md)** - Setup and development environment
- **[API Reference](docs/api/endpoints.md)** - Complete API documentation with examples

### 🏗️ Technical Documentation  
- **[System Architecture](docs/architecture/overview.md)** - Architecture and design decisions
- **[Database Schema](docs/architecture/database.md)** - Data models and relationships
- **[Recommendation Algorithms](docs/architecture/recommendations.md)** - AI recommendation details

### 📈 Project Management
- **[Changelog](docs/progress/changelog.md)** - Version history and updates
- **[Roadmap](docs/progress/roadmap.md)** - Feature roadmap and timeline
- **[Contributing](docs/development/contributing.md)** - Contribution guidelines

### 🚀 Quick Links
- **API Status**: `GET /api/v1/status` - Service information
- **Health Check**: `GET /health` - Service health
- **Get Recommendations**: `GET /api/v1/recommendations/{user_id}` - Personalized music
- **Library Stats**: `GET /api/v1/library/stats` - Music library statistics

## 📡 API Endpoints

### Health & Status
- `GET /health` - Health check
- `GET /health/ready` - Readiness check
- `GET /health/live` - Liveness check
- `GET /api/v1/status` - API status

### Music & Recommendations
- `GET /api/v1/recommendations/:user_id` - Get user recommendations
- `POST /api/v1/playlists/generate` - Generate smart playlist
- `POST /api/v1/library/scan` - Scan music library
- `GET /api/v1/stats` - Get system statistics

### Admin (Protected)
- `GET /admin/users` - List users
- `GET /admin/system` - System information

## 🔨 Development

### Prerequisites

- Rust 1.80+
- Node.js 18+ and npm
- SQLite 3.x
- Docker (optional)

### Setup

```bash
# Clone and setup
git clone https://github.com/RM-Stephey/stepheybot-music.git
cd stepheybot-music

# Run comprehensive system test
./test-system.sh

# Start development environment (both backend + frontend)
./start-dev.sh

# Access the application:
# - Backend API: http://localhost:8083
# - Frontend UI: http://localhost:5173
```

### Individual Components

```bash
# Backend only
cargo run

# Frontend only
cd frontend
npm install --legacy-peer-deps
npm run dev
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out html

# Integration tests
cargo test --test integration
```

### Docker Development

```bash
# Build development image
docker build -t stepheybot-music:dev .

# Run with live reload
docker run -v $(pwd):/app stepheybot-music:dev
```

## 🎨 Neon-Themed Interface

StepheyBot Music features a stunning cyberpunk aesthetic designed for tech enthusiasts:

### 🌟 Visual Features
- **Neon Color Palette**: Electric cyan (#00FFFF), hot pink (#FF00FF), deep purple (#8000FF)
- **Glowing Effects**: Animated borders, pulsing elements, and shadow effects
- **Dark Gradient Backgrounds**: Smooth transitions from black to deep blue/purple
- **Futuristic Typography**: Orbitron and Rajdhani fonts with text glow effects
- **Smooth Animations**: Hover effects, loading states, and transitions

### 🎨 Theme Configuration

```css
/* Built-in neon theme variables */
:root {
  --neon-cyan: #00FFFF;
  --neon-pink: #FF00FF;
  --neon-purple: #8000FF;
  --bg-primary: #0A0A0F;
  --bg-secondary: #1A1A2E;
  --shadow-neon: 0 0 20px var(--neon-cyan);
}
```

### 📱 Responsive Design
- Mobile-first approach with touch-friendly controls
- Adaptive layouts for desktop, tablet, and mobile
- High contrast mode support for accessibility
- Reduced motion support for users with vestibular disorders

### Custom Recommendation Algorithms

```rust
// Implement custom recommendation trait
impl RecommendationEngine for CustomEngine {
    async fn generate_recommendations(&self, user_id: &str) -> Result<Vec<Track>> {
        // Your custom algorithm here
    }
}
```

## 🐳 Docker Deployment

### Docker Compose

```yaml
version: '3.8'
services:
  stepheybot-music:
    image: stepheybot-music:latest
    ports:
      - "8083:8083"
    volumes:
      - ./data:/data
      - ./music:/music
      - ./cache:/cache
    environment:
      - STEPHEYBOT__DATABASE__URL=sqlite:/data/stepheybot-music.db
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8083/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Production Deployment

```bash
# Build production image
docker build -t stepheybot-music:prod --target production .

# Deploy with proper security
docker run -d \
  --name stepheybot-music \
  --user 1001:1001 \
  --read-only \
  --tmpfs /tmp \
  -p 8083:8083 \
  stepheybot-music:prod
```

## 🔍 Monitoring

### Health Checks

```bash
# Basic health
curl http://localhost:8083/health

# Detailed status
curl http://localhost:8083/api/v1/status | jq
```

### Metrics & Logs

```bash
# View logs
docker logs -f stepheybot-music

# System stats
curl http://localhost:8083/api/v1/stats | jq
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md).

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run all checks
make check
```

## 📋 Roadmap

### ✅ Completed (v0.1.0)
- [x] **Rust Backend API** - Complete REST API with health checks
- [x] **SQLite Database** - Normalized schema with sample data
- [x] **Neon-Themed Frontend** - Beautiful Svelte interface with cyberpunk aesthetics
- [x] **Docker Support** - Production-ready containerization
- [x] **Basic Recommendations** - Foundation for AI-powered suggestions
- [x] **Development Tools** - Test scripts and startup automation

### 🚧 In Progress (v0.2.0)
- [ ] **Enhanced Recommendation Engine** - Machine learning algorithms
- [ ] **Music Player Component** - Integrated audio playback
- [ ] **Real-time Updates** - WebSocket integration
- [ ] **Navidrome Integration** - Complete music server connectivity

### 📅 Planned (v0.3.0+)
- [ ] **Voice Control** - "StepheyBot, play something energetic"
- [ ] **Mobile PWA** - Progressive web app for mobile devices  
- [ ] **Smart Home Integration** - Home Assistant compatibility
- [ ] **Social Features** - Share playlists and recommendations
- [ ] **Advanced Analytics** - Machine learning insights
- [ ] **Multi-user Support** - Family and shared accounts

## 🔧 Troubleshooting

### Common Issues

**Database Connection Error**
```bash
# Check database file permissions
ls -la data/stepheybot-music.db

# Reset database
rm data/stepheybot-music.db
cargo run -- migrate
```

**Port Already in Use**
```bash
# Find process using port 8083
lsof -i :8083

# Use different port
STEPHEYBOT__SERVER__PORT=8084 cargo run
```

**Docker Build Fails**
```bash
# Clean Docker cache
docker system prune -a

# Rebuild with no cache
docker build --no-cache -t stepheybot-music .
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Navidrome** - Excellent music server foundation
- **Lidarr** - Music collection management
- **Rust Community** - Amazing ecosystem and support
- **Contributors** - Everyone who makes this project better

## 📞 Support

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/stephey/stepheybot-music/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/stephey/stepheybot-music/discussions)
- 📧 **Email**: stephey@stepheybot.dev
- 🐙 **GitHub**: [RM-Stephey/stepheybot-music](https://github.com/RM-Stephey/stepheybot-music)

---

<div align="center">
  <p>Built with ❤️ by <a href="https://github.com/stephey">@stephey</a></p>
  <p>🎵 <em>"Making your music experience intelligent and personal"</em> 🎵</p>
</div>