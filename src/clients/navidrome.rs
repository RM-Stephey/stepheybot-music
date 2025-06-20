//! Navidrome API client for StepheyBot Music
//!
//! This module provides a comprehensive client for interacting with the Navidrome music server
//! using the Subsonic API protocol. It handles authentication, rate limiting, and provides
//! type-safe methods for all major operations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::time::{sleep, Instant};
use tracing::{debug, error, info, warn};
use url::Url;

/// Navidrome API client
#[derive(Clone)]
pub struct NavidromeClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
    salt: String,
    token: String,
    last_request: std::sync::Arc<tokio::sync::Mutex<Option<Instant>>>,
    rate_limit_delay: Duration,
}

/// Subsonic API response wrapper
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponse<T> {
    pub subsonic_response: SubsonicResponseInner<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponseInner<T> {
    pub status: String,
    pub version: String,
    pub error: Option<SubsonicError>,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct SubsonicError {
    pub code: u32,
    pub message: String,
}

/// User information from Navidrome
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeUser {
    pub username: String,
    pub email: Option<String>,
    pub scrobbling_enabled: Option<bool>,
    pub admin_role: Option<bool>,
    pub settings_role: Option<bool>,
    pub stream_role: Option<bool>,
    pub jukebox_role: Option<bool>,
    pub download_role: Option<bool>,
    pub upload_role: Option<bool>,
    pub playlist_role: Option<bool>,
    pub cover_art_role: Option<bool>,
    pub comment_role: Option<bool>,
    pub podcast_role: Option<bool>,
    pub share_role: Option<bool>,
    pub video_conversion_role: Option<bool>,
    pub music_folder_id: Option<String>,
    pub max_bit_rate: Option<u32>,
    pub avatar_last_changed: Option<String>,
}

/// Artist information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeArtist {
    pub id: String,
    pub name: String,
    pub album_count: Option<u32>,
    pub starred: Option<String>,
    pub user_rating: Option<u32>,
    pub average_rating: Option<f64>,
    pub art_id: Option<String>,
    pub biography: Option<String>,
    pub music_brainz_id: Option<String>,
    pub last_fm_url: Option<String>,
    pub small_image_url: Option<String>,
    pub medium_image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub similar_artist: Option<Vec<SimilarArtist>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarArtist {
    pub id: String,
    pub name: String,
}

/// Album information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeAlbum {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    pub play_count: Option<u64>,
    pub created: Option<String>,
    pub starred: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub user_rating: Option<u32>,
    pub average_rating: Option<f64>,
    pub music_brainz_id: Option<String>,
    pub sort_name: Option<String>,
    pub order_tag: Option<String>,
    pub size: Option<u64>,
    pub disc_titles: Option<Vec<DiscTitle>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscTitle {
    pub disc: u32,
    pub title: String,
}

/// Song/Track information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromeSong {
    pub id: String,
    pub parent: Option<String>,
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub is_dir: Option<bool>,
    pub cover_art: Option<String>,
    pub created: Option<String>,
    pub duration: Option<u32>,
    pub bit_rate: Option<u32>,
    pub track: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub size: Option<u64>,
    pub suffix: Option<String>,
    pub content_type: Option<String>,
    pub is_video: Option<bool>,
    pub path: Option<String>,
    pub play_count: Option<u64>,
    pub starred: Option<String>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub r#type: Option<String>,
    pub bookmark_position: Option<u64>,
    pub original_width: Option<u32>,
    pub original_height: Option<u32>,
    pub user_rating: Option<u32>,
    pub average_rating: Option<f64>,
    pub music_brainz_id: Option<String>,
    pub channels: Option<u32>,
    pub comment: Option<String>,
    pub sort_name: Option<String>,
    pub media_type: Option<String>,
    pub bpm: Option<u32>,
    pub lyrics: Option<String>,
}

/// Playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavidromePlaylist {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub public: Option<bool>,
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    pub created: Option<String>,
    pub changed: Option<String>,
    pub cover_art: Option<String>,
    pub allowed_user: Option<Vec<String>>,
    pub rules: Option<String>,
    pub entry: Option<Vec<NavidromeSong>>,
}

/// Scrobble entry for listening history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrobbleEntry {
    pub id: String,
    pub time: u64, // Unix timestamp in milliseconds
    pub submission: Option<bool>,
}

