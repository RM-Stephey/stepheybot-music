# StepheyBot Music

🎵 **Private, self-hosted music streaming service with AI-powered recommendations**

A Spotify-like experience built with Rust and Svelte, featuring seamless Navidrome integration, intelligent music discovery, and a beautiful responsive interface.

## ✨ Features

- 🎧 High-quality music streaming via Navidrome
- 🤖 AI-powered music recommendations
- 🎨 Modern, responsive web interface
- 📚 Automatic library management with Lidarr
- 🔐 OAuth2 authentication with SSO
- 🐳 Full Docker deployment

## 🚀 Quick Start

```bash
# Deploy with Docker Compose
cd ../  # Navigate to docker-compose.yml location
docker-compose build stepheybot-music
docker-compose up -d stepheybot-music

# Access the application
open http://localhost:8083
```

## 📖 Documentation

**👉 [Complete Project Documentation](docs/PROJECT_DOCUMENTATION.md) 👈**

The comprehensive documentation includes:
- Detailed architecture overview
- Complete API reference
- Development setup guide
- Configuration options
- Troubleshooting guide
- Contributing guidelines

## 🔗 Quick Links

- **Health Check**: http://localhost:8083/health
- **Music Discovery**: http://localhost:8083/discover
- **API Status**: http://localhost:8083/api/v1/status

## 🏗️ Tech Stack

- **Backend**: Rust + Axum
- **Frontend**: Svelte + SvelteKit
- **Database**: SQLite + PostgreSQL
- **Streaming**: Navidrome integration
- **Library**: Lidarr integration
- **Auth**: OAuth2 proxy

---

**Status**: ✅ Production Ready  
**Version**: 0.1.0  
**Maintainer**: Stephey <stephey@stepheybot.dev>