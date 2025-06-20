# 🎵 StepheyBot Music API Documentation

Complete reference for the StepheyBot Music REST API endpoints.

## 📋 Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [Response Format](#response-format)
- [Error Handling](#error-handling)
- [Endpoints](#endpoints)
  - [Health & Status](#health--status)
  - [Music Recommendations](#music-recommendations)
  - [Library Management](#library-management)
  - [User Management](#user-management)
  - [Playlist Management](#playlist-management)
  - [Admin Endpoints](#admin-endpoints)

## Overview

The StepheyBot Music API provides a RESTful interface for accessing music recommendations, library management, and user data. All endpoints return JSON responses and support standard HTTP methods.

### API Version
- **Current Version**: v1
- **Base Path**: `/api/v1`
- **Protocol**: HTTP/HTTPS
- **Format**: JSON

## Base URL

```
http://localhost:8083
```

**Production URL** (when deployed):
```
https://your-domain.com
```

## Authentication

Currently, the API operates without authentication for development purposes. Future versions will include:

- 🔐 **OAuth2** integration
- 🎫 **JWT tokens** for session management
- 🔑 **API keys** for service-to-service communication

## Response Format

All API responses follow a consistent JSON structure:

### Success Response
```json
{
  "status": "success",
  "data": { ... },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### Error Response
```json
{
  "status": "error",
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "The requested resource was not found",
    "details": { ... }
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

## Error Handling

### HTTP Status Codes

| Code | Status | Description |
|------|--------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Authentication required |
| 403 | Forbidden | Access denied |
| 404 | Not Found | Resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Service temporarily unavailable |

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_PARAMETER` | Request parameter is invalid |
| `RESOURCE_NOT_FOUND` | Requested resource does not exist |
| `DATABASE_ERROR` | Database operation failed |
| `EXTERNAL_SERVICE_ERROR` | External service (Navidrome/Lidarr) error |
| `RATE_LIMIT_EXCEEDED` | Too many requests |

---

## Endpoints

## Health & Status

### Health Check
Check if the service is healthy and operational.

**Endpoint**: `GET /health`

**Response**:
```json
{
  "status": "healthy",
  "service": "stepheybot-music",
  "version": "0.1.0",
  "checks": {
    "database": "✅ connected",
    "recommendations": "✅ operational"
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### Readiness Check
Check if the service is ready to accept requests.

**Endpoint**: `GET /health/ready`

**Response**:
```json
{
  "status": "ready",
  "checks": {
    "database": "✅ connected",
    "sample_data": "✅ loaded"
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### Liveness Check
Simple liveness probe for container orchestration.

**Endpoint**: `GET /health/live`

**Response**:
```json
{
  "status": "alive",
  "uptime": "running",
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### API Status
Get detailed API status and feature information.

**Endpoint**: `GET /api/v1/status`

**Response**:
```json
{
  "api_version": "v1",
  "service": "StepheyBot Music",
  "version": "0.1.0",
  "status": "operational",
  "features": {
    "database_integration": "✅ active",
    "recommendation_engine": "✅ functional",
    "listening_history": "✅ tracking",
    "user_profiles": "✅ managed",
    "audio_analysis": "✅ available"
  },
  "statistics": {
    "total_tracks": 10,
    "total_albums": 5,
    "total_artists": 5,
    "total_users": 3,
    "total_listening_history": 45,
    "database_size_mb": 1.5,
    "last_updated": "2025-06-20T05:15:00.000Z"
  },
  "algorithms": [
    "collaborative_filtering",
    "content_based_filtering",
    "popularity_based",
    "audio_feature_matching",
    "listening_pattern_analysis"
  ],
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

---

## Music Recommendations

### Get User Recommendations
Get personalized music recommendations for a specific user.

**Endpoint**: `GET /api/v1/recommendations/{user_id}`

**Parameters**:
- `user_id` (path, required): User identifier
- `limit` (query, optional): Number of recommendations (default: 20, max: 50)
- `offset` (query, optional): Pagination offset (default: 0)
- `genre` (query, optional): Filter by genre
- `mood` (query, optional): Filter by mood

**Example Request**:
```bash
GET /api/v1/recommendations/user1?limit=10&genre=Synthwave
```

**Response**:
```json
{
  "recommendations": [
    {
      "track_id": "track1",
      "title": "Midnight Drive",
      "artist": "Synthwave Collective",
      "album": "Neon Horizons",
      "score": 0.95,
      "reason": "Matches your preference for Synthwave music",
      "recommendation_type": "collaborative_content",
      "duration": 245,
      "year": null,
      "genre": "Synthwave",
      "play_count": 1250,
      "user_rating": 4.8
    }
  ],
  "total": 10,
  "offset": 0,
  "limit": 10,
  "algorithm": "hybrid_collaborative_content",
  "generated_at": "2025-06-20T05:15:00.000Z"
}
```

### Get Trending Tracks
Get currently trending tracks based on play counts and ratings.

**Endpoint**: `GET /api/v1/recommendations/trending`

**Response**:
```json
{
  "trending": [
    {
      "track_id": "track1",
      "title": "Midnight Drive",
      "artist": "Synthwave Collective",
      "album": "Neon Horizons",
      "score": 0.9,
      "reason": "Popular track with high ratings",
      "recommendation_type": "popularity_based",
      "duration": 245,
      "genre": "Synthwave",
      "play_count": 1250,
      "user_rating": 4.8
    }
  ],
  "period": "last_7_days",
  "algorithm": "play_count_weighted",
  "generated_at": "2025-06-20T05:15:00.000Z"
}
```

### Get Discovery Tracks
Get hidden gems and lesser-known high-quality tracks.

**Endpoint**: `GET /api/v1/recommendations/discover`

**Response**:
```json
{
  "discovery": [
    {
      "track_id": "track5",
      "title": "Floating Cosmos",
      "artist": "Digital Dreams",
      "album": "Digital Meditation",
      "score": 0.85,
      "reason": "Hidden gem - high quality, underplayed track",
      "recommendation_type": "discovery",
      "duration": 456,
      "genre": "Ambient",
      "play_count": 342,
      "user_rating": 4.9
    }
  ],
  "criteria": "high_quality_low_plays",
  "algorithm": "hidden_gems",
  "generated_at": "2025-06-20T05:15:00.000Z"
}
```

---

## Library Management

### Get Library Statistics
Get comprehensive statistics about the music library.

**Endpoint**: `GET /api/v1/library/stats`

**Response**:
```json
{
  "total_tracks": 10,
  "total_albums": 5,
  "total_artists": 5,
  "total_users": 3,
  "total_listening_history": 45,
  "database_size_mb": 1.5,
  "last_updated": "2025-06-20T05:15:00.000Z"
}
```

### Search Library
Search for tracks, albums, and artists in the library.

**Endpoint**: `GET /api/v1/library/search`

**Parameters**:
- `q` (query, required): Search query string

**Example Request**:
```bash
GET /api/v1/library/search?q=synthwave
```

**Response**:
```json
{
  "results": {
    "tracks": [
      {
        "id": "track1",
        "title": "Midnight Drive",
        "artist": "Synthwave Collective",
        "album": "Neon Horizons",
        "duration": 245,
        "genre": "Synthwave"
      }
    ],
    "albums": [
      {
        "id": "album1",
        "title": "Neon Horizons",
        "artist": "Synthwave Collective",
        "year": 2023,
        "track_count": 12
      }
    ],
    "artists": [
      {
        "id": "artist1",
        "name": "Synthwave Collective",
        "genre": "Synthwave",
        "country": "USA"
      }
    ]
  },
  "query": "synthwave",
  "total_results": 3
}
```

---

## User Management

### List Users
Get list of all users in the system.

**Endpoint**: `GET /api/v1/users`

**Response**:
```json
{
  "users": [
    {
      "id": "user1",
      "username": "stephey",
      "email": "stephey@stepheybot.dev",
      "created_at": "2025-06-20T05:15:00.000Z",
      "last_active": "2025-06-20T05:15:00.000Z"
    }
  ],
  "total": 3
}
```

### Get User Listening History
Get listening history for a specific user.

**Endpoint**: `GET /api/v1/users/{user_id}/history`

**Parameters**:
- `user_id` (path, required): User identifier

**Response**:
```json
{
  "user_id": "user1",
  "history": [
    {
      "track": {
        "id": "track1",
        "title": "Midnight Drive",
        "artist": "Synthwave Collective",
        "album": "Neon Horizons"
      },
      "played_at": "2025-06-20T05:15:00.000Z",
      "duration_played": 240,
      "completed": true
    }
  ],
  "total": 25
}
```

---

## Playlist Management

### Generate Smart Playlist
Create a smart playlist based on specified criteria.

**Endpoint**: `POST /api/v1/playlists/generate`

**Request Body**:
```json
{
  "name": "Evening Vibes",
  "description": "Relaxing tracks for evening listening",
  "duration_minutes": 60
}
```

**Response**:
```json
{
  "playlist": {
    "name": "Evening Vibes",
    "description": "Relaxing tracks for evening listening",
    "track_count": 15,
    "total_duration_seconds": 3600,
    "total_duration_minutes": 60,
    "tracks": [
      {
        "track_id": "track5",
        "title": "Floating Cosmos",
        "artist": "Digital Dreams",
        "album": "Digital Meditation",
        "score": 0.8,
        "reason": "Selected for 'Evening Vibes' playlist",
        "recommendation_type": "playlist_generation",
        "duration": 456,
        "genre": "Ambient"
      }
    ]
  },
  "algorithm": "mood_and_energy_matching",
  "status": "generated",
  "generated_at": "2025-06-20T05:15:00.000Z"
}
```

---

## Admin Endpoints

### System Information
Get detailed system information and configuration.

**Endpoint**: `GET /admin/system`

**Response**:
```json
{
  "system": {
    "service": "StepheyBot Music",
    "version": "0.1.0",
    "status": "operational",
    "environment": "development",
    "features": [
      "✅ SQLite database",
      "✅ AI recommendations",
      "✅ User tracking",
      "✅ Audio analysis",
      "✅ Smart playlists"
    ]
  },
  "library": {
    "total_tracks": 10,
    "total_albums": 5,
    "total_artists": 5,
    "total_users": 3,
    "total_listening_history": 45,
    "database_size_mb": 1.5,
    "last_updated": "2025-06-20T05:15:00.000Z"
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### Database Information
Get detailed database statistics and performance metrics.

**Endpoint**: `GET /admin/database`

**Response**:
```json
{
  "database": {
    "type": "SQLite",
    "status": "connected",
    "size_mb": 1.5,
    "tables": {
      "users": 3,
      "tracks": 10,
      "albums": 5,
      "artists": 5,
      "listening_history": 45
    }
  },
  "performance": {
    "avg_query_time": "< 1ms",
    "connection_pool": "optimal"
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

### System Statistics
Get comprehensive system performance and usage statistics.

**Endpoint**: `GET /api/v1/stats`

**Response**:
```json
{
  "system": {
    "service": "StepheyBot Music",
    "version": "0.1.0",
    "status": "operational",
    "uptime": "running"
  },
  "library": {
    "total_tracks": 10,
    "total_albums": 5,
    "total_artists": 5,
    "total_users": 3,
    "total_listening_history": 45,
    "database_size_mb": 1.5,
    "last_updated": "2025-06-20T05:15:00.000Z"
  },
  "timestamp": "2025-06-20T05:15:00.000Z"
}
```

---

## Rate Limiting

Currently, no rate limiting is implemented, but future versions will include:

- **General API**: 1000 requests/hour per IP
- **Recommendations**: 100 requests/hour per user
- **Search**: 200 requests/hour per IP
- **Admin endpoints**: 50 requests/hour per authenticated user

## WebSocket Support

Future versions will include WebSocket endpoints for:

- 🎵 **Real-time playback status**
- 📊 **Live recommendation updates**
- 🔄 **Library sync notifications**
- 💬 **User activity feeds**

## SDK & Client Libraries

Planned client libraries:
- 🦀 **Rust** - Native client library
- 🟨 **JavaScript/TypeScript** - Web client
- 🐍 **Python** - Data analysis integration
- 📱 **Flutter/Dart** - Mobile applications

## Examples & Testing

For complete API testing examples, see:
- [`examples.md`](examples.md) - Detailed usage examples
- [`postman.json`](postman.json) - Postman collection
- [`curl-examples.sh`](../setup/curl-examples.sh) - Shell script examples

---

**Last Updated**: 2025-06-20  
**API Version**: v1.0.0  
**Documentation Version**: 1.0