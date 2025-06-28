# 🎵 StepheyBot Music - Transmission Integration Guide

## Overview

This guide provides complete documentation for integrating Transmission with StepheyBot Music Brain for automated music downloading. This replaces the previous qBittorrent setup due to authentication complexity issues.

## 🏗️ Architecture Overview

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│  StepheyBot Music   │◄──►│      Lidarr         │◄──►│     Jackett         │
│      Brain          │    │   (Management)      │    │   (Indexers)        │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
           │                           │                           │
           ▼                           ▼                           ▼
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│     Navidrome       │    │    Transmission     │    │    Gluetun VPN      │
│   (Music Server)    │    │   (Downloads)       │    │   (Network Proxy)   │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

## 🚀 Initial Setup & Configuration

### Step 1: Docker Compose Configuration

The Transmission service is configured in `docker-compose.yml`:

```yaml
# Transmission download client (routes through VPN)
transmission:
    image: linuxserver/transmission:latest
    container_name: stepheybot_music_transmission
    restart: unless-stopped
    network_mode: "service:gluetun" # Route through VPN
    depends_on:
        - gluetun
    environment:
        - PUID=1000
        - PGID=1000
        - TZ=UTC
        - USER=admin
        - PASS=adminadmin
    volumes:
        - transmission_data:/config
        - /mnt/nvme/upload:/downloads
        - /mnt/nvme/upload:/watch
```

### Step 2: VPN Container Port Configuration

Update gluetun service ports:

```yaml
gluetun:
    ports:
        - "9092:9091" # Transmission WebUI (external:internal)
        - "51413:51413" # Transmission torrenting
        - "51413:51413/udp"
```

**Note**: External port 9092 is used to avoid conflicts. Internal port remains 9091.

### Step 3: Directory Structure Setup

Create required directories with proper permissions:

```bash
# Create directories
sudo mkdir -p /mnt/nvme/upload/complete
sudo mkdir -p /mnt/nvme/upload/incomplete
sudo mkdir -p /mnt/nvme/upload/watch
sudo mkdir -p /mnt/nvme/apps/transmission

# Set proper ownership (PUID=1000, PGID=1000)
sudo chown -R 1000:1000 /mnt/nvme/upload/

# Set proper permissions
sudo chmod -R 755 /mnt/nvme/upload/
```

### Directory Mapping

```
Container Path → Host Path → Purpose
/downloads     → /mnt/nvme/upload           → Main downloads
/downloads/complete → /mnt/nvme/upload/complete → Completed downloads
/downloads/incomplete → /mnt/nvme/upload/incomplete → In-progress downloads
/watch         → /mnt/nvme/upload/watch     → Watch folder for .torrent files
/config        → /mnt/nvme/apps/transmission → Transmission configuration
```

## 🌐 Transmission WebUI Configuration

### Access Information
- **URL**: `http://localhost:9092`
- **Username**: `admin`
- **Password**: `adminadmin`

### Required Settings

#### Downloads Tab:
- **Download Directory**: `/downloads`
- **Incomplete Directory**: `/downloads/incomplete` (enable this)
- **Watch Directory**: `/watch` (enable this)
- **Call script when torrent is done**: Disabled

#### Network Tab:
- **Peer Port**: `51413`
- **Port Forwarding**: Enabled
- **µTP**: Enabled

#### Bandwidth Tab:
- **Upload Limit**: `1000 KB/s` (or as preferred)
- **Download Limit**: `Unlimited`
- **Priority**: Normal

#### Seeding Tab:
- **Stop seeding at ratio**: `2.0`
- **Stop seeding if idle for**: `30 minutes`

## 🎵 Lidarr Integration

### Download Client Configuration

In Lidarr (`http://localhost:8686`):

1. Go to **Settings** → **Download Clients**
2. Click **+** → **Transmission**
3. Configure:

```yaml
Name: Transmission Music
Enable: ✅
Host: stepheybot_music_vpn
Port: 9091
Use SSL: ❌
URL Base: /transmission/
Username: admin
Password: adminadmin
Category: [LEAVE EMPTY]
Post-Import Category: music-imported
Directory: [LEAVE EMPTY]
Recent Priority: Normal
Older Priority: Normal
Add Paused: No
Client Priority: 1
Tags: [LEAVE EMPTY]
```

### Download Handling Configuration

```yaml
Completed Download Handling: ✅ ON
Failed Download Handling: ✅ ON
Redownload Failed: ✅ ON
Redownload Failed from Interactive Search: ✅ ON
```

### Remote Path Mappings

**Recommended**: Leave empty (delete any existing mappings)

