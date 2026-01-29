use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use axum::{
    async_trait,
    extract::{FromRequestParts, FromRef},
    http::{header, request::Parts, StatusCode},
};
use tracing::{error, info};
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub exp: usize,
}

pub struct User(pub Claims);

#[derive(Debug, Deserialize)]
struct Jwk {
    _kid: String,
    x: String,
    y: String,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

pub struct AuthState {
    pub supabase_url: String,
    pub public_key: RwLock<Option<DecodingKey>>,
}

impl AuthState {
    pub fn new(supabase_url: String) -> Self {
        Self {
            supabase_url,
            public_key: RwLock::new(None),
        }
    }

    pub async fn get_decoding_key(&self) -> Option<DecodingKey> {
        {
            let lock = self.public_key.read().await;
            if lock.is_some() {
                return lock.clone();
            }
        }

        // Fetch JWKS
        let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", self.supabase_url);
        info!("Fetching JWKS from {}", jwks_url);
        
        match reqwest::get(&jwks_url).await {
            Ok(resp) => {
                if let Ok(jwks) = resp.json::<Jwks>().await {
                    if let Some(key) = jwks.keys.first() {
                        if let Ok(decoding_key) = DecodingKey::from_ec_components(&key.x, &key.y) {
                            let mut lock = self.public_key.write().await;
                            *lock = Some(decoding_key.clone());
                            return Some(decoding_key);
                        }
                    }
                }
            }
            Err(e) => error!("Failed to fetch JWKS: {}", e),
        }

        None
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    Arc<AuthState>: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = Arc::<AuthState>::from_ref(state);

        let auth_header = parts.headers.get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];

        let decoding_key = auth_state.get_decoding_key().await
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let validation = Validation::new(Algorithm::ES256);
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| {
                error!("JWT Verification failed: {}", e);
                StatusCode::UNAUTHORIZED
            })?;

        Ok(User(token_data.claims))
    }
}
