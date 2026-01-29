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
use crate::db::{self, DbPool};

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthState>,
    pub db_pool: DbPool,
    pub settings: Settings,
}

impl FromRef<AppState> for Arc<AuthState> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth.clone()
    }
}

pub async fn start_server(settings: Settings) -> anyhow::Result<()> {
    let supabase_url = env::var("VITE_SUPABASE_URL")
        .context("VITE_SUPABASE_URL must be set for authentication")?;
    
    let db_pool = db::establish_connection(&settings.database_url)?;
    let auth_state = Arc::new(AuthState::new(supabase_url));
    
    let app_state = AppState {
        auth: auth_state,
        db_pool,
        settings: settings.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
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
