# 🎵 StepheyBot Music - Navidrome Integration Guide

> **Connect your personal music library to AI-powered recommendations**

Transform StepheyBot Music from using sample data to your real music collection with seamless Navidrome integration.

---

## 🌟 What is Navidrome Integration?

Navidrome integration connects StepheyBot Music to your [Navidrome](https://www.navidrome.org/) music server, enabling:

- **Real Music Data**: Use your actual music library instead of sample tracks
- **AI Recommendations**: Get personalized suggestions based on your collection
- **Live Library Stats**: See real-time statistics from your music server
- **Playlist Management**: Create and manage playlists directly in Navidrome
- **Artist & Album Data**: Access complete metadata from your library

---

## 🚀 Quick Start

### Prerequisites

1. **Navidrome Server**: Running and accessible
2. **Valid Credentials**: Username and password for Navidrome
3. **Network Access**: StepheyBot Music can reach your Navidrome instance

### One-Command Setup

```bash
./enable-navidrome.sh
```

This script will:
- Detect running Navidrome instances
- Test connectivity and authentication
- Configure environment variables
- Switch to Navidrome-integrated mode
- Test the complete integration

---

## 📋 Step-by-Step Setup

### Step 1: Test Navidrome Connection

```bash
./test-navidrome.sh
```

This will verify:
- ✅ Navidrome server is reachable
- ✅ Authentication works
- ✅ API endpoints respond
- ✅ Library data is accessible

### Step 2: Configure Integration

```bash
./setup-navidrome.sh
```

The setup script will:
- Auto-detect Navidrome instances on common ports
- Prompt for server URL, username, and password
- Test the connection
- Create `.env` configuration file

### Step 3: Enable Integration

```bash
./enable-navidrome.sh
```

This switches StepheyBot Music to use:
- Navidrome-integrated main.rs
- Real music library data
- Enhanced API endpoints

### Step 4: Start Development Environment

```bash
./start-dev.sh
```

Access your enhanced music recommendation system:
- **Frontend**: http://localhost:5173 (Neon-themed UI)
- **Backend**: http://localhost:8083 (API with Navidrome data)

---

## 🔧 Configuration

### Environment Variables

The integration uses these key environment variables:

```bash
# Navidrome server configuration
NAVIDROME_URL=http://localhost:4533
NAVIDROME_USERNAME=your_username
NAVIDROME_PASSWORD=your_password
NAVIDROME_TIMEOUT=30
NAVIDROME_VERIFY_SSL=true

# StepheyBot Music settings
PORT=8083
DATABASE_URL=sqlite:data/stepheybot-music.db
POPULATE_SAMPLE_DATA=false
ENABLE_BACKGROUND_TASKS=true
```

### Manual Configuration

Edit `.env` file directly:

```bash
# Example for local Navidrome
NAVIDROME_URL=http://localhost:4533
NAVIDROME_USERNAME=admin
NAVIDROME_PASSWORD=your_secure_password

# Example for remote Navidrome
NAVIDROME_URL=https://music.yourdomain.com
NAVIDROME_USERNAME=stephey
NAVIDROME_PASSWORD=your_secure_password
```

---

## 🎵 Features & Endpoints

### Enhanced API Endpoints

With Navidrome integration, you get additional endpoints:

#### Navidrome Status
```bash
curl http://localhost:8083/api/v1/navidrome/status
```

#### Real Artist Data
```bash
curl http://localhost:8083/api/v1/navidrome/artists
```

#### Album Information
```bash
curl http://localhost:8083/api/v1/navidrome/albums
```

#### Library Statistics (Real Data)
```bash
curl http://localhost:8083/api/v1/library/stats
```

#### Recommendations from Your Library
```bash
curl http://localhost:8083/api/v1/recommendations/user1
```

### Data Sources

| Feature | Sample Data Mode | Navidrome Mode |
|---------|------------------|----------------|
| Artists | 5 synthetic artists | Your complete artist collection |
| Albums | 5 sample albums | All albums in your library |
| Tracks | 10 demo tracks | Your entire music collection |
| Recommendations | Based on sample data | Based on your listening habits |
| Playlists | Generated examples | Real Navidrome playlists |
| Cover Art | None | Actual album artwork |

---

## 🛠️ Available Scripts

### Core Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `test-navidrome.sh` | Test Navidrome connection | `./test-navidrome.sh` |
| `setup-navidrome.sh` | Configure integration | `./setup-navidrome.sh` |
| `enable-navidrome.sh` | Enable integration | `./enable-navidrome.sh` |
| `disable-navidrome.sh` | Switch back to sample data | `./disable-navidrome.sh` |

### Development Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `start-dev.sh` | Start both servers | `./start-dev.sh` |
| `test-system.sh` | Complete system test | `./test-system.sh` |

---

## 🔍 Testing Your Integration

### Quick Health Check

```bash
# Test Navidrome connection
./test-navidrome.sh

# Check integration status
curl http://localhost:8083/api/v1/navidrome/status

# Verify library stats
curl http://localhost:8083/api/v1/library/stats
```

### Frontend Testing

1. Open http://localhost:5173
2. Check the dashboard shows real library statistics
3. Test recommendations to see your actual music
4. Verify search works with your collection

### Backend API Testing

```bash
# Get recommendations from your library
curl "http://localhost:8083/api/v1/recommendations/user1?limit=5"

# Search your music collection
curl "http://localhost:8083/api/v1/library/search?q=artist_name"

# Get trending tracks from your library
curl "http://localhost:8083/api/v1/recommendations/trending"
```

---

## 🚨 Troubleshooting

### Common Issues

#### "Connection Failed" Error

**Problem**: Cannot connect to Navidrome server

**Solutions**:
```bash
# Check if Navidrome is running
curl http://localhost:4533

# Verify URL in configuration
cat .env | grep NAVIDROME_URL

# Test with different port
./test-navidrome.sh
```

#### "Authentication Failed" Error

**Problem**: Invalid credentials

**Solutions**:
```bash
# Verify credentials in Navidrome web interface
# Check username/password in .env file
cat .env | grep NAVIDROME_USERNAME

# Test authentication manually
./test-navidrome.sh
```

#### "Empty Library" Response

**Problem**: Navidrome library appears empty

**Solutions**:
```bash
# Check Navidrome has scanned your music
# Verify user has access to library
# Check Navidrome logs for scan issues
```

#### Build Errors After Enabling

**Problem**: Compilation fails with Navidrome integration

**Solutions**:
```bash
# Restore original version
./disable-navidrome.sh

# Clean build cache
cargo clean && cargo build

# Check Rust version
cargo --version
```

### Advanced Troubleshooting

#### Network Issues

```bash
# Test basic connectivity
ping your_navidrome_server

# Check port accessibility
telnet localhost 4533

# Verify SSL certificates (for HTTPS)
openssl s_client -connect your_server:443
```

#### Configuration Issues

```bash
# View complete configuration
cat .env

# Check for syntax errors
grep -v "^#" .env | grep "="

# Validate environment loading
env | grep NAVIDROME
```

#### Log Analysis

```bash
# View StepheyBot Music logs
tail -f backend.log

# Check Navidrome logs
sudo journalctl -u navidrome -f

# Debug network requests
RUST_LOG=debug cargo run
```

---

## ⚡ Performance Considerations

### Optimization Tips

1. **Connection Pooling**: Navidrome client uses connection pooling for efficiency
2. **Rate Limiting**: Built-in rate limiting prevents API overload
3. **Caching**: Recommendations are cached to reduce server load
4. **Background Sync**: Optional background tasks sync data periodically

### Configuration for Large Libraries

```bash
# For libraries with 10,000+ tracks
NAVIDROME_TIMEOUT=60
DB_MAX_CONNECTIONS=20
SYNC_INTERVAL=120
RECOMMENDATION_INTERVAL=180
```

### Memory Usage

- **Sample Data Mode**: ~50MB RAM
- **Small Library** (<1,000 tracks): ~100MB RAM
- **Large Library** (>10,000 tracks): ~200MB RAM

---

## 🔄 Switching Between Modes

### Enable Navidrome Integration

```bash
# From sample data mode to Navidrome
./enable-navidrome.sh

# Force enable without prompts
./enable-navidrome.sh --force
```

### Disable Navidrome Integration

```bash
# Switch back to sample data
./disable-navidrome.sh

# Keep configuration for later
./disable-navidrome.sh --backup-only
```

### Check Current Mode

```bash
# View API status
curl http://localhost:8083/api/v1/status

# Check configuration
grep -E "(NAVIDROME|POPULATE_SAMPLE)" .env
```

---

## 🎨 Frontend Integration

### Neon UI with Real Data

The beautiful neon-themed frontend automatically adapts to show:

- **Real Library Statistics**: Your actual artist/album/track counts
- **Your Music Recommendations**: AI suggestions from your collection
- **Album Artwork**: Cover art from your Navidrome library
- **Personal Playlists**: Your actual Navidrome playlists

### UI Indicators

| Element | Sample Mode | Navidrome Mode |
|---------|-------------|----------------|
| Library Stats | Shows sample counts | Shows real library size |
| Recommendations | Generic electronic music | Your music preferences |
| Source Badge | "Sample Data" | "Navidrome Connected" |
| Status Indicator | Yellow (Sample) | Green (Connected) |

---

## 🔮 Advanced Features

### Background Synchronization

When enabled, StepheyBot Music periodically syncs with Navidrome:

```bash
# Enable background tasks
ENABLE_BACKGROUND_TASKS=true
SYNC_INTERVAL=30        # Minutes between syncs
```

### Custom Recommendation Algorithms

The integration supports multiple recommendation strategies:

- **Collaborative Filtering**: Based on user listening patterns
- **Content-Based**: Using audio features and metadata
- **Popularity-Based**: Trending tracks in your library
- **Discovery Mode**: Hidden gems from your collection

### Real-time Updates

Monitor Navidrome connection status:

```bash
# Health monitoring endpoint
curl http://localhost:8083/health

# Detailed status with connection info
curl http://localhost:8083/api/v1/navidrome/status
```

---

## 📚 Additional Resources

### Documentation

- [Navidrome Official Docs](https://www.navidrome.org/docs/)
- [Subsonic API Reference](http://www.subsonic.org/pages/api.jsp)
- [StepheyBot Music API Docs](docs/api/README.md)

### Community

- [Navidrome GitHub](https://github.com/navidrome/navidrome)
- [StepheyBot Music Issues](https://github.com/RM-Stephey/stepheybot-music/issues)

### Example Configurations

#### Docker Compose Integration

```yaml
version: '3.8'
services:
  navidrome:
    image: deluan/navidrome:latest
    ports:
      - "4533:4533"
    volumes:
      - ./music:/music:ro
      - ./data:/data
    
  stepheybot-music:
    depends_on:
      - navidrome
    environment:
      - NAVIDROME_URL=http://navidrome:4533
      - NAVIDROME_USERNAME=admin
      - NAVIDROME_PASSWORD=password
```

#### Production Setup

```bash
# High-performance configuration
NAVIDROME_URL=https://music.yourdomain.com
NAVIDROME_TIMEOUT=60
DB_MAX_CONNECTIONS=50
ENABLE_BACKGROUND_TASKS=true
SYNC_INTERVAL=15
RUST_LOG=warn
```

---

## 🎉 Success!

Your StepheyBot Music is now connected to your personal music library! 

Enjoy AI-powered recommendations based on your actual music collection, beautifully presented in the neon-themed interface designed for the cyberpunk aesthetic you love.

**Next Steps**:
1. Explore your personalized recommendations at http://localhost:5173
2. Test the API endpoints with your real music data
3. Create smart playlists from your collection
4. Monitor the beautiful neon dashboard showing your library stats

🎵 **Happy listening with your AI-powered music companion!** 🎵