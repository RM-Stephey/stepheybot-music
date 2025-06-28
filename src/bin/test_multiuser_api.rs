//! Test binary for StepheyBot Music Multi-User API
//!
//! This is a simple test server that demonstrates the multi-user functionality
//! we've implemented. It sets up the database, services, and API routes.

use jsonwebtoken::{Algorithm, Validation};
use stepheybot_music::api::{create_api_router, ApiState};
use stepheybot_music::auth::{AuthConfig, AuthService};
use stepheybot_music::database::Database;
use stepheybot_music::services::UserService;

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "test_multiuser_api=info,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .try_init()?;

    info!("🎵 Starting StepheyBot Music Multi-User API Test Server");

    // Create test database (in-memory for testing)
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:test_multiuser.db".to_string());

    info!("📦 Connecting to database: {}", database_url);
    let database = Database::new(&database_url).await?;

    // Run migrations
    info!("🔧 Running database migrations...");
    database.migrate().await?;
    info!("✅ Database migrations completed");

    // Create services
    info!("🛠️ Initializing services...");

    let user_service = Arc::new(UserService::new(Arc::new(database)));
    info!("👤 User service initialized");

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "test_secret_key_for_development_only_do_not_use_in_production".to_string()
    });

    // Create auth config
    let auth_config = AuthConfig {
        keycloak_realm_url: "http://localhost:8080/auth/realms/stepheybot".to_string(),
        keycloak_client_id: "stepheybot-music".to_string(),
        jwt_secret,
        jwt_algorithm: Algorithm::HS256,
        token_validation: Validation::default(),
    };

    let auth_service = Arc::new(AuthService::new(auth_config, user_service.clone())?);
    info!("🔐 Auth service initialized");

    // Create API state
    let api_state = ApiState {
        user_service,
        auth_service,
    };

    // Create the API router using our new multi-user structure
    info!("🌐 Creating API router...");
    let api_router = create_api_router(api_state);

    // Create the main application router
    let app = Router::new().nest("/api", api_router).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive()),
    );

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("🚀 Server starting on http://{}", addr);

    // Print available endpoints
    print_available_endpoints(port);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 StepheyBot Music Multi-User API Test Server shutdown complete");
    Ok(())
}

fn print_available_endpoints(port: u16) {
    info!("📋 Available API endpoints:");
    info!("   Health Check:     http://127.0.0.1:{}/api/health", port);
    info!("   Version Info:     http://127.0.0.1:{}/api/version", port);
    info!(
        "   User Auth:        http://127.0.0.1:{}/api/v1/auth/login",
        port
    );
    info!(
        "   User Profile:     http://127.0.0.1:{}/api/v1/user/profile",
        port
    );
    info!(
        "   User Preferences: http://127.0.0.1:{}/api/v1/user/preferences",
        port
    );
    info!(
        "   User Dashboard:   http://127.0.0.1:{}/api/v1/user/dashboard",
        port
    );
    info!(
        "   Library Stats:    http://127.0.0.1:{}/api/v1/library/stats",
        port
    );
    info!(
        "   Playlists:        http://127.0.0.1:{}/api/v1/playlists",
        port
    );
    info!(
        "   Recommendations:  http://127.0.0.1:{}/api/v1/recommendations",
        port
    );
    info!(
        "   Integrations:     http://127.0.0.1:{}/api/v1/integrations/status",
        port
    );
    info!("");
    info!("🧪 Test the API with:");
    info!("   curl http://127.0.0.1:{}/api/health", port);
    info!("   curl http://127.0.0.1:{}/api/version", port);
    info!("   curl http://127.0.0.1:{}/api/v1/library/stats", port);
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down gracefully...");
        },
    }
}
