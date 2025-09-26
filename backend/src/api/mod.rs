pub mod auth;
pub mod work_groups;
pub mod health;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::auth::AuthState;
use crate::config::AppConfig;
use crate::services::{EntityManager, RuleEngine};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_state: AuthState,
    pub entity_manager: Arc<EntityManager>,
    pub rule_engine: Arc<RuleEngine>,
}

pub fn create_router(
    config: Arc<AppConfig>, 
    auth_state: AuthState,
    entity_manager: Arc<EntityManager>,
    rule_engine: Arc<RuleEngine>,
) -> Router {
    let app_state = AppState {
        config,
        auth_state,
        entity_manager,
        rule_engine,
    };

    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        
        // Auth routes
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        
        // Protected routes are handled separately in main.rs
        
        .with_state(app_state)
}
