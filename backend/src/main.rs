mod api;
mod auth;
mod config;
mod database;
mod engines;
mod models;
mod services;

use anyhow::Result;
use axum::{
    middleware,
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::{jwt::JwtService, middleware::auth_middleware, AuthState},
    config::AppConfig,
    database::{DatabaseManager, RepositoryManager, SeedManager, run_migrations, health_check},
    engines::{
        assignment::BalancedRotationStrategy,
        validation::{AvailabilityCheckValidator, CapacityCheckValidator},
    },
    services::{logging, EntityManager, RuleEngine},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    dotenvy::dotenv().ok();
    let config = Arc::new(AppConfig::new()?);
    
    // Initialize logging
    logging::init_logging(&config)?;
    
    tracing::info!("🚀 Starting VividShift Generic Assignment Engine...");
    tracing::info!("Environment: {}", config.server.environment);
    tracing::info!("Domain: {}", config.domain.display_name);
    tracing::info!("Server will bind to: {}:{}", config.server.host, config.server.port);

    // Validate domain configuration
    if let Err(e) = config.validate_domain_config() {
        tracing::error!("Domain configuration validation failed: {}", e);
        return Err(anyhow::anyhow!("Invalid domain configuration: {}", e));
    }

    // Initialize database connection
    tracing::info!("🗄️  Initializing database connection...");
    let db_manager = Arc::new(DatabaseManager::new(&config.database).await?);
    
    // Run database migrations
    if let Err(e) = run_migrations(db_manager.pool()).await {
        tracing::error!("Database migration failed: {}", e);
        return Err(e);
    }
    
    // Perform database health check
    if let Err(e) = health_check(db_manager.pool()).await {
        tracing::error!("Database health check failed: {}", e);
        return Err(e);
    }

    // Initialize repository manager
    let repo_manager = Arc::new(RepositoryManager::new(db_manager.pool_clone()));

    // Seed database in development environment
    if config.is_development() {
        tracing::info!("🌱 Seeding database with sample data...");
        let seed_manager = SeedManager::new((*repo_manager).clone(), db_manager.pool_clone());
        if let Err(e) = seed_manager.seed_all(false).await {
            tracing::warn!("Database seeding failed: {}", e);
        } else {
            tracing::info!("✅ Database seeding completed");
        }
    }

    // Initialize entity manager (keeping for backward compatibility)
    let entity_manager = Arc::new(EntityManager::new(config.domain.clone()));
    
    // Load default data
    if let Err(e) = entity_manager.load_default_data().await {
        tracing::warn!("Failed to load default data: {}", e);
    }

    // Initialize rule engine
    let mut rule_engine = RuleEngine::new(config.rule_engine.clone());
    
    // Register assignment strategies
    rule_engine.register_strategy(BalancedRotationStrategy::new());
    
    // Register validation rules
    rule_engine.register_validator(CapacityCheckValidator);
    rule_engine.register_validator(AvailabilityCheckValidator);
    
    let rule_engine = Arc::new(rule_engine);

    // Initialize JWT service
    let jwt_service = Arc::new(JwtService::new(
        &config.auth.jwt_secret,
        config.auth.jwt_expiration,
    ));

    let auth_state = AuthState { jwt_service };

    // Create router
    let app = create_app(config.clone(), auth_state, entity_manager, rule_engine, repo_manager, db_manager.clone()).await?;

    // Start server
    let listener = TcpListener::bind(format!("{}:{}", config.server.host, config.server.port)).await?;
    
    tracing::info!("✅ Server listening on http://{}:{}", config.server.host, config.server.port);
    tracing::info!("📊 Health check available at: http://{}:{}/health", config.server.host, config.server.port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_app(
    config: Arc<AppConfig>, 
    auth_state: AuthState,
    entity_manager: Arc<EntityManager>,
    rule_engine: Arc<RuleEngine>,
    repo_manager: Arc<RepositoryManager>,
    db_manager: Arc<DatabaseManager>,
) -> Result<Router> {
    // Create the main router with public routes
    let public_router = api::create_router(
        config.clone(), 
        auth_state.clone(), 
        entity_manager.clone(), 
        rule_engine.clone()
    );
    
    // Create protected routes that require authentication
    let protected_router = Router::new()
        .route("/api/work-groups/generate", axum::routing::post(api::work_groups::generate_work_groups_legacy))
        .route("/api/work-groups/history", axum::routing::get(api::work_groups::get_history))
        .route("/api/work-groups/assignments", 
               axum::routing::get(api::work_groups::get_assignments_config)
               .post(api::work_groups::update_assignments_config))
        .with_state(api::AppState {
            config: config.clone(),
            auth_state: auth_state.clone(),
            entity_manager: entity_manager.clone(),
            rule_engine: rule_engine.clone(),
        })
        .route_layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .merge(public_router)
        .merge(protected_router)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    Ok(app)
}