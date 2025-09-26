use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use std::collections::HashMap;
use tower::ServiceExt;
use vividshift_backend::{
    config::AppConfig,
    engines::assignment::BalancedRotationStrategy,
    engines::validation::{AvailabilityCheckValidator, CapacityCheckValidator},
    services::{EntityManager, RuleEngine},
    auth::{jwt::JwtService, AuthState},
};

async fn create_test_app() -> axum::Router {
    let config = std::sync::Arc::new(AppConfig::default());
    
    // Initialize entity manager
    let entity_manager = std::sync::Arc::new(EntityManager::new(config.domain.clone()));
    
    // Initialize rule engine
    let mut rule_engine = RuleEngine::new(config.rule_engine.clone());
    rule_engine.register_strategy(BalancedRotationStrategy::new());
    rule_engine.register_validator(CapacityCheckValidator);
    rule_engine.register_validator(AvailabilityCheckValidator);
    let rule_engine = std::sync::Arc::new(rule_engine);

    // Initialize JWT service
    let jwt_service = std::sync::Arc::new(JwtService::new(
        &config.auth.jwt_secret,
        config.auth.jwt_expiration,
    ));
    let auth_state = AuthState { jwt_service };

    // Create router
    vividshift_backend::api::create_router(config, auth_state, entity_manager, rule_engine)
}

#[tokio::test]
async fn test_health_endpoints() {
    let app = create_test_app().await;

    // Test health endpoint
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test readiness endpoint
    let request = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_endpoints() {
    let app = create_test_app().await;

    // Test login with valid credentials
    let login_request = json!({
        "username": "admin",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(login_request.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test login with invalid credentials
    let invalid_login = json!({
        "username": "admin",
        "password": "wrongpassword"
    });

    let request = Request::builder()
        .uri("/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(invalid_login.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_endpoints_require_auth() {
    let app = create_test_app().await;

    // Test accessing protected endpoint without token
    let request = Request::builder()
        .uri("/api/entities/participant")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

async fn get_auth_token(app: &axum::Router) -> String {
    let login_request = json!({
        "username": "admin",
        "password": "password123"
    });

    let request = Request::builder()
        .uri("/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(login_request.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let auth_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    auth_response["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_entity_crud_endpoints() {
    let app = create_test_app().await;
    let token = get_auth_token(&app).await;

    // Test creating an entity
    let create_request = json!({
        "name": "Test Participant",
        "availability": true,
        "group": "TestGroup"
    });

    let request = Request::builder()
        .uri("/api/entities/participant")
        .method("POST")
        .header("authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(create_request.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test listing entities
    let request = Request::builder()
        .uri("/api/entities/participant")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rule_engine_endpoints() {
    let app = create_test_app().await;
    let token = get_auth_token(&app).await;

    // Test listing strategies
    let request = Request::builder()
        .uri("/api/rules/strategies")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let strategies: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert!(strategies.contains(&"balanced_rotation".to_string()));

    // Test listing validators
    let request = Request::builder()
        .uri("/api/rules/validators")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let validators: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert!(validators.contains(&"capacity_check".to_string()));
    assert!(validators.contains(&"availability_check".to_string()));
}
