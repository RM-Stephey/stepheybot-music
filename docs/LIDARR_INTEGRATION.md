# 🎵 StepheyBot Music - Lidarr Integration Guide

## Overview

This guide provides complete documentation for integrating Lidarr with StepheyBot Music Brain for automated music discovery, downloading, and library management. The setup includes Lidarr, qBittorrent (through VPN), Jackett, and the StepheyBot Music Brain interface.

## 🏗️ Architecture Overview

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│  StepheyBot Music   │◄──►│      Lidarr         │◄──►│     Jackett         │
│      Brain          │    │   (Management)      │    │   (Indexers)        │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
           │                           │                           │
           ▼                           ▼                           ▼
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│     Navidrome       │    │    qBittorrent      │    │    Gluetun VPN      │
│   (Music Server)    │    │   (Downloads)       │    │   (Network Proxy)   │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

## 🚀 Initial Setup & Configuration

### Step 1: Verify Service Status

First, ensure all services are running:

```bash
# Check container status
docker ps | grep -E "(lidarr|qbittorrent|jackett|stepheybot_music|gluetun)"

# Verify StepheyBot Music Brain can communicate with Lidarr
curl http://localhost:8083/api/v1/lidarr/status | jq .
```

Expected response:
```json
{
  "lidarr_addon": {
    "connected": true,
    "enabled": true,
    "error": null,
    "features": [
      "artist_monitoring",
      "music_search", 
      "automatic_downloads",
      "library_management"
    ],
    "system_status": null,
    "url": "http://lidarr:8686",
    "version": "lidarr_addon_v1"
  }
}
```

### Step 2: Configure Lidarr API Access

1. **Get Lidarr API Key**:
   - Access Lidarr at `http://localhost:8686` or `manage.music.stepheybot.dev`
   - Go to Settings → General → Security
   - Copy the API Key

2. **Set Environment Variable**:
   ```bash
   # Add to your .env file or docker-compose.yml
   STEPHEYBOT__LIDARR__API_KEY=your_lidarr_api_key_here
   ```

3. **Restart StepheyBot Music Brain**:
   ```bash
   docker-compose restart stepheybot-music
   ```

### Step 3: Configure qBittorrent (Through VPN)

The qBittorrent client runs through the Gluetun VPN container for privacy and security.

1. **Access qBittorrent WebUI**:
   - URL: `http://localhost:8080`
   - Default credentials: `admin` / `adminadmin`

2. **Configure qBittorrent in Lidarr**:
   - Go to Lidarr → Settings → Download Clients
   - Add qBittorrent client:
     - **Name**: qBittorrent-VPN
     - **Host**: `gluetun` (container name)
     - **Port**: `8080`
     - **Username**: `admin`
     - **Password**: `adminadmin`
     - **Category**: `music`

3. **Verify Connection**:
   ```bash
   # Test qBittorrent through VPN
   curl -u admin:adminadmin http://localhost:8080/api/v2/app/version
   ```

### Step 4: Configure Jackett Indexers

1. **Access Jackett**:
   - URL: `http://localhost:9117`
   - Configure music-focused indexers (1337x, RARBG, etc.)

2. **Add Indexers to Lidarr**:
   - In Lidarr → Settings → Indexers
   - Add Torznab indexers from Jackett
   - Use Jackett's API URLs and keys

## 🔄 Download Workflow

### Complete End-to-End Process

1. **Search for Music** (via StepheyBot Music Brain):
   ```bash
   curl "http://localhost:8083/api/v1/search/global/artist%20name"
   ```

2. **Request Download** (triggers Lidarr):
   ```bash
   curl -X POST "http://localhost:8083/api/v1/download/request" \
        -H "Content-Type: application/json" \
        -d '{
          "query": "Artist Name - Album Name",
          "type": "album",
          "musicbrainz_id": "optional_mb_id"
        }'
   ```

3. **Monitor Download Status**:
   ```bash
   # Check active downloads
   curl "http://localhost:8083/api/v1/download/active"
   
   # Check download stats
   curl "http://localhost:8083/api/v1/download/stats"
   ```

### Download Flow Diagram

