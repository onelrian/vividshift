pub mod connection;
pub mod json_migration;
pub mod migrations;
pub mod models;
pub mod observability;
pub mod repositories;
pub mod seeding;
pub mod security;

pub use connection::*;
pub use json_migration::*;
pub use models::*;
pub use observability::*;
pub use repositories::*;
pub use seeding::*;
pub use security::*;
pub use migrations::MigrationManager;

use anyhow::Result;
use sqlx::{PgPool, Row};
use tracing::{info, warn};

/// Database health check
pub async fn health_check(pool: &PgPool) -> Result<()> {
    let row = sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await?;
    
    let result: i32 = row.get("health_check");
    if result == 1 {
        info!("Database health check passed");
        Ok(())
    } else {
        warn!("Database health check failed");
        Err(anyhow::anyhow!("Database health check failed"))
    }
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations completed successfully");
    Ok(())
}
