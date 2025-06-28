//! Transmission API client for StepheyBot Music
//!
//! This module provides integration with Transmission for automated torrent management,
//! download queuing, and file organization.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Transmission API client
#[derive(Clone)]
pub struct TransmissionClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
    session_id: Option<String>,
}

/// Torrent information from Transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "totalSize")]
    pub size: u64,
    #[serde(rename = "percentDone")]
    pub progress: f64,
    #[serde(rename = "rateDownload")]
    pub download_speed: u64,
    #[serde(rename = "rateUpload")]
    pub upload_speed: u64,
    pub status: i32,
    #[serde(rename = "hashString")]
    pub hash: String,
    #[serde(rename = "downloadDir")]
    pub download_dir: String,
    #[serde(rename = "errorString")]
    pub error: String,
    #[serde(rename = "eta")]
    pub eta: i64,
    #[serde(rename = "uploadRatio")]
    pub ratio: f64,
}

impl TorrentInfo {
    /// Get human-readable status
    pub fn status_string(&self) -> &'static str {
        match self.status {
            0 => "stopped",
            1 => "check_pending",
            2 => "checking",
            3 => "download_pending",
            4 => "downloading",
            5 => "seed_pending",
            6 => "seeding",
            _ => "unknown",
        }
    }

    /// Check if torrent is downloading
    pub fn is_downloading(&self) -> bool {
        self.status == 4
    }

    /// Check if torrent is completed
    pub fn is_completed(&self) -> bool {
        self.progress >= 1.0
    }

    /// Check if torrent is seeding
    pub fn is_seeding(&self) -> bool {
        self.status == 6
    }
}

/// Transmission RPC request structure
#[derive(Debug, Serialize)]
struct TransmissionRequest {
    method: String,
    arguments: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<i32>,
}

/// Transmission RPC response structure
#[derive(Debug, Deserialize)]
struct TransmissionResponse {
    result: String,
    arguments: Option<serde_json::Value>,
    tag: Option<i32>,
}

