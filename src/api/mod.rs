pub mod auth;
pub mod handlers;

use anyhow::Context;
use axum::{
    routing::{get, post, patch, delete},
    Router,
    extract::FromRef,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use crate::api::auth::AuthState;
use crate::db::DbPool;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthState>,
    pub db_pool: DbPool,
    pub settings: Arc<Settings>,
}

impl FromRef<AppState> for Arc<AuthState> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth.clone()
    }
}

impl FromRef<AppState> for DbPool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}

impl FromRef<AppState> for Arc<Settings> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.settings.clone()
    }
}

// Support simply extracting Settings if needed (clones the inner settings)
impl FromRef<AppState> for Settings {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.settings.as_ref().clone()
    }
}

pub async fn start_server(settings: Settings, db_pool: DbPool) -> anyhow::Result<()> {
    let jwt_secret = settings.jwt_secret.clone();
    
    let auth_state = Arc::new(AuthState::new(jwt_secret));
    
    let app_state = AppState {
        auth: auth_state,
        db_pool,
        settings: Arc::new(settings),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        // Auth Routes
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        // App Routes
        .route("/api/dashboard", get(handlers::get_dashboard_data))
        .route("/api/shuffle", post(handlers::trigger_shuffle))
        // People Management
        .route("/api/people", get(handlers::list_people))
        .route("/api/people", post(handlers::create_person))
        .route("/api/people/:id", patch(handlers::update_person))
        .route("/api/people/:id", delete(handlers::delete_person))
        // Settings
        .route("/api/settings", get(handlers::get_app_settings))
        .route("/api/settings", post(handlers::update_app_setting))
        // Auth/Profile
        .route("/api/auth/profile", get(handlers::get_profile))
        .route("/api/auth/profile", patch(handlers::update_profile))
        // Admin User Management
        .route("/api/admin/users", get(handlers::list_users))
        .route("/api/admin/users/:id/role", patch(handlers::update_user_role))
        // Assignments
        .route("/api/assignments", get(handlers::get_assignment_history))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("📡 API Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
