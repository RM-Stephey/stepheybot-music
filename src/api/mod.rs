//! API module for StepheyBot Music
//!
//! This module organizes all HTTP API endpoints for the multi-user music system,
//! including authentication, user management, music library, and social features.

pub mod user_api;

use crate::auth::AuthService;
use crate::services::UserService;

use axum::{http::StatusCode, response::Json, routing::get, Router};
use serde::Serialize;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// API state containing all services
#[derive(Clone)]
pub struct ApiState {
    pub user_service: Arc<UserService>,
    pub auth_service: Arc<AuthService>,
}

/// Create the complete API router with all endpoints
pub fn create_api_router(state: ApiState) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/version", get(version_info))
        // API v1 routes
        .nest("/v1", create_v1_router(state))
        // Middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
}

/// Create API v1 router
fn create_v1_router(state: ApiState) -> Router {
    // Create the user API router with its specific state
    let user_router = user_api::create_user_router(user_api::UserApiState {
        user_service: state.user_service.clone(),
        auth_service: state.auth_service.clone(),
    });

    // Create placeholder routes that don't need state
    let placeholder_router = Router::new()
        .route("/library/stats", get(library_stats))
        .route("/library/scan", get(library_scan))
        .route("/playlists", get(get_playlists))
        .route("/recommendations", get(get_recommendations))
        .route("/integrations/status", get(integration_status));

    // Merge the routers
    Router::new().merge(user_router).merge(placeholder_router)
}

// ============================================================================
// HEALTH AND INFO ENDPOINTS
// ============================================================================

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "StepheyBot Music".to_string(),
    })
}

/// Version information endpoint
async fn version_info() -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        description: env!("CARGO_PKG_DESCRIPTION").to_string(),
        build_date: "unknown".to_string(),
        git_commit: "unknown".to_string(),
        features: get_enabled_features(),
    })
}

// ============================================================================
// PLACEHOLDER ENDPOINTS (TO BE IMPLEMENTED)
// ============================================================================

/// Library statistics endpoint (placeholder)
async fn library_stats() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Library stats endpoint - coming soon",
        "stats": {
            "tracks": 0,
            "artists": 0,
            "albums": 0
        }
    }))
}

/// Library scan endpoint (placeholder)
async fn library_scan() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Library scan endpoint - coming soon"
    }))
}

/// Get playlists endpoint (placeholder)
async fn get_playlists() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Playlists endpoint - coming soon",
        "playlists": []
    }))
}

/// Get recommendations endpoint (placeholder)
async fn get_recommendations() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Recommendations endpoint - coming soon",
        "recommendations": []
    }))
}

/// Integration status endpoint (placeholder)
async fn integration_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Integration status endpoint - coming soon",
        "integrations": {
            "listenbrainz": "disconnected",
            "spotify": "disconnected",
            "navidrome": "unknown"
        }
    }))
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    version: String,
    service: String,
}

#[derive(Debug, Serialize)]
struct VersionResponse {
    version: String,
    name: String,
    description: String,
    build_date: String,
    git_commit: String,
    features: Vec<String>,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get list of enabled compile-time features
fn get_enabled_features() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "auth")]
    features.push("auth".to_string());

    #[cfg(feature = "recommendations")]
    features.push("recommendations".to_string());

    #[cfg(feature = "social")]
    features.push("social".to_string());

    #[cfg(feature = "integrations")]
    features.push("integrations".to_string());

    // Default features
    features.push("user-profiles".to_string());
    features.push("music-library".to_string());
    features.push("playlists".to_string());

    features
}

/// Common API error handler
pub async fn handle_api_error(
    error: Box<dyn std::error::Error + Send + Sync>,
) -> (StatusCode, Json<ApiErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorResponse {
            success: false,
            error: "Internal Server Error".to_string(),
            message: error.to_string(),
            timestamp: chrono::Utc::now(),
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub success: bool,
    pub error: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled_features() {
        let features = get_enabled_features();
        assert!(!features.is_empty());
        assert!(features.contains(&"user-profiles".to_string()));
        assert!(features.contains(&"music-library".to_string()));
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.0.status, "healthy");
        assert_eq!(response.0.service, "StepheyBot Music");
    }

    #[tokio::test]
    async fn test_version_info() {
        let response = version_info().await;
        assert!(!response.0.version.is_empty());
        assert!(!response.0.features.is_empty());
    }
}