impl TransmissionClient {
    /// Create a new Transmission client
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            password,
            session_id: None,
        }
    }

    /// Make an RPC request to Transmission
    async fn rpc_request(
        &mut self,
        method: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/transmission/rpc", self.base_url);

        let request_body = TransmissionRequest {
            method: method.to_string(),
            arguments,
            tag: None,
        };

        let mut retries = 0;
        loop {
            let mut request_builder = self
                .client
                .post(&url)
                .basic_auth(&self.username, Some(&self.password))
                .json(&request_body);

            // Add session ID if we have one
            if let Some(ref session_id) = self.session_id {
                request_builder = request_builder.header("X-Transmission-Session-Id", session_id);
            }

            let response = request_builder
                .send()
                .await
                .context("Failed to send request to Transmission")?;

            let status = response.status();

            if status == reqwest::StatusCode::CONFLICT {
                // Extract session ID from response headers
                if let Some(session_id) = response.headers().get("X-Transmission-Session-Id") {
                    self.session_id = Some(session_id.to_str()?.to_string());
                    debug!("🔄 Got new Transmission session ID, retrying...");
                    retries += 1;
                    if retries > 2 {
                        return Err(anyhow::anyhow!("Too many session ID retries"));
                    }
                    continue;
                }
            }

            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Transmission request failed: {} - {}",
                    status,
                    error_text
                ));
            }

            let transmission_response: TransmissionResponse = response
                .json()
                .await
                .context("Failed to parse Transmission response")?;

            if transmission_response.result != "success" {
                return Err(anyhow::anyhow!(
                    "Transmission RPC error: {}",
                    transmission_response.result
                ));
            }

            return Ok(transmission_response
                .arguments
                .unwrap_or(serde_json::Value::Null));
        }
    }

    /// Health check - verify Transmission is accessible
    pub async fn health_check(&mut self) -> Result<()> {
        info!(
            "🔍 Starting Transmission health check for: {}",
            self.base_url
        );

        // First test basic connectivity
        info!("📡 Testing basic connectivity to Transmission...");
        let ping_url = format!("{}/transmission/web/", self.base_url);
        let ping_response = match self.client.get(&ping_url).send().await {
            Ok(response) => {
                info!(
                    "✅ Basic connectivity successful - status: {}",
                    response.status()
                );
                response
            }
            Err(e) => {
                warn!("❌ Basic connectivity failed: {}", e);
                return Err(anyhow::anyhow!(
                    "Failed to connect to Transmission - basic connectivity test failed: {}",
                    e
                ));
            }
        };

        info!("✅ Basic connectivity test passed");

        // Test RPC endpoint with session-get
        info!("📋 Testing Transmission RPC endpoint...");
        match self
            .rpc_request("session-get", serde_json::Value::Object(Default::default()))
            .await
        {
            Ok(_) => {
                info!("🎉 Transmission health check successful");
                Ok(())
            }
            Err(e) => {
                warn!("❌ RPC endpoint test failed: {}", e);
                Err(anyhow::anyhow!("Transmission health check failed: {}", e))
            }
        }
    }

    /// Add a torrent by magnet link or torrent URL
    pub async fn add_magnet(
        &mut self,
        magnet_url: &str,
        download_dir: Option<&str>,
        paused: Option<bool>,
    ) -> Result<String> {
        info!(
            "🧲 Adding magnet to Transmission: {}",
            &magnet_url[..50.min(magnet_url.len())]
        );

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "filename".to_string(),
            serde_json::Value::String(magnet_url.to_string()),
        );

        if let Some(dir) = download_dir {
            arguments.insert(
                "download-dir".to_string(),
                serde_json::Value::String(dir.to_string()),
            );
        }

        if let Some(paused) = paused {
            arguments.insert("paused".to_string(), serde_json::Value::Bool(paused));
        }

        let response = self
            .rpc_request("torrent-add", serde_json::Value::Object(arguments))
            .await
            .context("Failed to add torrent to Transmission")?;

        // Extract torrent hash from response
        if let Some(torrent_added) = response.get("torrent-added") {
            if let Some(hash) = torrent_added.get("hashString") {
                let hash_str = hash.as_str().unwrap_or("unknown").to_string();
                info!("✅ Torrent added successfully with hash: {}", hash_str);
                return Ok(hash_str);
            }
        }

        // Check if torrent already exists
        if let Some(torrent_duplicate) = response.get("torrent-duplicate") {
            if let Some(hash) = torrent_duplicate.get("hashString") {
                let hash_str = hash.as_str().unwrap_or("unknown").to_string();
                info!("⚠️ Torrent already exists with hash: {}", hash_str);
                return Ok(hash_str);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to extract torrent hash from response"
        ))
    }

    /// Get information about all torrents
    pub async fn get_torrents(&mut self) -> Result<Vec<TorrentInfo>> {
        let fields = vec![
            "id",
            "name",
            "totalSize",
            "percentDone",
            "rateDownload",
            "rateUpload",
            "status",
            "hashString",
            "downloadDir",
            "errorString",
            "eta",
            "uploadRatio",
        ];

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "fields".to_string(),
            serde_json::Value::Array(
                fields
                    .into_iter()
                    .map(|f| serde_json::Value::String(f.to_string()))
                    .collect(),
            ),
        );

        let response = self
            .rpc_request("torrent-get", serde_json::Value::Object(arguments))
            .await
            .context("Failed to get torrents from Transmission")?;

        if let Some(torrents) = response.get("torrents") {
            let torrents: Vec<TorrentInfo> =
                serde_json::from_value(torrents.clone()).context("Failed to parse torrent list")?;
            debug!("📊 Retrieved {} torrents from Transmission", torrents.len());
            Ok(torrents)
        } else {
            Ok(vec![])
        }
    }

    /// Get information about a specific torrent by hash
    pub async fn get_torrent_by_hash(&mut self, hash: &str) -> Result<Option<TorrentInfo>> {
        let torrents = self.get_torrents().await?;
        Ok(torrents.into_iter().find(|t| t.hash == hash))
    }

    /// Pause a torrent
    pub async fn pause_torrent(&mut self, hash: &str) -> Result<()> {
        let torrent_info = self
            .get_torrent_by_hash(hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Torrent not found: {}", hash))?;

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "ids".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Number(torrent_info.id.into())]),
        );

        self.rpc_request("torrent-stop", serde_json::Value::Object(arguments))
            .await
            .context("Failed to pause torrent")?;

        info!("⏸️ Paused torrent: {}", hash);
        Ok(())
    }

    /// Resume a torrent
    pub async fn resume_torrent(&mut self, hash: &str) -> Result<()> {
        let torrent_info = self
            .get_torrent_by_hash(hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Torrent not found: {}", hash))?;

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "ids".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Number(torrent_info.id.into())]),
        );

        self.rpc_request("torrent-start", serde_json::Value::Object(arguments))
            .await
            .context("Failed to resume torrent")?;

        info!("▶️ Resumed torrent: {}", hash);
        Ok(())
    }

    /// Remove a torrent
    pub async fn remove_torrent(&mut self, hash: &str, delete_files: bool) -> Result<()> {
        let torrent_info = self
            .get_torrent_by_hash(hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Torrent not found: {}", hash))?;

        let mut arguments = serde_json::Map::new();
        arguments.insert(
            "ids".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Number(torrent_info.id.into())]),
        );
        arguments.insert(
            "delete-local-data".to_string(),
            serde_json::Value::Bool(delete_files),
        );

        self.rpc_request("torrent-remove", serde_json::Value::Object(arguments))
            .await
            .context("Failed to remove torrent")?;

        info!(
            "🗑️ Removed torrent: {} (delete files: {})",
            hash, delete_files
        );
        Ok(())
    }

    /// Get session statistics
    pub async fn get_session_stats(&mut self) -> Result<serde_json::Value> {
        self.rpc_request(
            "session-stats",
            serde_json::Value::Object(Default::default()),
        )
        .await
        .context("Failed to get session stats")
    }

    /// Get the current download directory
    pub async fn get_download_dir(&mut self) -> Result<String> {
        let response = self
            .rpc_request("session-get", serde_json::Value::Object(Default::default()))
            .await?;

        if let Some(download_dir) = response.get("download-dir") {
            if let Some(dir_str) = download_dir.as_str() {
                return Ok(dir_str.to_string());
            }
        }

        Err(anyhow::anyhow!("Failed to get download directory"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torrent_status_string() {
        let mut torrent = TorrentInfo {
            id: 1,
            name: "test".to_string(),
            size: 1000,
            progress: 0.5,
            download_speed: 100,
            upload_speed: 10,
            status: 4, // downloading
            hash: "abc123".to_string(),
            download_dir: "/downloads".to_string(),
            error: "".to_string(),
            eta: 3600,
            ratio: 0.5,
        };

        assert_eq!(torrent.status_string(), "downloading");
        assert!(torrent.is_downloading());
        assert!(!torrent.is_completed());

        torrent.status = 6; // seeding
        torrent.progress = 1.0;
        assert_eq!(torrent.status_string(), "seeding");
        assert!(!torrent.is_downloading());
        assert!(torrent.is_completed());
        assert!(torrent.is_seeding());
    }
}
