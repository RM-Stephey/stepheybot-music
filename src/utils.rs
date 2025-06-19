//! Utility functions for StepheyBot Music
//!
//! This module contains common helper functions used throughout the application
//! including file operations, string manipulation, audio utilities, and more.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::{debug, warn};
use uuid::Uuid;

/// Generate a new UUID string
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Convert duration in seconds to a human-readable format
pub fn format_duration(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

/// Parse duration string (MM:SS or HH:MM:SS) to seconds
pub fn parse_duration(duration_str: &str) -> Option<u32> {
    let parts: Vec<&str> = duration_str.split(':').collect();

    match parts.len() {
        2 => {
            // MM:SS format
            let minutes: u32 = parts[0].parse().ok()?;
            let seconds: u32 = parts[1].parse().ok()?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            // HH:MM:SS format
            let hours: u32 = parts[0].parse().ok()?;
            let minutes: u32 = parts[1].parse().ok()?;
            let seconds: u32 = parts[2].parse().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}

/// Sanitize string for use in file paths
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitize string for use in URLs
pub fn sanitize_url_path(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            ' ' => '_',
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => c,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Validate email address format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Calculate similarity between two strings using Jaccard similarity
pub fn calculate_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 0.95;
    }

    // Convert to character sets
    let a_chars: std::collections::HashSet<char> = a_lower.chars().collect();
    let b_chars: std::collections::HashSet<char> = b_lower.chars().collect();

    let intersection = a_chars.intersection(&b_chars).count();
    let union = a_chars.union(&b_chars).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Calculate Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

/// Format file size in human-readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if bytes < THRESHOLD {
        return format!("{} B", bytes);
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Get file extension from path
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if file is an audio file based on extension
pub fn is_audio_file(path: &Path) -> bool {
    const AUDIO_EXTENSIONS: &[&str] = &[
        "mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "ape", "opus", "aiff",
    ];

    get_file_extension(path)
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

/// Check if file is an image file based on extension
pub fn is_image_file(path: &Path) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];

    get_file_extension(path)
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

/// Create directory if it doesn't exist
pub async fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        tokio::fs::create_dir_all(path)
            .await
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        debug!("Created directory: {}", path.display());
    }
    Ok(())
}

/// Get file modification time
pub fn get_file_mtime(path: &Path) -> Result<SystemTime> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

    metadata
        .modified()
        .with_context(|| format!("Failed to get modification time for: {}", path.display()))
}

/// Safely truncate string to specified length
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated = s
            .chars()
            .take(max_len.saturating_sub(3))
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

/// Convert snake_case to Title Case
pub fn snake_to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Hash a string using SHA-256
pub fn hash_string(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random alphanumeric string
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Retry mechanism with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut delay = initial_delay_ms;

    for attempt in 0..=max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt == max_retries {
                    return Err(error);
                }

                warn!(
                    "Operation failed (attempt {}/{}): {:?}. Retrying in {}ms",
                    attempt + 1,
                    max_retries + 1,
                    error,
                    delay
                );

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

/// Convert BPM to tempo description
pub fn bpm_to_tempo_description(bpm: f64) -> &'static str {
    match bpm {
        bpm if bpm < 60.0 => "Very Slow",
        bpm if bpm < 80.0 => "Slow",
        bpm if bpm < 100.0 => "Moderate",
        bpm if bpm < 120.0 => "Medium",
        bpm if bpm < 140.0 => "Fast",
        bpm if bpm < 160.0 => "Very Fast",
        _ => "Extremely Fast",
    }
}

/// Audio quality descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQuality {
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub format: Option<String>,
}

impl AudioQuality {
    pub fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(bitrate) = self.bitrate {
            parts.push(format!("{} kbps", bitrate));
        }

        if let Some(sample_rate) = self.sample_rate {
            parts.push(format!("{} Hz", sample_rate));
        }

        if let Some(channels) = self.channels {
            let channel_desc = match channels {
                1 => "Mono".to_string(),
                2 => "Stereo".to_string(),
                n => format!("{} channels", n),
            };
            parts.push(channel_desc);
        }

