//! User API endpoints for StepheyBot Music
//!
//! This module provides HTTP endpoints for user authentication, profile management,
//! and user-related operations in the multi-user music system.

use crate::auth::AuthService;
use crate::models::user::{
    AuthenticatedUser, UpdateUserPreferencesRequest, UpdateUserRequest, UserError, UserSearchQuery,
};
use crate::services::UserService;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{Json as ResponseJson, Redirect},
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, warn};

/// User API state
#[derive(Clone)]
pub struct UserApiState {
    pub user_service: Arc<UserService>,
    pub auth_service: Arc<AuthService>,
}

/// Create user API router
pub fn create_user_router(state: UserApiState) -> Router {
    Router::new()
        // Authentication endpoints
        .route("/auth/login", get(auth_login))
        .route("/auth/callback", get(auth_callback))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/profile", get(auth_profile))
        // User profile endpoints
        .route("/user/profile", get(get_user_profile))
        .route("/user/profile", put(update_user_profile))
        .route("/user/preferences", get(get_user_preferences))
        .route("/user/preferences", put(update_user_preferences))
        .route("/user/dashboard", get(get_user_dashboard))
        .route("/user/stats", get(get_user_stats))
        .route("/user/activity", get(get_user_activity))
        // User management endpoints
        .route("/users/search", get(search_users))
        .route("/users/:username", get(get_public_user_profile))
        .route("/users/:user_id/follow", post(follow_user))
        .route("/users/:user_id/follow", delete(unfollow_user))
        // Admin endpoints
        .route("/admin/users", get(admin_list_users))
        .route("/admin/users/:user_id", get(admin_get_user))
        .route("/admin/users/:user_id", delete(admin_delete_user))
        .route("/admin/users/:user_id/activate", post(admin_activate_user))
        .route(
            "/admin/users/:user_id/deactivate",
            post(admin_deactivate_user),
        )
        .with_state(state)
}

// ============================================================================
// AUTHENTICATION ENDPOINTS
// ============================================================================

/// Redirect to Keycloak login
async fn auth_login() -> Redirect {
    // In a real implementation, this would redirect to Keycloak
    // For now, return a placeholder redirect
    Redirect::temporary("/auth/keycloak-login")
}

/// Handle Keycloak callback
async fn auth_callback(
    Query(params): Query<AuthCallbackParams>,
    State(state): State<UserApiState>,
) -> Result<ResponseJson<AuthResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    debug!("Processing auth callback with code: {}", params.code);

    // In a real implementation, this would exchange the code for tokens
    // For now, return a success response
    Ok(ResponseJson(AuthResponse {
        success: true,
        message: "Authentication successful".to_string(),
        redirect_url: Some("/dashboard".to_string()),
    }))
}

/// Logout user
async fn auth_logout() -> ResponseJson<AuthResponse> {
    // In a real implementation, this would invalidate the session/token
    ResponseJson(AuthResponse {
        success: true,
        message: "Logged out successfully".to_string(),
        redirect_url: Some("/".to_string()),
    })
}

/// Get current authenticated user profile
async fn auth_profile(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<UserProfileResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let profile = state
        .user_service
        .get_user_profile_with_stats(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get user profile: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(UserProfileResponse {
        success: true,
        user: profile,
    }))
}

// ============================================================================
// USER PROFILE ENDPOINTS
// ============================================================================

/// Get user's own profile
async fn get_user_profile(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<UserProfileResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let profile = state
        .user_service
        .get_user_profile_with_stats(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get user profile: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(UserProfileResponse {
        success: true,
        user: profile,
    }))
}

/// Update user profile
async fn update_user_profile(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(update_request): Json<UpdateUserRequest>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    state
        .user_service
        .update_user_profile(user.id, update_request)
        .await
        .map_err(|e| {
            warn!("Failed to update user profile: {}", e);
            match e {
                UserError::Validation(msg) => (
                    StatusCode::BAD_REQUEST,
                    ResponseJson(ErrorResponse::validation_error(msg)),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ErrorResponse::internal_error()),
                ),
            }
        })?;

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "Profile updated successfully".to_string(),
    }))
}