Both Lidarr and Transmission containers map the same host directory (`/mnt/nvme/upload`) to the same container path (`/downloads`), so path mapping is not required.

## 🔧 StepheyBot Music Brain Configuration

### Environment Variables

Update `docker-compose.yml` StepheyBot Music Brain service:

```yaml
# Transmission integration
- STEPHEYBOT__TRANSMISSION__URL=http://stepheybot_music_vpn:9091
- STEPHEYBOT__TRANSMISSION__USERNAME=admin
- STEPHEYBOT__TRANSMISSION__PASSWORD=adminadmin
```

### Code Configuration

Update `src/main.rs`:

```rust
let download_config = DownloadConfig {
    transmission_url: std::env::var("STEPHEYBOT__TRANSMISSION__URL")
        .unwrap_or_else(|_| "http://stepheybot_music_vpn:9091".to_string()),
    transmission_username: std::env::var("STEPHEYBOT__TRANSMISSION__USERNAME")
        .unwrap_or_else(|_| "admin".to_string()),
    transmission_password: std::env::var("STEPHEYBOT__TRANSMISSION__PASSWORD")
        .unwrap_or_else(|_| "adminadmin".to_string()),
    // ... other config
};
```

## 🧪 Testing & Verification

### 1. Container Health Check

```bash
# Check container status
docker ps | grep transmission
docker logs stepheybot_music_transmission --tail 10

# Expected logs (healthy):
# User UID:    1000
# User GID:    1000
# Connection to localhost (127.0.0.1) 9091 port [tcp/*] succeeded!
# [ls.io-init] done.
```

### 2. WebUI Access Test

```bash
# Test WebUI access
curl -s "http://localhost:9092" | head -3
# Expected: <h1>401: Unauthorized</h1> (normal when not logged in)
```

### 3. Lidarr Connection Test

```bash
# Test Lidarr → Transmission connection
curl -s -X POST "http://localhost:8686/api/v1/downloadclient/test" \
-H "X-Api-Key: YOUR_LIDARR_API_KEY" \
-H "Content-Type: application/json" \
-d '{
  "name": "Transmission Music",
  "implementation": "Transmission",
  "configContract": "TransmissionSettings",
  "implementationName": "Transmission",
  "priority": 1,
  "enable": true,
  "protocol": "torrent",
  "fields": [
    {"name": "host", "value": "stepheybot_music_vpn"},
    {"name": "port", "value": 9091},
    {"name": "username", "value": "admin"},
    {"name": "password", "value": "adminadmin"},
    {"name": "urlBase", "value": "/transmission/"}
  ]
}' | jq .

# Expected response: {} (empty object = success)
```

### 4. End-to-End Download Test

#### Manual Transmission Test:
1. Access `http://localhost:9092`
2. Login with `admin`/`adminadmin`
3. Add a test magnet link
4. Verify download starts and files appear in `/downloads`

#### Lidarr Integration Test:
1. Add a test artist in Lidarr
2. Trigger manual search
3. Verify torrent appears in Transmission
4. Verify completed files get imported to music library

#### StepheyBot Music Brain Test:
```bash
# Test download request
curl -X POST "http://localhost:8083/api/v1/download/request" \
     -H "Content-Type: application/json" \
     -d '{
       "title": "Test Download",
       "artist": "Test Artist",
       "album": "Test Album",
       "external_id": "magnet:?xt=urn:btih:...",
       "source": "test"
     }'

# Check download stats
curl "http://localhost:8083/api/v1/download/stats" | jq .
```

## 🛠️ Troubleshooting

### Common Issues & Solutions

#### 1. Permission Errors in Logs

**Symptoms**: 
```
stat: cannot statx '/downloads/complete': No such file or directory
**** Permissions could not be set ****
```

**Solution**:
```bash
# Recreate directories with proper permissions
sudo mkdir -p /mnt/nvme/upload/complete /mnt/nvme/upload/incomplete /mnt/nvme/upload/watch
sudo chown -R 1000:1000 /mnt/nvme/upload/
sudo chmod -R 755 /mnt/nvme/upload/
docker-compose restart transmission
```

#### 2. WebUI Environment Variable Error

**Symptoms**: "Changes Required! This image no longer bundles 3rd party Transmission UI packages"

**Solution**: Remove `TRANSMISSION_WEB_HOME` environment variable and recreate container:
```bash
docker-compose stop transmission
docker-compose rm -f transmission
docker-compose up -d transmission
```

#### 3. Port Conflicts

**Symptoms**: "failed to bind host port... address already in use"