```
User Request → StepheyBot Music Brain → Lidarr → Jackett → qBittorrent → File System
     ↓                    ↓                ↓         ↓          ↓           ↓
  Frontend UI        API Processing    Artist Add   Torrent    Download    Import
                                                    Search     via VPN     to Library
```

## 📁 Storage & File Management

### Directory Structure

The system uses a tiered storage approach:

```
/mnt/nvme/upload/          # Hot downloads (NVMe SSD)
├── processing/            # Active downloads
├── completed/             # Completed, pending import
└── temp/                  # Temporary files

/mnt/hdd/media/music/      # Cold storage (HDD)
├── library/               # Final music library
└── backup/                # Archive/backup copies
```

### File Processing Flow

1. **Download Phase**: Files download to `/mnt/nvme/upload/processing/`
2. **Completion Phase**: Moved to `/mnt/nvme/upload/completed/`
3. **Import Phase**: Lidarr imports to `/mnt/hdd/media/music/library/`
4. **Cleanup Phase**: Processed files moved to cold storage

## 🔧 Configuration Details

### StepheyBot Music Brain Environment Variables

```bash
# Lidarr Integration
STEPHEYBOT__LIDARR__URL=http://lidarr:8686
STEPHEYBOT__LIDARR__API_KEY=your_api_key

# File Paths
STEPHEYBOT__PATHS__MUSIC_PATH=/music
STEPHEYBOT__PATHS__DOWNLOAD_PATH=/hot_downloads
STEPHEYBOT__PATHS__COLD_DOWNLOAD_PATH=/cold_downloads
STEPHEYBOT__PATHS__PROCESSING_PATH=/processing

# Download Settings
STEPHEYBOT__DOWNLOAD__MAX_CONCURRENT=3
STEPHEYBOT__DOWNLOAD__CLEANUP_INTERVAL=3600
```

### Lidarr Configuration

Key settings for optimal integration:

1. **Media Management**:
   - Root Folder: `/music`
   - Rename Files: Yes
   - Replace Illegal Characters: Yes

2. **Download Clients**:
   - qBittorrent: `gluetun:8080`
   - Category: `music`
   - Import Mode: Move

3. **Indexers**:
   - All configured Jackett indexers
   - API limits respected

## 🧪 Testing & Verification

### Integration Tests

1. **Test Lidarr Connection**:
   ```bash
   curl http://localhost:8083/api/v1/lidarr/status
   ```

2. **Test Download Request**:
   ```bash
   curl -X POST http://localhost:8083/api/v1/download/request \
        -H "Content-Type: application/json" \
        -d '{"query": "test artist", "type": "artist"}'
   ```

3. **Verify qBittorrent Access**:
   ```bash
   # Through VPN (internal)
   docker exec stepheybot_music_brain curl http://gluetun:8080/api/v2/app/version
   
   # Direct access (external)
   curl -u admin:adminadmin http://localhost:8080/api/v2/app/version
   ```

### Manual Testing Checklist

- [ ] StepheyBot Music Brain starts successfully
- [ ] Lidarr API connection established
- [ ] qBittorrent accessible through VPN
- [ ] Jackett indexers configured
- [ ] Download request creates Lidarr job
- [ ] Download progresses in qBittorrent
- [ ] Completed files imported to library
- [ ] Navidrome detects new music

## 🛠️ Troubleshooting

### Common Issues & Solutions

#### 1. Lidarr Connection Failed

**Symptoms**: `lidarr_addon.connected: false`

**Solutions**:
```bash
# Check Lidarr service
docker logs stepheybot_music_lidarr

# Verify API key
echo $STEPHEYBOT__LIDARR__API_KEY

# Test direct connection
curl -H "X-Api-Key: $STEPHEYBOT__LIDARR__API_KEY" http://localhost:8686/api/v1/system/status
```

#### 2. qBittorrent Authentication Failed

**Symptoms**: Download requests fail, qBittorrent shows auth errors

**Solutions**:
```bash
# Reset qBittorrent password
docker exec -it stepheybot_music_qbittorrent /bin/sh
# Inside container: reset admin password

# Or use the script
./fix-qbittorrent-auth.sh
```

