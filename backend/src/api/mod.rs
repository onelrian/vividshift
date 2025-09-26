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

pub fn create_router(
    config: Arc<AppConfig>, 
    auth_state: AuthState,
    entity_manager: Arc<EntityManager>,
    rule_engine: Arc<RuleEngine>,
) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        
        // Auth routes
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        
        // Generic assignment routes (protected)
        .route("/api/assignments/generate", post(work_groups::generate_assignments))
        .route("/api/assignments/history", get(work_groups::get_history))
        
        // Entity management routes (protected)
        .route("/api/entities/:entity_type", get(work_groups::list_entities))
        .route("/api/entities/:entity_type", post(work_groups::create_entity))
        .route("/api/entities/:entity_type/:entity_id", get(work_groups::get_entity))
        .route("/api/entities/:entity_type/:entity_id", put(work_groups::update_entity))
        .route("/api/entities/:entity_type/:entity_id", delete(work_groups::delete_entity))
        
        // Rule engine routes (protected)
        .route("/api/rules/strategies", get(work_groups::list_strategies))
        .route("/api/rules/validators", get(work_groups::list_validators))
        
        // Legacy work group routes for backward compatibility
        .route("/api/work-groups/generate", post(work_groups::generate_work_groups_legacy))
        .route("/api/work-groups/assignments", get(work_groups::get_assignments_config))
        .route("/api/work-groups/assignments", post(work_groups::update_assignments_config))
        
        .with_state(auth_state)
        .with_state(config)
        .with_state(entity_manager)
        .with_state(rule_engine)
}
