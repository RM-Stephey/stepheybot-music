//! MusicBrainz API client for StepheyBot Music
//!
//! This module provides integration with MusicBrainz for music metadata enrichment,
//! artist information, release data, and cover art fetching.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tracing::{debug, info, warn};

/// MusicBrainz API client with rate limiting
#[derive(Clone)]
pub struct MusicBrainzClient {
    client: Client,
    base_url: String,
    cover_art_url: String,
    user_agent: String,
    last_request: Arc<Mutex<Option<Instant>>>,
    rate_limit_delay: Duration,
}

/// MusicBrainz artist entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub disambiguation: Option<String>,
    pub country: Option<String>,
    pub area: Option<Area>,
    pub begin_area: Option<Area>,
    pub end_area: Option<Area>,
    pub life_span: Option<LifeSpan>,
    pub aliases: Option<Vec<Alias>>,
    pub genres: Option<Vec<Genre>>,
    pub tags: Option<Vec<Tag>>,
    pub relations: Option<Vec<Relation>>,
    pub artist_type: Option<String>,
    pub gender: Option<String>,
}

/// MusicBrainz release entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub title: String,
    pub disambiguation: Option<String>,
    pub date: Option<String>,
    pub country: Option<String>,
    pub status: Option<String>,
    pub packaging: Option<String>,
    pub barcode: Option<String>,
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub release_group: Option<ReleaseGroup>,
    pub media: Option<Vec<Medium>>,
    pub label_info: Option<Vec<LabelInfo>>,
    pub cover_art_archive: Option<CoverArtArchive>,
    pub relations: Option<Vec<Relation>>,
    pub tags: Option<Vec<Tag>>,
    pub genres: Option<Vec<Genre>>,
}

/// MusicBrainz recording entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub disambiguation: Option<String>,
    pub length: Option<u32>, // Duration in milliseconds
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub releases: Option<Vec<Release>>,
    pub isrcs: Option<Vec<String>>,
    pub tags: Option<Vec<Tag>>,
    pub genres: Option<Vec<Genre>>,
    pub relations: Option<Vec<Relation>>,
    pub rating: Option<Rating>,
}

/// MusicBrainz release group entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGroup {
    pub id: String,
    pub title: String,
    pub disambiguation: Option<String>,
    pub primary_type: Option<String>,
    pub secondary_types: Option<Vec<String>>,
    pub first_release_date: Option<String>,
    pub artist_credit: Option<Vec<ArtistCredit>>,
    pub tags: Option<Vec<Tag>>,
    pub genres: Option<Vec<Genre>>,
}

/// Artist credit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistCredit {
    pub name: String,
    pub artist: Option<Artist>,
    pub joinphrase: Option<String>,
}

/// Geographic area information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub area_type: Option<String>,
    pub iso_3166_1_codes: Option<Vec<String>>,
    pub iso_3166_2_codes: Option<Vec<String>>,
    pub iso_3166_3_codes: Option<Vec<String>>,
}

/// Life span information (birth/death dates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeSpan {
    pub begin: Option<String>,
    pub end: Option<String>,
    pub ended: Option<bool>,
}

/// Alias information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub name: String,
    pub sort_name: Option<String>,
    pub alias_type: Option<String>,
    pub locale: Option<String>,
    pub primary: Option<bool>,
}

/// Genre information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub count: Option<u32>,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub count: Option<u32>,
}

/// Relation information (links between entities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub relation_type: String,
    pub direction: Option<String>,
    pub target: Option<String>,
    pub artist: Option<Artist>,
    pub release: Option<Release>,
    pub recording: Option<Recording>,
    pub url: Option<Url>,
    pub begin: Option<String>,
    pub end: Option<String>,
    pub ended: Option<bool>,
    pub attributes: Option<Vec<String>>,
}

/// URL information in relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    pub id: String,
    pub resource: String,
}

/// Rating information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub value: Option<f64>,
    pub votes_count: Option<u32>,
}

/// Medium information (CD, vinyl, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medium {
    pub title: Option<String>,
    pub format: Option<String>,
    pub position: Option<u32>,
    pub track_count: Option<u32>,
    pub tracks: Option<Vec<Track>>,
}

/// Track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub position: Option<u32>,
    pub length: Option<u32>,
    pub number: Option<String>,
    pub recording: Option<Recording>,
    pub artist_credit: Option<Vec<ArtistCredit>>,
}

/// Label information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelInfo {
    pub catalog_number: Option<String>,
    pub label: Option<Label>,
}

