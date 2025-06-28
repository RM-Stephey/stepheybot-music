//! StepheyBot Music Library
//!
//! A private Spotify-like music streaming service with recommendations,
//! multi-user support, and various integrations.

pub mod api;
pub mod auth;
pub mod clients;
pub mod database;
pub mod models;
pub mod services;
pub mod utils;

// Re-export commonly used types for convenience
pub use api::{create_api_router, ApiState};
pub use auth::AuthService;
pub use database::Database;
pub use services::UserService;
