use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::time::Duration;
use tracing::info;

use crate::config::DatabaseConfig;

/// Database connection pool manager
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with connection pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Initializing database connection pool...");
        info!("Database URL: {}", mask_password(&config.url));
        info!("Max connections: {}", config.max_connections);
        info!("Min connections: {}", config.min_connections);
        info!("Connect timeout: {}s", config.connect_timeout);

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(600)) // 10 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            .connect(&config.url)
            .await?;

        // Test the connection
        let row = sqlx::query("SELECT version() as version")
            .fetch_one(&pool)
            .await?;
        
        let version: String = row.get("version");
        info!("Connected to PostgreSQL: {}", version);

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get a clone of the connection pool
    pub fn pool_clone(&self) -> PgPool {
        self.pool.clone()
    }

    /// Close the connection pool gracefully
    pub async fn close(&self) {
        info!("Closing database connection pool...");
        self.pool.close().await;
        info!("Database connection pool closed");
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let mut masked = parsed.clone();
        if parsed.password().is_some() {
            let _ = masked.set_password(Some("****"));
        }
        masked.to_string()
    } else {
        "invalid_url".to_string()
    }
}