/// Label entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub sort_name: Option<String>,
    pub disambiguation: Option<String>,
    pub label_type: Option<String>,
    pub country: Option<String>,
    pub area: Option<Area>,
    pub life_span: Option<LifeSpan>,
}

/// Cover Art Archive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtArchive {
    pub artwork: Option<bool>,
    pub count: Option<u32>,
    pub front: Option<bool>,
    pub back: Option<bool>,
}

/// Cover art image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtImage {
    pub id: String,
    pub image: String,
    pub thumbnails: Option<CoverArtThumbnails>,
    pub front: Option<bool>,
    pub back: Option<bool>,
    pub types: Option<Vec<String>>,
    pub comment: Option<String>,
    pub approved: Option<bool>,
    pub edit: Option<u32>,
}

/// Cover art thumbnails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverArtThumbnails {
    pub small: Option<String>,
    pub large: Option<String>,
    #[serde(rename = "250")]
    pub thumb_250: Option<String>,
    #[serde(rename = "500")]
    pub thumb_500: Option<String>,
    #[serde(rename = "1200")]
    pub thumb_1200: Option<String>,
}

/// Search result container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    pub created: Option<String>,
    pub count: Option<u32>,
    pub offset: Option<u32>,
    pub artists: Option<Vec<T>>,
    pub releases: Option<Vec<T>>,
    pub recordings: Option<Vec<T>>,
    #[serde(rename = "release-groups")]
    pub release_groups: Option<Vec<T>>,
}

