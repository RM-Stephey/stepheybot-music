//! Authentication middleware and utilities for StepheyBot Music
//!
//! This module provides JWT token validation, user authentication, and authorization
//! middleware for integration with Keycloak SSO and multi-user support.

use crate::models::user::{
    AuthenticatedUser, CreateUserRequest, User, UserError, UserResult, UserRole,
};
use crate::services::user_service::UserService;

use anyhow::{Context, Result};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

/// JWT Claims structure from Keycloak
#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakClaims {
    pub sub: String,                             // Keycloak user ID
    pub preferred_username: String,              // Username
    pub email: String,                           // Email address
    pub name: Option<String>,                    // Full name
    pub given_name: Option<String>,              // First name
    pub family_name: Option<String>,             // Last name
    pub realm_access: RealmAccess,               // Realm roles
    pub resource_access: Option<ResourceAccess>, // Client roles
    pub exp: usize,                              // Expiration time
    pub iat: usize,                              // Issued at time
    pub iss: String,                             // Issuer
    pub aud: Vec<String>,                        // Audience
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    #[serde(rename = "stepheybot-music")]
    pub stepheybot_music: Option<ClientAccess>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientAccess {
    pub roles: Vec<String>,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub keycloak_realm_url: String,
    pub keycloak_client_id: String,
    pub jwt_secret: String,
    pub jwt_algorithm: Algorithm,
    pub token_validation: Validation,
}

impl AuthConfig {
    /// Create new authentication configuration
    pub fn new(keycloak_realm_url: String, keycloak_client_id: String, jwt_secret: String) -> Self {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&keycloak_client_id]);
        validation.set_issuer(&[&keycloak_realm_url]);

        Self {
            keycloak_realm_url,
            keycloak_client_id,
            jwt_secret,
            jwt_algorithm: Algorithm::RS256,
            token_validation: validation,
        }
    }

    /// Create configuration for development/testing with HS256
    pub fn development(jwt_secret: String) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["stepheybot-music"]);
        validation.leeway = 60; // Allow 60 seconds leeway for time sync

        Self {
            keycloak_realm_url: "http://localhost:8080/realms/stepheybot".to_string(),
            keycloak_client_id: "stepheybot-music".to_string(),
            jwt_secret,
            jwt_algorithm: Algorithm::HS256,
            token_validation: validation,
        }
    }
}

/// Authentication service for handling JWT tokens and user management
#[derive(Clone)]
pub struct AuthService {
    config: AuthConfig,
    decoding_key: DecodingKey,
    user_service: Arc<UserService>,
}

impl AuthService {
    /// Create new authentication service
    pub fn new(config: AuthConfig, user_service: Arc<UserService>) -> Result<Self> {
        let decoding_key = match config.jwt_algorithm {
            Algorithm::HS256 => DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            Algorithm::RS256 => {
                // In production, this would load the public key from Keycloak
                // For now, we'll use the secret as a placeholder
                DecodingKey::from_secret(config.jwt_secret.as_bytes())
            }
            _ => return Err(anyhow::anyhow!("Unsupported JWT algorithm")),
        };

        Ok(Self {
            config,
            decoding_key,
            user_service,
        })
    }

    /// Validate JWT token and extract user information
    pub async fn validate_token(&self, token: &str) -> UserResult<AuthenticatedUser> {
        debug!("Validating JWT token");

        // Decode and validate the JWT token
        let token_data =
            decode::<KeycloakClaims>(token, &self.decoding_key, &self.config.token_validation)
                .map_err(|e| {
                    warn!("JWT validation failed: {}", e);
                    UserError::Authentication(format!("Invalid token: {}", e))
                })?;

        let claims = token_data.claims;
        debug!(
            "JWT token validated for user: {}",
            claims.preferred_username
        );

        // Extract roles from token
        let mut roles = claims.realm_access.roles.clone();
        if let Some(ref resource_access) = claims.resource_access {
            if let Some(client_access) = &resource_access.stepheybot_music {
                roles.extend(client_access.roles.clone());
            }
        }

        // Get or create user in database
        let user = self
            .user_service
            .get_or_create_user_from_keycloak(&claims)
            .await?;

        Ok(AuthenticatedUser {
            id: user.id,
            keycloak_id: claims.sub,
            username: claims.preferred_username,
            email: claims.email,
            display_name: claims.name.or(user.display_name),
            roles,
            is_active: user.is_active,
        })
    }

