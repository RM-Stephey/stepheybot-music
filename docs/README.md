# 🎵 StepheyBot Music - Project Documentation

> **Private Spotify-like music streaming service with AI-powered recommendations**

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg?style=flat-square&logo=docker)](https://www.docker.com)
[![SQLite](https://img.shields.io/badge/database-sqlite-green.svg?style=flat-square&logo=sqlite)](https://sqlite.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)

## 📋 Table of Contents

- [Project Overview](#-project-overview)
- [Current Status](#-current-status)
- [Architecture](#-architecture)
- [Documentation Structure](#-documentation-structure)
- [Quick Start](#-quick-start)
- [Development Progress](#-development-progress)
- [Team & Contact](#-team--contact)

## 🎯 Project Overview

StepheyBot Music is a self-hosted, privacy-focused music streaming platform designed to provide intelligent music recommendations while keeping your data completely under your control. Built with modern Rust technology and optimized for CachyOS + Hyprland environments.

### Key Features

- 🎶 **AI-Powered Recommendations** - Multiple algorithms including collaborative filtering, content-based analysis, and audio feature matching
- 🎵 **Personal Music Library** - Stream from your own collection with rich metadata enrichment
- 📱 **Modern Web Interface** - Responsive design with customizable neon-themed UI
- 🔒 **Privacy First** - Your data never leaves your server
- 🐳 **Docker Ready** - Easy deployment with optimized containers
- 🎧 **Navidrome Integration** - Seamless compatibility with existing music servers
- 📊 **Advanced Analytics** - Detailed insights into listening patterns and preferences

### Vision

Create the ultimate personalized music experience that combines the convenience of modern streaming services with the privacy and control of self-hosted solutions, specifically tailored for power users who value customization and technical excellence.

## 🚀 Current Status

### ✅ Completed Features

- **Backend Infrastructure** (v0.1.0)
  - ✅ Rust HTTP server with Axum framework
  - ✅ SQLite database with comprehensive schema
  - ✅ Multi-algorithm recommendation engine
  - ✅ RESTful API with full endpoint coverage
  - ✅ Sample data with realistic music metadata
  - ✅ Docker containerization
  - ✅ Health monitoring and diagnostics

### 🚧 In Progress

- **Web Interface** (v0.2.0) - Starting Next
  - 🎨 Neon-themed Svelte frontend with smooth animations
  - 📱 Responsive design for all devices
  - 🎵 Real-time music player integration
  - ⚡ Hot-reload development environment

### 📋 Planned Features

- **Enhanced Integration** (v0.3.0)
  - 🎧 Full Navidrome client integration
  - 📥 Lidarr music acquisition support
  - 🎵 MusicBrainz metadata enrichment
  - 🎤 Voice command integration ("StepheyBot, play something energetic")

- **Advanced Features** (v0.4.0)
  - 🤖 Machine learning model training
  - 🏠 Home Assistant integration
  - 📊 Advanced analytics dashboard
  - 👥 Multi-user support with profiles

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Client    │───▶│  StepheyBot     │───▶│   Navidrome     │
│ (Svelte/Neon UI)│    │  Music API      │    │ (Music Server)  │
│                 │    │   (Rust)        │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                       ┌─────────────────┐    ┌─────────────────┐
                       │    SQLite       │    │     Lidarr      │
                       │   Database      │    │ (Music Manager) │
                       └─────────────────┘    └─────────────────┘
```

### Technology Stack

- **Backend**: Rust + Axum + SQLx + SQLite
- **Frontend**: Svelte + TypeScript + Tailwind CSS
- **Database**: SQLite with performance optimizations
- **Containerization**: Docker with multi-stage builds
- **Integration**: Navidrome API, Lidarr API, MusicBrainz
- **Deployment**: Docker Compose + Nginx

## 📚 Documentation Structure

This documentation is organized into the following sections:

### 📖 Core Documentation
- [`README.md`](README.md) - This overview document
- [`../README.md`](../README.md) - Main project README

### 🔧 Setup & Installation
- [`setup/installation.md`](setup/installation.md) - Installation guide
- [`setup/configuration.md`](setup/configuration.md) - Configuration reference
- [`setup/docker.md`](setup/docker.md) - Docker deployment guide
- [`setup/development.md`](setup/development.md) - Development environment setup

### 🏗️ Architecture & Design
- [`architecture/overview.md`](architecture/overview.md) - System architecture
- [`architecture/database.md`](architecture/database.md) - Database schema and design
- [`architecture/api.md`](architecture/api.md) - API design principles
- [`architecture/recommendations.md`](architecture/recommendations.md) - Recommendation algorithms

### 📡 API Documentation
- [`api/endpoints.md`](api/endpoints.md) - Complete API reference
- [`api/authentication.md`](api/authentication.md) - Authentication methods
- [`api/examples.md`](api/examples.md) - API usage examples
- [`api/postman.json`](api/postman.json) - Postman collection

### 👨‍💻 Development
- [`development/contributing.md`](development/contributing.md) - Contribution guidelines
- [`development/coding-standards.md`](development/coding-standards.md) - Code style guide
- [`development/testing.md`](development/testing.md) - Testing strategies
- [`development/debugging.md`](development/debugging.md) - Debugging guide

### 📈 Progress Tracking
- [`progress/changelog.md`](progress/changelog.md) - Version history
- [`progress/roadmap.md`](progress/roadmap.md) - Feature roadmap
- [`progress/milestones.md`](progress/milestones.md) - Development milestones

## ⚡ Quick Start

### Prerequisites
- Rust 1.80+
- Docker & Docker Compose
- Git

### 1. Clone & Setup
```bash
git clone https://github.com/RM-Stephey/stepheybot-music.git
cd stepheybot-music
cp .env.template .env
# Edit .env with your configuration
```

### 2. Run with Docker
```bash
docker build -t stepheybot-music .
docker run -p 8083:8083 -v ./data:/data stepheybot-music
```

### 3. Run from Source
```bash
cargo build --release
./target/release/stepheybot-music
```

### 4. Test the API
```bash
# Health check
curl http://localhost:8083/health

# Get recommendations
curl http://localhost:8083/api/v1/recommendations/user1

# Library stats
curl http://localhost:8083/api/v1/library/stats
```

## 📊 Development Progress

### Phase 1: Core Backend ✅ COMPLETED
- [x] Project setup and structure
- [x] Database schema design
- [x] API endpoint implementation
- [x] Recommendation engine algorithms
- [x] Sample data generation
- [x] Docker containerization
- [x] Documentation foundation

### Phase 2: Web Interface 🚧 IN PROGRESS
- [ ] Svelte frontend setup with SvelteKit
- [ ] Neon theme implementation with CSS animations
- [ ] Interactive music player component
- [ ] Real-time recommendation UI
- [ ] Dynamic library browser
- [ ] User management interface

### Phase 3: Integration 📋 PLANNED
- [ ] Navidrome client integration
- [ ] Lidarr music acquisition
- [ ] MusicBrainz metadata enrichment
- [ ] Authentication system
- [ ] Real-time WebSocket updates

### Phase 4: Advanced Features 📋 PLANNED
- [ ] Machine learning improvements
- [ ] Voice command integration
- [ ] Home Assistant plugin
- [ ] Mobile application
- [ ] Multi-user support

### Design Philosophy

### User Experience
- **Immediate Functionality**: Everything should work out of the box
- **Progressive Enhancement**: Add complexity gradually as needed
- **Visual Appeal**: Neon aesthetics with Svelte's smooth animations
- **Performance First**: Fast, responsive, and efficient (Svelte's compile-time optimizations)

### Technical Principles
- **Type Safety**: Rust's type system prevents runtime errors
- **Data Ownership**: Users own and control their data completely
- **Modularity**: Clean separation between components
- **Testability**: Comprehensive test coverage for reliability

## 👥 Team & Contact

### Core Team
- **RM-Stephey** - Project Lead & Backend Developer
  - GitHub: [@RM-Stephey](https://github.com/RM-Stephey)
  - Email: stephey@stepheybot.dev

### Contributing
We welcome contributions! Please see [`development/contributing.md`](development/contributing.md) for guidelines.

### Community
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/RM-Stephey/stepheybot-music/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/RM-Stephey/stepheybot-music/discussions)
- 📧 **Email**: stephey@stepheybot.dev

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- **Navidrome** - Excellent foundation for music streaming
- **Rust Community** - Amazing ecosystem and support
- **CachyOS** - Optimized Linux distribution
- **Hyprland** - Modern Wayland compositor

---

<div align="center">
  <p>🎵 <strong>Built with ❤️ for the self-hosted music community</strong> 🎵</p>
  <p><em>"Making your music experience intelligent, personal, and beautiful"</em></p>
</div>