/// Now playing information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingEntry {
    pub username: String,
    pub minutes_ago: u32,
    pub player_id: String,
    pub player_name: Option<String>,
    #[serde(flatten)]
    pub song: NavidromeSong,
}

/// API response containers
#[derive(Debug, Deserialize)]
pub struct UsersResponse {
    pub users: Users,
}

#[derive(Debug, Deserialize)]
pub struct Users {
    pub user: Vec<NavidromeUser>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistsResponse {
    pub artists: Artists,
}

#[derive(Debug, Deserialize)]
pub struct Artists {
    pub index: Vec<ArtistIndex>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistIndex {
    pub name: String,
    pub artist: Vec<NavidromeArtist>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumsResponse {
    #[serde(rename = "albumList2")]
    pub album_list2: AlbumList,
}

#[derive(Debug, Deserialize)]
pub struct AlbumList {
    pub album: Vec<NavidromeAlbum>,
}

#[derive(Debug, Deserialize)]
pub struct SongsResponse {
    #[serde(rename = "searchResult3")]
    pub search_result3: Option<SearchResult>,
    pub directory: Option<Directory>,
    #[serde(rename = "randomSongs")]
    pub random_songs: Option<RandomSongs>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub song: Option<Vec<NavidromeSong>>,
    pub album: Option<Vec<NavidromeAlbum>>,
    pub artist: Option<Vec<NavidromeArtist>>,
}

#[derive(Debug, Deserialize)]
pub struct Directory {
    pub child: Vec<NavidromeSong>,
}

#[derive(Debug, Deserialize)]
pub struct RandomSongs {
    pub song: Vec<NavidromeSong>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistsResponse {
    pub playlists: Playlists,
}

#[derive(Debug, Deserialize)]
pub struct Playlists {
    pub playlist: Vec<NavidromePlaylist>,
}

#[derive(Debug, Deserialize)]
pub struct NowPlayingResponse {
    #[serde(rename = "nowPlaying")]
    pub now_playing: NowPlaying,
}

#[derive(Debug, Deserialize)]
pub struct NowPlaying {
    pub entry: Vec<NowPlayingEntry>,
}

impl NavidromeClient {
    /// Create a new Navidrome API client
    pub fn new(base_url: &str, username: &str, password: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("StepheyBot-Music/1.0")
            .build()
            .context("Failed to create HTTP client")?;

        // Generate salt for authentication
        let salt = uuid::Uuid::new_v4().to_string().replace("-", "");

        // Create token (MD5 hash of password + salt)
        let token_input = format!("{}{}", password, salt);
        let token = format!("{:x}", md5::compute(token_input.as_bytes()));

        let mut base_url = base_url.to_string();
        if !base_url.ends_with('/') {
            base_url.push('/');
        }

        Ok(Self {
            client,
            base_url,
            username: username.to_string(),
            password: password.to_string(),
            salt,
            token,
            last_request: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            rate_limit_delay: Duration::from_millis(100),
        })
    }

    /// Perform health check on the Navidrome server
    pub async fn health_check(&self) -> Result<()> {
        let response = self.make_request("ping", &[]).await?;

        if response.subsonic_response.status != "ok" {
            anyhow::bail!(
                "Navidrome health check failed: {:?}",
                response.subsonic_response.error
            );
        }

        Ok(())
    }

    /// Get all users from Navidrome
    pub async fn get_users(&self) -> Result<Vec<NavidromeUser>> {
        let response: SubsonicResponse<UsersResponse> = self.make_request("getUsers", &[]).await?;

        match response.subsonic_response.data {
            Some(data) => Ok(data.users.user),
            None => Ok(Vec::new()),
        }
    }

    /// Get user information by username
    pub async fn get_user(&self, username: &str) -> Result<Option<NavidromeUser>> {
        let users = self.get_users().await?;
        Ok(users.into_iter().find(|u| u.username == username))
    }

    /// Get all artists
    pub async fn get_artists(&self) -> Result<Vec<NavidromeArtist>> {
        let response: SubsonicResponse<ArtistsResponse> =
            self.make_request("getArtists", &[]).await?;

        match response.subsonic_response.data {
            Some(data) => {
                let mut artists = Vec::new();
                for index in data.artists.index {
                    artists.extend(index.artist);
                }
                Ok(artists)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Get artist information by ID
    pub async fn get_artist(&self, artist_id: &str) -> Result<Option<NavidromeArtist>> {
        let params = [("id", artist_id)];
        let response: SubsonicResponse<ArtistsResponse> =
            self.make_request("getArtist", &params).await?;

        // Note: This is a simplification - the actual getArtist response structure may differ
        match response.subsonic_response.data {
            Some(data) => Ok(data
                .artists
                .index
                .into_iter()
                .flat_map(|idx| idx.artist)
                .find(|a| a.id == artist_id)),
            None => Ok(None),
        }
    }

    /// Get albums with optional filtering
    pub async fn get_albums(
        &self,
        album_type: Option<&str>,
        size: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<NavidromeAlbum>> {
        let mut params = Vec::new();

        if let Some(t) = album_type {
            params.push(("type", t));
        }
        if let Some(s) = size {
            let s_str = s.to_string();
            params.push(("size", &s_str));
        }
        if let Some(o) = offset {
            let o_str = o.to_string();
            params.push(("offset", &o_str));
        }

        let response: SubsonicResponse<AlbumsResponse> =
            self.make_request("getAlbumList2", &params).await?;

        match response.subsonic_response.data {
            Some(data) => Ok(data.album_list2.album),
            None => Ok(Vec::new()),
        }
    }

    /// Get recently added albums
    pub async fn get_recent_albums(&self, count: u32) -> Result<Vec<NavidromeAlbum>> {
        self.get_albums(Some("newest"), Some(count), None).await
    }

    /// Get random songs
    pub async fn get_random_songs(
        &self,
        size: Option<u32>,
        genre: Option<&str>,
        from_year: Option<u32>,
        to_year: Option<u32>,
    ) -> Result<Vec<NavidromeSong>> {
        let mut params = Vec::new();

        if let Some(s) = size {
            let s_str = s.to_string();
            params.push(("size", &s_str));
        }
        if let Some(g) = genre {
            params.push(("genre", g));
        }
        if let Some(fy) = from_year {
            let fy_str = fy.to_string();
            params.push(("fromYear", &fy_str));
        }
        if let Some(ty) = to_year {
            let ty_str = ty.to_string();
            params.push(("toYear", &ty_str));
        }

        let response: SubsonicResponse<SongsResponse> =
            self.make_request("getRandomSongs", &params).await?;

        match response.subsonic_response.data.and_then(|d| d.random_songs) {
            Some(random_songs) => Ok(random_songs.song),
            None => Ok(Vec::new()),
        }
    }

    /// Search for music
    pub async fn search(
        &self,
        query: &str,
        artist_count: Option<u32>,
        album_count: Option<u32>,
        song_count: Option<u32>,
    ) -> Result<SearchResult> {
        let mut params = vec![("query", query)];

        if let Some(ac) = artist_count {
            let ac_str = ac.to_string();
            params.push(("artistCount", &ac_str));
        }
        if let Some(alc) = album_count {
            let alc_str = alc.to_string();
            params.push(("albumCount", &alc_str));
        }
        if let Some(sc) = song_count {
            let sc_str = sc.to_string();
            params.push(("songCount", &sc_str));
        }

        let response: SubsonicResponse<SongsResponse> =
            self.make_request("search3", &params).await?;

        match response
            .subsonic_response
            .data
            .and_then(|d| d.search_result3)
        {
            Some(result) => Ok(result),
            None => Ok(SearchResult {
                song: None,
                album: None,
                artist: None,
            }),
        }
    }

    /// Get user playlists
    pub async fn get_playlists(&self, username: Option<&str>) -> Result<Vec<NavidromePlaylist>> {
        let mut params = Vec::new();
        if let Some(u) = username {
            params.push(("username", u));
        }

        let response: SubsonicResponse<PlaylistsResponse> =
            self.make_request("getPlaylists", &params).await?;

        match response.subsonic_response.data {
            Some(data) => Ok(data.playlists.playlist),
            None => Ok(Vec::new()),
        }
    }

    /// Create a new playlist
    pub async fn create_playlist(
        &self,
        name: &str,
        song_ids: &[String],
    ) -> Result<NavidromePlaylist> {
        let mut params = vec![("name", name)];

        // Add song IDs as separate parameters
        let song_id_strings: Vec<String> = song_ids.iter().map(|id| id.clone()).collect();
        for song_id in &song_id_strings {
            params.push(("songId", song_id));
        }

        let response: SubsonicResponse<PlaylistsResponse> =
            self.make_request("createPlaylist", &params).await?;

        match response.subsonic_response.data {
            Some(data) => data
                .playlists
                .playlist
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Failed to create playlist")),
            None => Err(anyhow::anyhow!("Failed to create playlist")),
        }
    }

    /// Update an existing playlist
    pub async fn update_playlist(
        &self,
        playlist_id: &str,
        name: Option<&str>,
        comment: Option<&str>,
        public: Option<bool>,
        song_ids_to_add: &[String],
        song_indices_to_remove: &[u32],
    ) -> Result<()> {
        let mut params = vec![("playlistId", playlist_id)];

        if let Some(n) = name {
            params.push(("name", n));
        }
        if let Some(c) = comment {
            params.push(("comment", c));
        }
        if let Some(p) = public {
            params.push(("public", if p { "true" } else { "false" }));
        }

        // Add songs to add
        for song_id in song_ids_to_add {
            params.push(("songIdToAdd", song_id));
        }

        // Add song indices to remove
        let index_strings: Vec<String> = song_indices_to_remove
            .iter()
            .map(|i| i.to_string())
            .collect();
        for index_str in &index_strings {
            params.push(("songIndexToRemove", index_str));
        }

        let response: SubsonicResponse<serde_json::Value> =
            self.make_request("updatePlaylist", &params).await?;

        if response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "Failed to update playlist: {:?}",
                response.subsonic_response.error
            ));
        }

        Ok(())
    }

    /// Create or update playlist
    pub async fn create_or_update_playlist(
        &self,
        user_id: &str,
        name: &str,
        song_ids: Vec<String>,
    ) -> Result<()> {
        // First, try to find existing playlist with this name
        let playlists = self.get_playlists(None).await?;

        if let Some(existing_playlist) = playlists.iter().find(|p| p.name == name) {
            // Update existing playlist by replacing all songs
            // First remove all existing songs, then add new ones
            let existing_song_count = existing_playlist.song_count.unwrap_or(0);
            let indices_to_remove: Vec<u32> = (0..existing_song_count).collect();

            self.update_playlist(
                &existing_playlist.id,
                None,
                None,
                None,
                &song_ids,
                &indices_to_remove,
            )
            .await?;
        } else {
            // Create new playlist
            self.create_playlist(name, &song_ids).await?;
        }

        Ok(())
    }

    /// Scrobble a song (mark as played)
    pub async fn scrobble(
        &self,
        song_id: &str,
        time: Option<u64>,
        submission: Option<bool>,
    ) -> Result<()> {
        let mut params = vec![("id", song_id)];

        if let Some(t) = time {
            params.push(("time", &t.to_string()));
        }
        if let Some(s) = submission {
            params.push(("submission", if s { "true" } else { "false" }));
        }

        let response: SubsonicResponse<serde_json::Value> =
            self.make_request("scrobble", &params).await?;

        if response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "Failed to scrobble: {:?}",
                response.subsonic_response.error
            ));
        }

        Ok(())
    }

    /// Get current playing information
    pub async fn get_now_playing(&self) -> Result<Vec<NowPlayingEntry>> {
        let response: SubsonicResponse<NowPlayingResponse> =
            self.make_request("getNowPlaying", &[]).await?;

        match response.subsonic_response.data {
            Some(data) => Ok(data.now_playing.entry),
            None => Ok(Vec::new()),
        }
    }

    /// Get recent plays for a user (this is a custom method, may not be available in all Subsonic implementations)
    pub async fn get_recent_plays(
        &self,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<ScrobbleEntry>> {
        // Note: This is a placeholder implementation as standard Subsonic API doesn't have this endpoint
        // You might need to implement this using database queries or custom Navidrome endpoints
        warn!("get_recent_plays is not implemented in standard Subsonic API");
        Ok(Vec::new())
    }

    /// Star a song, album, or artist
    pub async fn star(
        &self,
        song_id: Option<&str>,
        album_id: Option<&str>,
        artist_id: Option<&str>,
    ) -> Result<()> {
        let mut params = Vec::new();

        if let Some(id) = song_id {
            params.push(("id", id));
        }
        if let Some(id) = album_id {
            params.push(("albumId", id));
        }
        if let Some(id) = artist_id {
            params.push(("artistId", id));
        }

        let response: SubsonicResponse<serde_json::Value> =
            self.make_request("star", &params).await?;

        if response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "Failed to star: {:?}",
                response.subsonic_response.error
            ));
        }