    /// Extract bearer token from authorization header
    fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
        headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_string())
                } else {
                    None
                }
            })
    }

    /// Create a new user from Keycloak claims
    async fn create_user_from_keycloak(&self, claims: &KeycloakClaims) -> UserResult<User> {
        let create_request = CreateUserRequest {
            keycloak_id: claims.sub.clone(),
            username: claims.preferred_username.clone(),
            email: claims.email.clone(),
            display_name: claims.name.clone(),
        };

        self.user_service.create_user(create_request).await
    }
}

/// Authentication middleware for protecting routes
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let token =
        AuthService::extract_bearer_token(request.headers()).ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token and get user
    let user = auth_service.validate_token(&token).await.map_err(|e| {
        warn!("Authentication failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Check if user is active
    if !user.is_active {
        warn!("Inactive user attempted access: {}", user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    // Store user in request extensions for downstream handlers
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Authorization middleware for role-based access control
pub async fn require_admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get authenticated user from request extensions
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user is admin
    if !user.is_admin() {
        warn!("Admin access denied for user: {}", user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// User role middleware for regular users
pub async fn require_user_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get authenticated user from request extensions
    let user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if user has user role or is admin
    if !user.has_role("user") && !user.is_admin() {
        warn!("User access denied for user: {}", user.username);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Optional authentication middleware (allows both authenticated and anonymous access)
pub async fn optional_auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and validate token
    if let Some(token) = AuthService::extract_bearer_token(request.headers()) {
        if let Ok(user) = auth_service.validate_token(&token).await {
            if user.is_active {
                request.extensions_mut().insert(user);
            }
        }
    }

    next.run(request).await
}

/// Extract authenticated user from request
pub fn get_authenticated_user(request: &Request) -> Option<&AuthenticatedUser> {
    request.extensions().get::<AuthenticatedUser>()
}

/// Require authenticated user from request (returns 401 if not found)
pub fn require_authenticated_user(request: &Request) -> Result<&AuthenticatedUser, StatusCode> {
    get_authenticated_user(request).ok_or(StatusCode::UNAUTHORIZED)
}

/// Check if current user can access another user's data
pub fn can_access_user_data(current_user: &AuthenticatedUser, target_user_id: i64) -> bool {
    // Admin can access any user's data
    if current_user.is_admin() {
        return true;
    }

    // Users can access their own data
    current_user.id == target_user_id
}

/// Authentication error responses
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
}

impl AuthErrorResponse {
    pub fn unauthorized() -> Self {
        Self {
            error: "Unauthorized".to_string(),
            message: "Authentication required".to_string(),
            code: "AUTH_REQUIRED".to_string(),
        }
    }

    pub fn forbidden() -> Self {
        Self {
            error: "Forbidden".to_string(),
            message: "Insufficient permissions".to_string(),
            code: "INSUFFICIENT_PERMISSIONS".to_string(),
        }
    }

    pub fn invalid_token() -> Self {
        Self {
            error: "Invalid Token".to_string(),
            message: "The provided authentication token is invalid".to_string(),
            code: "INVALID_TOKEN".to_string(),
        }
    }

    pub fn inactive_user() -> Self {
        Self {
            error: "Account Inactive".to_string(),
            message: "Your account has been deactivated".to_string(),
            code: "ACCOUNT_INACTIVE".to_string(),
        }
    }
}

/// Utility functions for authentication
pub mod utils {
    use super::*;
    use axum::response::Json;

    /// Create unauthorized response
    pub fn unauthorized_response() -> (StatusCode, Json<AuthErrorResponse>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse::unauthorized()),
        )
    }

    /// Create forbidden response
    pub fn forbidden_response() -> (StatusCode, Json<AuthErrorResponse>) {
        (StatusCode::FORBIDDEN, Json(AuthErrorResponse::forbidden()))
    }

    /// Create invalid token response
    pub fn invalid_token_response() -> (StatusCode, Json<AuthErrorResponse>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse::invalid_token()),
        )
    }

    /// Create inactive user response
    pub fn inactive_user_response() -> (StatusCode, Json<AuthErrorResponse>) {
        (
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse::inactive_user()),
        )
    }

    /// Generate a simple JWT token for testing
    pub fn generate_test_token(
        user_id: &str,
        username: &str,
        email: &str,
        roles: Vec<String>,
        secret: &str,
    ) -> Result<String> {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let exp = now + 3600; // 1 hour from now

        let claims = KeycloakClaims {
            sub: user_id.to_string(),
            preferred_username: username.to_string(),
            email: email.to_string(),
            name: Some(username.to_string()),
            given_name: Some(username.to_string()),
            family_name: None,
            realm_access: RealmAccess { roles },
            resource_access: None,
            exp,
            iat: now,
            iss: "http://localhost:8080/realms/stepheybot".to_string(),
            aud: vec!["stepheybot-music".to_string()],
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());

        encode(&header, &claims, &encoding_key).context("Failed to generate test token")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_creation() {
        let config = AuthConfig::new(
            "http://localhost:8080/realms/test".to_string(),
            "test-client".to_string(),
            "secret".to_string(),
        );

        assert_eq!(config.keycloak_client_id, "test-client");
        assert_eq!(config.jwt_algorithm, Algorithm::RS256);
    }

    #[test]
    fn test_development_config() {
        let config = AuthConfig::development("dev-secret".to_string());
        assert_eq!(config.jwt_algorithm, Algorithm::HS256);
        assert_eq!(config.keycloak_client_id, "stepheybot-music");
    }

    #[test]
    fn test_bearer_token_extraction() {
        use axum::http::HeaderValue;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer test-token-123"),
        );

        let token = AuthService::extract_bearer_token(&headers);
        assert_eq!(token, Some("test-token-123".to_string()));

        // Test invalid format
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Invalid token"));
        let token = AuthService::extract_bearer_token(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_can_access_user_data() {
        let admin_user = AuthenticatedUser {
            id: 1,
            keycloak_id: "admin".to_string(),
            username: "admin".to_string(),
            email: "admin@test.com".to_string(),
            display_name: None,
            roles: vec!["admin".to_string()],
            is_active: true,
        };

        let regular_user = AuthenticatedUser {
            id: 2,
            keycloak_id: "user".to_string(),
            username: "user".to_string(),
            email: "user@test.com".to_string(),
            display_name: None,
            roles: vec!["user".to_string()],
            is_active: true,
        };

        // Admin can access any user's data
        assert!(can_access_user_data(&admin_user, 1));
        assert!(can_access_user_data(&admin_user, 2));
        assert!(can_access_user_data(&admin_user, 999));

        // Regular user can only access their own data
        assert!(can_access_user_data(&regular_user, 2));
        assert!(!can_access_user_data(&regular_user, 1));
        assert!(!can_access_user_data(&regular_user, 999));
    }

    #[tokio::test]
    async fn test_generate_test_token() {
        let token = utils::generate_test_token(
            "test-user-id",
            "testuser",
            "test@example.com",
            vec!["user".to_string()],
            "test-secret",
        )
        .unwrap();

        assert!(!token.is_empty());
        assert!(token.contains('.'));
    }
}