/// Get user preferences
async fn get_user_preferences(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<UserPreferencesResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let user_data = state
        .user_service
        .get_user_by_id(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    let preferences = user_data.preferences().map_err(|e| {
        error!("Failed to parse user preferences: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse::internal_error()),
        )
    })?;

    Ok(ResponseJson(UserPreferencesResponse {
        success: true,
        preferences,
    }))
}

/// Update user preferences
async fn update_user_preferences(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(update_request): Json<UpdateUserPreferencesRequest>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    state
        .user_service
        .update_user_preferences(user.id, update_request)
        .await
        .map_err(|e| {
            warn!("Failed to update user preferences: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "Preferences updated successfully".to_string(),
    }))
}

/// Get user dashboard data
async fn get_user_dashboard(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<UserDashboardResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let dashboard_stats = state
        .user_service
        .get_user_dashboard_stats(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get dashboard stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(UserDashboardResponse {
        success: true,
        dashboard: dashboard_stats,
    }))
}

/// Get user statistics
async fn get_user_stats(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let stats = state
        .user_service
        .get_user_dashboard_stats(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get user stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(json!({
        "success": true,
        "stats": stats,
        "timestamp": chrono::Utc::now()
    })))
}

/// Get user activity
async fn get_user_activity(
    State(_state): State<UserApiState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ActivityParams>,
) -> Result<ResponseJson<serde_json::Value>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // For now, return placeholder activity data
    // In a real implementation, this would fetch actual activity from the database
    let activity = json!({
        "recent_listens": [],
        "recent_favorites": [],
        "recent_playlists": [],
        "listening_streak": 0
    });

    Ok(ResponseJson(json!({
        "success": true,
        "activity": activity,
        "limit": params.limit.unwrap_or(20)
    })))
}

// ============================================================================
// USER DISCOVERY ENDPOINTS
// ============================================================================

/// Search for users
async fn search_users(
    State(state): State<UserApiState>,
    Query(query): Query<UserSearchQuery>,
) -> Result<ResponseJson<UserSearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Optional authentication - anyone can search for public users

    let search_result = state.user_service.search_users(query).await.map_err(|e| {
        error!("Failed to search users: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse::internal_error()),
        )
    })?;

    Ok(ResponseJson(UserSearchResponse {
        success: true,
        result: search_result,
    }))
}

/// Get public user profile by username
async fn get_public_user_profile(
    Path(username): Path<String>,
    State(state): State<UserApiState>,
) -> Result<ResponseJson<UserProfileResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    let user = state
        .user_service
        .get_user_by_username(&username)
        .await
        .map_err(|e| match e {
            UserError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse::not_found(format!("User: {}", username))),
            ),
            _ => {
                error!("Failed to get user by username: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ErrorResponse::internal_error()),
                )
            }
        })?;

    let profile = state
        .user_service
        .get_user_profile_with_stats(user.id)
        .await
        .map_err(|e| {
            error!("Failed to get user profile: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    // Filter sensitive information for public profiles
    // In a real implementation, respect privacy settings
    Ok(ResponseJson(UserProfileResponse {
        success: true,
        user: profile,
    }))
}

/// Follow a user
async fn follow_user(
    Path(user_id): Path<i64>,
    State(_state): State<UserApiState>,
    Extension(current_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // Prevent self-following
    if current_user.id == user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse::validation_error(
                "Cannot follow yourself".to_string(),
            )),
        ));
    }

    // TODO: Implement follow functionality
    // This would involve adding a record to the user_follows table

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "User followed successfully".to_string(),
    }))
}

/// Unfollow a user
async fn unfollow_user(
    Path(_user_id): Path<i64>,
    State(_state): State<UserApiState>,
    Extension(_current_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    // TODO: Implement unfollow functionality
    // This would involve removing a record from the user_follows table

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "User unfollowed successfully".to_string(),
    }))
}

// ============================================================================
// ADMIN ENDPOINTS
// ============================================================================

/// List all users (admin only)
async fn admin_list_users(
    State(state): State<UserApiState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<AdminListParams>,
) -> Result<ResponseJson<UserSearchResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    if !user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(ErrorResponse::forbidden()),
        ));
    }

    let query = UserSearchQuery {
        query: params.search,
        privacy_level: None,
        is_active: params.active,
        has_playlists: None,
        limit: Some(params.limit.unwrap_or(50)),
        offset: Some(params.offset.unwrap_or(0)),
    };

    let search_result = state.user_service.search_users(query).await.map_err(|e| {
        error!("Failed to list users: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(ErrorResponse::internal_error()),
        )
    })?;

    Ok(ResponseJson(UserSearchResponse {
        success: true,
        result: search_result,
    }))
}