**Solution**: Change external port mapping in `docker-compose.yml`:
```yaml
ports:
    - "9092:9091" # Use different external port
```

#### 4. Lidarr Cannot Find Completed Downloads

**Symptoms**: Downloads complete in Transmission but Lidarr doesn't import them

**Solutions**:
1. **Check volume mappings** - ensure both containers see same paths
2. **Add remote path mapping** if needed:
   - Host: `stepheybot_music_vpn`
   - Remote Path: `/downloads`
   - Local Path: `/downloads`
3. **Manual import** in Lidarr to test path visibility

#### 5. Authentication Issues

**Symptoms**: 401/403 errors when accessing Transmission

**Solutions**:
1. **Verify credentials** in both Transmission and Lidarr config
2. **Check environment variables** in docker-compose.yml
3. **Reset credentials** by recreating container

### Debug Commands

```bash
# Service logs
docker logs stepheybot_music_transmission --tail 20
docker logs stepheybot_music_lidarr --tail 20
docker logs stepheybot_music_brain --tail 20

# Container inspection
docker inspect stepheybot_music_transmission | grep -A 10 "Mounts"
docker inspect stepheybot_music_lidarr | grep -A 10 "Mounts"

# Network connectivity
docker exec stepheybot_music_brain timeout 5 bash -c "</dev/tcp/stepheybot_music_vpn/9091"

# Directory permissions
ls -la /mnt/nvme/upload/
```

## 📊 Performance Monitoring

### Key Metrics to Monitor

1. **Download Performance**:
   - Active downloads in Transmission WebUI
   - Download/upload speeds
   - Seeding ratios

2. **Storage Usage**:
   - `/mnt/nvme/upload` space usage
   - Download completion rates
   - Failed download cleanup

3. **Integration Health**:
   - Lidarr → Transmission connection status
   - StepheyBot Music Brain API response times
   - Import success rates

### Maintenance Tasks

**Daily**:
- Monitor download queue in Transmission
- Check Lidarr import activity
- Verify VPN connection status

**Weekly**:
- Clean up completed downloads
- Review seeding ratios
- Check storage usage trends

**Monthly**:
- Update container images
- Review failed downloads
- Optimize seeding settings

## 🔄 Migration from qBittorrent

### Pre-Migration Checklist

1. **Backup configurations**:
   ```bash
   docker cp stepheybot_music_qbittorrent:/data/config ./qbittorrent-backup
   ```

2. **Document current settings**:
   - Download directories
   - Seeding limits
   - Category configurations

### Migration Steps

1. **Stop qBittorrent services**:
   ```bash
   docker stop stepheybot_music_qbittorrent
   docker rm stepheybot_music_qbittorrent
   ```

2. **Update docker-compose.yml** with Transmission configuration

3. **Create Transmission directories** and set permissions

4. **Deploy Transmission**:
   ```bash
   docker-compose up -d transmission
   ```

5. **Reconfigure Lidarr** download clients

6. **Update StepheyBot Music Brain** codebase

7. **Test thoroughly** before removing qBittorrent volumes

### Post-Migration Cleanup

```bash
# Remove qBittorrent volumes (only after successful migration!)
docker volume rm nextcloud-modern_qbittorrent_data
sudo rm -rf /mnt/nvme/apps/qbittorrent
```

## 🚀 Advanced Configuration

### Custom Download Categories

Configure specialized handling in Transmission:

1. **Create category subdirectories**:
   ```bash
   mkdir -p /mnt/nvme/upload/music-high-priority
   mkdir -p /mnt/nvme/upload/music-archive
   ```

2. **Configure in Lidarr** using different download clients for different priorities

### Automated Cleanup Scripts

```bash
#!/bin/bash
# cleanup-downloads.sh

# Remove completed downloads older than 7 days
find /mnt/nvme/upload/complete -type f -mtime +7 -delete

# Clean up failed downloads
find /mnt/nvme/upload -name "*.part" -mtime +1 -delete

# Log cleanup activity
echo "$(date): Cleanup completed" >> /var/log/transmission-cleanup.log
```

### Enhanced Monitoring

Set up monitoring for:
- Download success rates
- Seeding health
- Storage usage alerts
- VPN connection status

## 📞 Support & Resources

- **Transmission Documentation**: https://github.com/transmission/transmission
- **LinuxServer.io Transmission**: https://docs.linuxserver.io/images/docker-transmission
- **Lidarr Wiki**: https://wiki.servarr.com/lidarr
- **StepheyBot Music Brain**: Internal documentation

---

*Last Updated: June 27, 2025*
*Version: 1.0.0*
*Migration: qBittorrent → Transmission Complete*