#### 3. Downloads Stuck in Queue

**Symptoms**: Downloads show as "queued" but never start

**Solutions**:
```bash
# Check VPN connection
docker exec stepheybot_music_vpn curl ifconfig.me

# Verify qBittorrent can reach indexers
docker logs stepheybot_music_qbittorrent

# Check Lidarr download client
curl -H "X-Api-Key: $API_KEY" http://localhost:8686/api/v1/downloadclient
```

#### 4. File Import Issues

**Symptoms**: Downloads complete but files not imported to library

**Solutions**:
```bash
# Check file permissions
ls -la /mnt/nvme/upload/
ls -la /mnt/hdd/media/music/

# Verify Lidarr import settings
# Check Activity tab in Lidarr UI

# Force library scan
curl -H "X-Api-Key: $API_KEY" -X POST http://localhost:8686/api/v1/command \
     -d '{"name": "RefreshArtist", "artistId": 1}'
```

### Debug Commands

```bash
# Service logs
docker-compose logs -f stepheybot-music
docker-compose logs -f lidarr
docker-compose logs -f qbittorrent
docker-compose logs -f gluetun

# API health checks
curl http://localhost:8083/health
curl http://localhost:8083/api/v1/stats
curl http://localhost:8083/api/v1/lidarr/status

# Storage checks  
df -h /mnt/nvme/upload
df -h /mnt/hdd/media
```

## 📊 Performance Monitoring

### Key Metrics to Monitor

1. **Download Performance**:
   - Active downloads: `GET /api/v1/download/active`
   - Download speed: Monitor qBittorrent
   - Success rate: `GET /api/v1/download/stats`

2. **Storage Usage**:
   - Hot storage: `/mnt/nvme/upload`
   - Cold storage: `/mnt/hdd/media`
   - Processing queue: Check for stuck files

3. **Service Health**:
   - All containers running and healthy
   - VPN connection active
   - API response times < 2s

### Maintenance Tasks

**Daily**:
- Check download queue status
- Verify VPN connection
- Monitor storage usage

**Weekly**:
- Clean up completed downloads
- Update indexer configurations
- Review failed downloads

**Monthly**:
- Update service images
- Backup configuration
- Performance optimization review

## 🚀 Advanced Configuration

### Custom Download Categories

Configure specialized download handling:

```yaml
# In Lidarr Settings → Download Clients
categories:
  music: Default music downloads
  music-high: High priority releases
  music-archive: Archive/collection downloads
```

### Automated Cleanup Scripts

```bash
#!/bin/bash
# cleanup-downloads.sh

# Remove completed downloads older than 7 days
find /mnt/nvme/upload/completed -type f -mtime +7 -delete

# Move large files to cold storage
find /mnt/nvme/upload -size +1G -exec mv {} /mnt/hdd/downloads/ \;

# Clean temporary files
rm -rf /mnt/nvme/upload/temp/*
```

### Quality Profiles

Optimize for different use cases:

1. **Lossless**: FLAC preferred, 1000+ kbps
2. **High Quality**: MP3 320kbps or higher
3. **Standard**: MP3 192-320kbps
4. **Mobile**: MP3 128-192kbps

## 📋 Next Steps

### Immediate Improvements

1. **Enhanced Monitoring**:
   - Add Grafana dashboards
   - Set up alerting for failed downloads
   - Monitor storage usage trends

2. **User Experience**:
   - Add download progress indicators
   - Implement download queue management
   - Create mobile-responsive interface

3. **Integration Enhancements**:
   - Support for additional indexers
   - Custom quality profiles per user
   - Automated music discovery based on listening habits

### Long-term Goals

- Machine learning-based music recommendations
- Integration with streaming services for discovery
- Advanced file organization and tagging
- Multi-user support with individual preferences

## 📞 Support & Resources

- **Project Repository**: StepheyBot Music Brain
- **Issue Tracker**: GitHub Issues
- **Configuration Examples**: `/docs/examples/`
- **API Documentation**: `/docs/API_REFERENCE.md`

---

*Last Updated: June 27, 2025*
*Version: 1.0.0*