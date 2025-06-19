//! Data models for StepheyBot Music
//!
//! This module contains all the data structures used throughout the application,
//! including database entities, API request/response types, and utility types.

pub mod entities;

// Re-export commonly used types
pub use entities::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Generate a new UUID string
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Current UTC timestamp
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

/// Validate email address format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
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

/// Calculate similarity between two strings (simple Levenshtein-like)
pub fn calculate_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 0.95;
    }

    // Simple similarity based on common characters
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert!(Uuid::parse_str(&id).is_ok());
    }

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
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
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
}