        if let Some(format) = &self.format {
            parts.push(format.to_uppercase());
        }

        if parts.is_empty() {
            "Unknown".to_string()
        } else {
            parts.join(", ")
        }
    }

    pub fn quality_score(&self) -> f64 {
        let mut score = 0.0;

        // Bitrate score (max 40 points)
        if let Some(bitrate) = self.bitrate {
            score += (bitrate as f64 / 320.0 * 40.0).min(40.0);
        }

        // Sample rate score (max 30 points)
        if let Some(sample_rate) = self.sample_rate {
            score += (sample_rate as f64 / 48000.0 * 30.0).min(30.0);
        }

        // Format score (max 30 points)
        if let Some(format) = &self.format {
            let format_score = match format.to_lowercase().as_str() {
                "flac" | "alac" | "ape" => 30.0,
                "wav" | "aiff" => 25.0,
                "mp3" | "aac" => 20.0,
                "ogg" | "opus" => 18.0,
                "wma" => 15.0,
                _ => 10.0,
            };
            score += format_score;
        }

        score / 100.0 // Normalize to 0-1
    }
}

/// Parse audio format information from filename
pub fn parse_audio_info_from_filename(filename: &str) -> AudioQuality {
    let filename_lower = filename.to_lowercase();

    // Extract format from extension
    let format = get_file_extension(Path::new(filename));

    // Try to extract bitrate from filename (common patterns like "320kbps", "320k", etc.)
    let bitrate = extract_bitrate_from_string(&filename_lower);

    AudioQuality {
        bitrate,
        sample_rate: None, // Would need actual file analysis
        channels: None,    // Would need actual file analysis
        format,
    }
}

/// Extract bitrate from string (common patterns in filenames)
fn extract_bitrate_from_string(s: &str) -> Option<u32> {
    use regex::Regex;

    let re = Regex::new(r"(\d+)k?bps?").ok()?;
    if let Some(captures) = re.captures(s) {
        captures.get(1)?.as_str().parse().ok()
    } else {
        None
    }
}

/// Normalize artist/album/track names for better matching
pub fn normalize_music_name(name: &str) -> String {
    let mut normalized = name.to_lowercase().trim().to_string();

    // Remove common prefixes/suffixes
    let prefixes = ["the ", "a ", "an "];
    for prefix in &prefixes {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
            break;
        }
    }

    // Remove special characters but keep spaces
    normalized = normalized
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect::<String>();

    // Normalize whitespace
    let re = regex::Regex::new(r"\s+").unwrap();
    re.replace_all(&normalized, " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(65), "1:05");
        assert_eq!(format_duration(3665), "1:01:05");
        assert_eq!(format_duration(30), "0:30");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1:05"), Some(65));
        assert_eq!(parse_duration("1:01:05"), Some(3665));
        assert_eq!(parse_duration("0:30"), Some(30));
        assert_eq!(parse_duration("invalid"), None);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Artist / Album"), "Artist _ Album");
        assert_eq!(sanitize_filename("Track: Title"), "Track_ Title");
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("test", "test"), 1.0);
        assert!(calculate_similarity("Test", "test") > 0.9);
        assert!(calculate_similarity("hello", "world") < 0.5);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("song.mp3")));
        assert!(is_audio_file(Path::new("song.flac")));
        assert!(!is_audio_file(Path::new("image.jpg")));
    }

    #[test]
    fn test_normalize_music_name() {
        assert_eq!(normalize_music_name("The Beatles"), "beatles");
        assert_eq!(normalize_music_name("A Song Title!"), "song title");
        assert_eq!(
            normalize_music_name("  Multiple   Spaces  "),
            "multiple spaces"
        );
    }

    #[test]
    fn test_audio_quality_score() {
        let quality = AudioQuality {
            bitrate: Some(320),
            sample_rate: Some(44100),
            channels: Some(2),
            format: Some("mp3".to_string()),
        };

        let score = quality.quality_score();
        assert!(score > 0.0 && score <= 1.0);
    }
}
