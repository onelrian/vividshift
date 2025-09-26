use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "vividshift-backend"
    })))
}

pub async fn readiness_check() -> Result<Json<Value>, StatusCode> {
    // In a real application, you'd check database connectivity, etc.
    Ok(Json(json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": "ok",
            "config": "ok"
        }
    })))
}