        Ok(())
    }

    /// Unstar a song, album, or artist
    pub async fn unstar(
        &self,
        song_id: Option<&str>,
        album_id: Option<&str>,
        artist_id: Option<&str>,
    ) -> Result<()> {
        let mut params = Vec::new();

        if let Some(id) = song_id {
            params.push(("id", id));
        }
        if let Some(id) = album_id {
            params.push(("albumId", id));
        }
        if let Some(id) = artist_id {
            params.push(("artistId", id));
        }

        let response: SubsonicResponse<serde_json::Value> =
            self.make_request("unstar", &params).await?;

        if response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "Failed to unstar: {:?}",
                response.subsonic_response.error
            ));
        }

        Ok(())
    }

    /// Get starred songs, albums, and artists
    pub async fn get_starred(&self) -> Result<SearchResult> {
        let response: SubsonicResponse<SongsResponse> =
            self.make_request("getStarred2", &[]).await?;

        match response
            .subsonic_response
            .data
            .and_then(|d| d.search_result3)
        {
            Some(result) => Ok(result),
            None => Ok(SearchResult {
                song: None,
                album: None,
                artist: None,
            }),
        }
    }

    /// Set rating for a song
    pub async fn set_rating(&self, song_id: &str, rating: u32) -> Result<()> {
        if rating > 5 {
            return Err(anyhow::anyhow!("Rating must be between 0 and 5"));
        }

        let params = [("id", song_id), ("rating", &rating.to_string())];
        let response: SubsonicResponse<serde_json::Value> =
            self.make_request("setRating", &params).await?;

        if response.subsonic_response.status != "ok" {
            return Err(anyhow::anyhow!(
                "Failed to set rating: {:?}",
                response.subsonic_response.error
            ));
        }

        Ok(())
    }

    /// Make a request to the Subsonic API with rate limiting
    async fn make_request<T>(
        &self,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<SubsonicResponse<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Rate limiting
        {
            let mut last_request = self.last_request.lock().await;
            if let Some(last) = *last_request {
                let elapsed = last.elapsed();
                if elapsed < self.rate_limit_delay {
                    sleep(self.rate_limit_delay - elapsed).await;
                }
            }
            *last_request = Some(Instant::now());
        }

        let mut url = Url::parse(&format!("{}rest/{}", self.base_url, method))
            .context("Failed to parse URL")?;

        // Add authentication parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("u", &self.username);
            query_pairs.append_pair("t", &self.token);
            query_pairs.append_pair("s", &self.salt);
            query_pairs.append_pair("v", "1.16.1"); // Subsonic API version
            query_pairs.append_pair("c", "StepheyBot-Music");
            query_pairs.append_pair("f", "json");

            // Add method-specific parameters
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
        }

        debug!("Making Navidrome API request: {}", method);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let response_text = response.text().await.context("Failed to read response")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP error {}: {}", status, response_text));
        }

        let parsed_response: SubsonicResponse<T> = serde_json::from_str(&response_text)
            .with_context(|| format!("Failed to parse JSON response: {}", response_text))?;

        if parsed_response.subsonic_response.status != "ok" {
            if let Some(error) = &parsed_response.subsonic_response.error {
                return Err(anyhow::anyhow!(
                    "Subsonic API error {}: {}",
                    error.code,
                    error.message
                ));
            } else {
                return Err(anyhow::anyhow!("Unknown Subsonic API error"));
            }
        }

        Ok(parsed_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = NavidromeClient::new("http://localhost:4533", "admin", "password").unwrap();
        assert_eq!(client.username, "admin");
        assert!(!client.salt.is_empty());
        assert!(!client.token.is_empty());
    }

    #[test]
    fn test_url_construction() {
        let base_url = "http://localhost:4533";
        let mut url = Url::parse(&format!("{}rest/ping", base_url)).unwrap();

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("u", "admin");
            query_pairs.append_pair("f", "json");
        }

        assert!(url.as_str().contains("u=admin"));
        assert!(url.as_str().contains("f=json"));
    }
}