/// Get user details (admin only)
async fn admin_get_user(
    Path(user_id): Path<i64>,
    State(state): State<UserApiState>,
    Extension(admin_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<UserProfileResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    if !admin_user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(ErrorResponse::forbidden()),
        ));
    }

    let profile = state
        .user_service
        .get_user_profile_with_stats(user_id)
        .await
        .map_err(|e| match e {
            UserError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse::not_found(format!("User ID: {}", user_id))),
            ),
            _ => {
                error!("Failed to get user profile: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ErrorResponse::internal_error()),
                )
            }
        })?;

    Ok(ResponseJson(UserProfileResponse {
        success: true,
        user: profile,
    }))
}

/// Delete user (admin only)
async fn admin_delete_user(
    Path(user_id): Path<i64>,
    State(state): State<UserApiState>,
    Extension(admin_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    if !admin_user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(ErrorResponse::forbidden()),
        ));
    }

    // Prevent admin from deleting themselves
    if admin_user.id == user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(ErrorResponse::validation_error(
                "Cannot delete your own account".to_string(),
            )),
        ));
    }

    state.user_service.delete_user(user_id).await.map_err(|e| {
        error!("Failed to delete user: {}", e);
        match e {
            UserError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                ResponseJson(ErrorResponse::not_found(format!("User ID: {}", user_id))),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            ),
        }
    })?;

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "User deleted successfully".to_string(),
    }))
}

/// Activate user (admin only)
async fn admin_activate_user(
    Path(user_id): Path<i64>,
    State(state): State<UserApiState>,
    Extension(admin_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    if !admin_user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(ErrorResponse::forbidden()),
        ));
    }

    state
        .user_service
        .reactivate_user(user_id)
        .await
        .map_err(|e| {
            error!("Failed to activate user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "User activated successfully".to_string(),
    }))
}

/// Deactivate user (admin only)
async fn admin_deactivate_user(
    Path(user_id): Path<i64>,
    State(state): State<UserApiState>,
    Extension(admin_user): Extension<AuthenticatedUser>,
) -> Result<ResponseJson<SuccessResponse>, (StatusCode, ResponseJson<ErrorResponse>)> {
    if !admin_user.is_admin() {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(ErrorResponse::forbidden()),
        ));
    }

    state
        .user_service
        .deactivate_user(user_id)
        .await
        .map_err(|e| {
            error!("Failed to deactivate user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ErrorResponse::internal_error()),
            )
        })?;

    Ok(ResponseJson(SuccessResponse {
        success: true,
        message: "User deactivated successfully".to_string(),
    }))
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
struct AuthCallbackParams {
    code: String,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActivityParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AdminListParams {
    search: Option<String>,
    active: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    success: bool,
    message: String,
    redirect_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct UserProfileResponse {
    success: bool,
    user: crate::models::user::UserProfileWithStats,
}

#[derive(Debug, Serialize)]
struct UserPreferencesResponse {
    success: bool,
    preferences: crate::models::user::UserPreferences,
}

#[derive(Debug, Serialize)]
struct UserDashboardResponse {
    success: bool,
    dashboard: crate::models::user::UserDashboardStats,
}

#[derive(Debug, Serialize)]
struct UserSearchResponse {
    success: bool,
    result: crate::models::user::UserSearchResult,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    code: String,
    message: String,
}

impl ErrorResponse {
    fn unauthorized() -> Self {
        Self {
            success: false,
            error: "Unauthorized".to_string(),
            code: "AUTH_REQUIRED".to_string(),
            message: "Authentication required".to_string(),
        }
    }

    fn forbidden() -> Self {
        Self {
            success: false,
            error: "Forbidden".to_string(),
            code: "INSUFFICIENT_PERMISSIONS".to_string(),
            message: "Insufficient permissions".to_string(),
        }
    }

    fn not_found(resource: String) -> Self {
        Self {
            success: false,
            error: "Not Found".to_string(),
            code: "RESOURCE_NOT_FOUND".to_string(),
            message: format!("Resource not found: {}", resource),
        }
    }

    fn validation_error(message: String) -> Self {
        Self {
            success: false,
            error: "Validation Error".to_string(),
            code: "VALIDATION_FAILED".to_string(),
            message,
        }
    }

    fn internal_error() -> Self {
        Self {
            success: false,
            error: "Internal Server Error".to_string(),
            code: "INTERNAL_ERROR".to_string(),
            message: "An internal server error occurred".to_string(),
        }
    }
}