impl MusicBrainzClient {
    /// Create a new MusicBrainz client
    pub fn new(user_agent: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(user_agent)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: "https://musicbrainz.org/ws/2".to_string(),
            cover_art_url: "https://coverartarchive.org".to_string(),
            user_agent: user_agent.to_string(),
            last_request: Arc::new(Mutex::new(None)),
            rate_limit_delay: Duration::from_millis(1000), // 1 request per second
        })
    }

    /// Respect MusicBrainz rate limits
    async fn rate_limit(&self) {
        let mut last_request = self.last_request.lock().await;

        if let Some(last) = *last_request {
            let elapsed = last.elapsed();
            if elapsed < self.rate_limit_delay {
                let wait_time = self.rate_limit_delay - elapsed;
                debug!("Rate limiting: waiting {:?}", wait_time);
                sleep(wait_time).await;
            }
        }

        *last_request = Some(Instant::now());
    }

    /// Perform health check on MusicBrainz
    pub async fn health_check(&self) -> Result<()> {
        self.rate_limit().await;

        let url = format!(
            "{}/artist/5b11f4ce-a62d-471e-81fc-a69a8278c7da",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .query(&[("fmt", "json")])
            .send()
            .await
            .context("Failed to connect to MusicBrainz")?;

        if response.status().is_success() {
            debug!("MusicBrainz health check passed");
            Ok(())
        } else {
            anyhow::bail!("MusicBrainz health check failed: {}", response.status());
        }
    }

    /// Get artist by MusicBrainz ID
    pub async fn get_artist(
        &self,
        mbid: &str,
        includes: Option<&[&str]>,
    ) -> Result<Option<Artist>> {
        self.rate_limit().await;

        let url = format!("{}/artist/{}", self.base_url, mbid);
        let mut params = vec![("fmt", "json")];

        if let Some(inc) = includes {
            params.push(("inc", &inc.join("+")));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get artist")?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let artist: Artist = response
                    .json()
                    .await
                    .context("Failed to parse artist response")?;
                Ok(Some(artist))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => anyhow::bail!("Failed to get artist: {}", response.status()),
        }
    }

    /// Get release by MusicBrainz ID
    pub async fn get_release(
        &self,
        mbid: &str,
        includes: Option<&[&str]>,
    ) -> Result<Option<Release>> {
        self.rate_limit().await;

        let url = format!("{}/release/{}", self.base_url, mbid);
        let mut params = vec![("fmt", "json")];

        if let Some(inc) = includes {
            params.push(("inc", &inc.join("+")));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get release")?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let release: Release = response
                    .json()
                    .await
                    .context("Failed to parse release response")?;
                Ok(Some(release))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => anyhow::bail!("Failed to get release: {}", response.status()),
        }
    }

    /// Get recording by MusicBrainz ID
    pub async fn get_recording(
        &self,
        mbid: &str,
        includes: Option<&[&str]>,
    ) -> Result<Option<Recording>> {
        self.rate_limit().await;

        let url = format!("{}/recording/{}", self.base_url, mbid);
        let mut params = vec![("fmt", "json")];

        if let Some(inc) = includes {
            params.push(("inc", &inc.join("+")));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to get recording")?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let recording: Recording = response
                    .json()
                    .await
                    .context("Failed to parse recording response")?;
                Ok(Some(recording))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => anyhow::bail!("Failed to get recording: {}", response.status()),
        }
    }

    /// Search for artists
    pub async fn search_artists(&self, query: &str, limit: Option<u32>) -> Result<Vec<Artist>> {
        self.rate_limit().await;

        let url = format!("{}/artist", self.base_url);
        let mut params = vec![("query", query), ("fmt", "json")];

        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to search artists")?;

        if response.status().is_success() {
            let result: SearchResult<Artist> = response
                .json()
                .await
                .context("Failed to parse search response")?;

            Ok(result.artists.unwrap_or_default())
        } else {
            anyhow::bail!("Failed to search artists: {}", response.status());
        }
    }

    /// Search for releases
    pub async fn search_releases(&self, query: &str, limit: Option<u32>) -> Result<Vec<Release>> {
        self.rate_limit().await;

        let url = format!("{}/release", self.base_url);
        let mut params = vec![("query", query), ("fmt", "json")];

        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to search releases")?;

        if response.status().is_success() {
            let result: SearchResult<Release> = response
                .json()
                .await
                .context("Failed to parse search response")?;

            Ok(result.releases.unwrap_or_default())
        } else {
            anyhow::bail!("Failed to search releases: {}", response.status());
        }
    }

    /// Search for recordings
    pub async fn search_recordings(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Recording>> {
        self.rate_limit().await;

        let url = format!("{}/recording", self.base_url);
        let mut params = vec![("query", query), ("fmt", "json")];

        if let Some(limit) = limit {
            params.push(("limit", &limit.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to search recordings")?;

        if response.status().is_success() {
            let result: SearchResult<Recording> = response
                .json()
                .await
                .context("Failed to parse search response")?;

            Ok(result.recordings.unwrap_or_default())
        } else {
            anyhow::bail!("Failed to search recordings: {}", response.status());
        }
    }

    /// Get cover art for a release
    pub async fn get_cover_art(&self, release_mbid: &str) -> Result<Vec<CoverArtImage>> {
        self.rate_limit().await;

        let url = format!("{}/release/{}", self.cover_art_url, release_mbid);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get cover art")?;

        match response.status() {
            reqwest::StatusCode::OK => {
                #[derive(Deserialize)]
                struct CoverArtResponse {
                    images: Vec<CoverArtImage>,
                }

                let result: CoverArtResponse = response
                    .json()
                    .await
                    .context("Failed to parse cover art response")?;

                Ok(result.images)
            }
            reqwest::StatusCode::NOT_FOUND => {
                debug!("No cover art found for release: {}", release_mbid);
                Ok(Vec::new())
            }
            _ => {
                warn!(
                    "Failed to get cover art for {}: {}",
                    release_mbid,
                    response.status()
                );
                Ok(Vec::new()) // Return empty instead of error for graceful degradation
            }
        }
    }

    /// Get front cover art URL for a release
    pub async fn get_front_cover_art_url(&self, release_mbid: &str) -> Result<Option<String>> {
        let images = self.get_cover_art(release_mbid).await?;

        // Find the front cover
        for image in images {
            if image.front == Some(true) {
                return Ok(Some(image.image));
            }
        }

        // If no front cover, return the first image
        Ok(images.first().map(|img| img.image.clone()))
    }

    /// Enhanced search for artist with fuzzy matching
    pub async fn search_artist_fuzzy(&self, name: &str) -> Result<Option<Artist>> {
        let query = format!("artist:{}", name);
        let artists = self.search_artists(&query, Some(10)).await?;

        if artists.is_empty() {
            return Ok(None);
        }

        // Simple scoring based on name similarity
        let mut best_match = None;
        let mut best_score = 0.0;

        let normalized_query = crate::utils::normalize_music_name(name);

        for artist in artists {
            let normalized_artist = crate::utils::normalize_music_name(&artist.name);
            let score = crate::utils::calculate_similarity(&normalized_query, &normalized_artist);

            if score > best_score {
                best_score = score;
                best_match = Some(artist);
            }
        }

        // Return match if similarity is reasonable
        if best_score > 0.7 {
            Ok(best_match)
        } else {
            Ok(None)
        }
    }

    /// Enhanced search for release with fuzzy matching
    pub async fn search_release_fuzzy(&self, artist: &str, album: &str) -> Result<Option<Release>> {
        let query = format!("artist:{} AND release:{}", artist, album);
        let releases = self.search_releases(&query, Some(10)).await?;

        if releases.is_empty() {
            return Ok(None);
        }

        // Score releases based on artist and album name similarity
        let mut best_match = None;
        let mut best_score = 0.0;

        let normalized_artist = crate::utils::normalize_music_name(artist);
        let normalized_album = crate::utils::normalize_music_name(album);

        for release in releases {
            let mut score = 0.0;

            // Score based on album title
            let normalized_release_title = crate::utils::normalize_music_name(&release.title);
            score +=
                crate::utils::calculate_similarity(&normalized_album, &normalized_release_title)
                    * 0.6;

            // Score based on artist
            if let Some(artist_credits) = &release.artist_credit {
                for credit in artist_credits {
                    let normalized_credit_name = crate::utils::normalize_music_name(&credit.name);
                    let artist_score = crate::utils::calculate_similarity(
                        &normalized_artist,
                        &normalized_credit_name,
                    );
                    score += artist_score * 0.4;
                    break; // Use first credit for scoring
                }
            }

            if score > best_score {
                best_score = score;
                best_match = Some(release);
            }
        }

        // Return match if similarity is reasonable
        if best_score > 0.6 {
            Ok(best_match)
        } else {
            Ok(None)
        }
    }

    /// Get artist relations (similar artists, band members, etc.)
    pub async fn get_artist_relations(&self, mbid: &str) -> Result<Vec<Relation>> {
        let artist = self
            .get_artist(mbid, Some(&["artist-rels", "url-rels"]))
            .await?;

        Ok(artist.and_then(|a| a.relations).unwrap_or_default())
    }

    /// Extract genres from an artist
    pub fn extract_artist_genres(&self, artist: &Artist) -> Vec<String> {
        let mut genres = Vec::new();

        if let Some(artist_genres) = &artist.genres {
            for genre in artist_genres {
                genres.push(genre.name.clone());
            }
        }

        if let Some(tags) = &artist.tags {
            for tag in tags {
                // Filter tags that look like genres
                if tag.count.unwrap_or(0) > 5 {
                    genres.push(tag.name.clone());
                }
            }
        }

        genres
    }

    /// Parse release date to structured format
    pub fn parse_release_date(&self, date_str: &str) -> Option<NaiveDate> {
        // MusicBrainz dates can be "YYYY", "YYYY-MM", or "YYYY-MM-DD"
        if date_str.len() == 4 {
            // Year only
            if let Ok(year) = date_str.parse::<i32>() {
                return NaiveDate::from_ymd_opt(year, 1, 1);
            }
        } else if date_str.len() == 7 {
            // Year-Month
            let parts: Vec<&str> = date_str.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                    return NaiveDate::from_ymd_opt(year, month, 1);
                }
            }
        } else if date_str.len() == 10 {
            // Full date
            return NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MusicBrainzClient::new("StepheyBot-Music/1.0").unwrap();
        assert_eq!(client.base_url, "https://musicbrainz.org/ws/2");
        assert_eq!(client.user_agent, "StepheyBot-Music/1.0");
    }

    #[test]
    fn test_parse_release_date() {
        let client = MusicBrainzClient::new("Test/1.0").unwrap();

        assert_eq!(
            client.parse_release_date("2023"),
            NaiveDate::from_ymd_opt(2023, 1, 1)
        );
        assert_eq!(
            client.parse_release_date("2023-05"),
            NaiveDate::from_ymd_opt(2023, 5, 1)
        );
        assert_eq!(
            client.parse_release_date("2023-05-15"),
            NaiveDate::from_ymd_opt(2023, 5, 15)
        );
        assert_eq!(client.parse_release_date("invalid"), None);
    }

    #[test]
    fn test_extract_artist_genres() {
        let client = MusicBrainzClient::new("Test/1.0").unwrap();

        let artist = Artist {
            id: "test".to_string(),
            name: "Test Artist".to_string(),
            sort_name: None,
            disambiguation: None,
            country: None,
            area: None,
            begin_area: None,
            end_area: None,
            life_span: None,
            aliases: None,
            genres: Some(vec![
                Genre {
                    id: "1".to_string(),
                    name: "Rock".to_string(),
                    count: Some(10),
                },
                Genre {
                    id: "2".to_string(),
                    name: "Pop".to_string(),
                    count: Some(5),
                },
            ]),
            tags: Some(vec![Tag {
                name: "alternative rock".to_string(),
                count: Some(8),
            }]),
            relations: None,
            artist_type: None,
            gender: None,
        };

        let genres = client.extract_artist_genres(&artist);
        assert!(genres.contains(&"Rock".to_string()));
        assert!(genres.contains(&"Pop".to_string()));
        assert!(genres.contains(&"alternative rock".to_string()));
    }
}
