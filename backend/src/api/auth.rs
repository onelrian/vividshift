use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::AppState,
    auth::{
        models::{AuthResponse, LoginRequest, RegisterRequest, UserInfo, UserRole},
    },
};

pub async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate request
    if let Err(_) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // In a real application, you'd verify against database
    // For demo purposes, we'll use hardcoded credentials
    if request.username == "admin" && request.password == "password123" {
        let jwt_service = &app_state.auth_state.jwt_service;

        let user_id = Uuid::new_v4();
        let token = jwt_service
            .generate_token(&user_id.to_string(), &UserRole::Admin.to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let user_info = UserInfo {
            id: user_id,
            username: request.username,
            email: "admin@vividshift.com".to_string(),
            role: UserRole::Admin,
        };

        Ok(Json(AuthResponse {
            token,
            user: user_info,
        }))
    } else if request.username == "user" && request.password == "password123" {
        let jwt_service = &app_state.auth_state.jwt_service;

        let user_id = Uuid::new_v4();
        let token = jwt_service
            .generate_token(&user_id.to_string(), &UserRole::User.to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let user_info = UserInfo {
            id: user_id,
            username: request.username,
            email: "user@vividshift.com".to_string(),
            role: UserRole::User,
        };

        Ok(Json(AuthResponse {
            token,
            user: user_info,
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Validate request
    if let Err(_) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // In a real application, you'd:
    // 1. Check if username/email already exists
    // 2. Hash the password with bcrypt
    // 3. Save to database
    // 4. Send verification email

    let jwt_service = &app_state.auth_state.jwt_service;

    let user_id = Uuid::new_v4();
    let token = jwt_service
        .generate_token(&user_id.to_string(), &UserRole::User.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_info = UserInfo {
        id: user_id,
        username: request.username,
        email: request.email,
        role: UserRole::User,
    };

    tracing::info!("New user registered: {}", user_info.username);

    Ok(Json(AuthResponse {
        token,
        user: user_info,
    }))
}
