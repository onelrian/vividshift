use diesel::prelude::*;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef, State, Json},
    http::{header, request::Parts, StatusCode},
    response::IntoResponse,
};
use tracing::{error, info, warn};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::models::{UserRole, NewUser};
use crate::schema::users;
use crate::db::DbPool;
use crate::auth::{hash_password, verify_password};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserRole,
}

pub struct AuthenticatedUser {
    pub claims: Claims,
    pub user: UserRole,
}

pub struct AdminUser(#[allow(dead_code)] pub AuthenticatedUser);

// Simplified AuthState - no longer needs Supabase URL
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
}

impl AuthState {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

pub async fn login(
    State(pool): State<DbPool>,
    State(settings): State<Arc<crate::config::Settings>>,
    Json(payload): Json<LoginRequest>,
) ->  Result<impl IntoResponse, (StatusCode, String)> {
    info!("Login attempt for email: {}", payload.email);
    let mut conn = pool.get().map_err(|e| {
        error!("DB connection error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    // Find user by email
    let user = users::table
        .filter(users::email.eq(&payload.email))
        .first::<UserRole>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Login DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    let password_valid = match &user.password_hash {
        Some(hash) => verify_password(&payload.password, hash).unwrap_or(false),
        None => false, // No password hash means verify fail
    };

    if !password_valid {
        warn!("Failed login attempt for email: {}", payload.email);
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    info!("User logged in successfully: {}", user.email);
    // Generate JWT
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        error!("Token creation error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed".to_string())
    })?;

    Ok(Json(AuthResponse {
        token,
        user,
    }))
}

pub async fn register(
    State(pool): State<DbPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Registration attempt for email: {}", payload.email);
    let mut conn = pool.get().map_err(|e| {
        error!("DB connection error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    // Check if email already exists
    let existing = users::table
        .filter(users::email.eq(&payload.email))
        .first::<UserRole>(&mut conn)
        .optional()
        .map_err(|e| {
            error!("Register DB check error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
        })?;

    if existing.is_some() {
        warn!("Registration attempt for existing email: {}", payload.email);
        return Err((StatusCode::BAD_REQUEST, "Email already registered".to_string()));
    }

    // Hash password
    let hashed_pwd = hash_password(&payload.password).map_err(|e| {
        error!("Password hashing error: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
    })?;

    let new_user = NewUser {
        id: Uuid::new_v4().to_string(),
        username: payload.username,
        email: payload.email,
        role: "USER".to_string(), // Default role is USER
        password_hash: hashed_pwd,
    };

    let created_user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<UserRole>(&mut conn)
        .map_err(|e| {
            error!("User creation error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string())
        })?;

    info!("New user registered: {} ({})", created_user.email, created_user.id);
    Ok((StatusCode::CREATED, Json(created_user)))
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    crate::config::Settings: FromRef<S>,
    DbPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let settings = crate::config::Settings::from_ref(state);
        let pool = DbPool::from_ref(state);

        let auth_header = parts.headers.get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];

        let decoding_key = DecodingKey::from_secret(settings.jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        
        // Decode and verify token
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                error!("JWT Verification failed: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        let claims = token_data.claims;
        
        // Verify user still exists in DB
        let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let user = users::table
            .find(&claims.sub)
            .first::<UserRole>(&mut conn)
            .map_err(|e| {
                error!("Failed to fetch user for token: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        Ok(AuthenticatedUser { claims, user })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    crate::config::Settings: FromRef<S>,
    DbPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;
        
        if auth_user.user.role == "ADMIN" {
            Ok(AdminUser(auth_user))
        } else {
            error!("Unauthorized access attempt by non-admin: {}", auth_user.user.email);
            Err(StatusCode::FORBIDDEN)
        }
    }
}
