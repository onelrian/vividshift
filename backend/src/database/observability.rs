use std::time::{Duration, Instant};
use sqlx::PgPool;
use tracing::{info, warn, error};
use anyhow::Result;
use crate::database::connection::PoolStats;

/// Database observability and metrics collection
pub struct DatabaseObservability {
    pool: PgPool,
}

impl DatabaseObservability {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record database operation metrics
    pub fn record_operation(_operation: &str, duration: Duration, success: bool) {
        // Metrics recording would be implemented here with proper metrics crate setup
        info!("Database operation: duration={:.2}ms, success={}", duration.as_millis(), success);
    }

    /// Record connection pool metrics
    pub fn record_pool_metrics(&self) {
        let stats = self.get_pool_stats();
        info!("Pool stats: active={}, idle={}", stats.size, stats.idle);
    }

    /// Get connection pool statistics
    fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Monitor slow queries (requires pg_stat_statements extension)
    pub async fn monitor_slow_queries(&self, _threshold_ms: u64) -> Result<()> {
        // Simplified implementation - would require pg_stat_statements extension
        info!("Slow query monitoring requires pg_stat_statements extension");
        Ok(())
    }

    /// Monitor database locks
    pub async fn monitor_locks(&self) -> Result<()> {
        let locks = sqlx::query!(
            r#"
            SELECT 
                mode,
                COUNT(*) as count
            FROM pg_locks 
            WHERE NOT granted 
            GROUP BY mode
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let total_locks = locks.iter().map(|l| l.count.unwrap_or(0)).sum::<i64>();
        
        if total_locks > 0 {
            warn!("Database locks detected: {} total", total_locks);
            
            for lock in locks {
                let mode = lock.mode.as_deref().unwrap_or("unknown");
                let count = lock.count.unwrap_or(0);
                info!("Lock mode '{}': {} waiting", mode, count);
            }
        }

        Ok(())
    }

    /// Monitor database size and growth
    pub async fn monitor_database_size(&self) -> Result<()> {
        let size_info = sqlx::query!(
            r#"
            SELECT 
                pg_database_size(current_database()) as db_size,
                (SELECT COUNT(*) FROM pg_stat_user_tables) as table_count
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(db_size) = size_info.db_size {
            info!("Database size: {} bytes", db_size);
        }

        if let Some(table_count) = size_info.table_count {
            info!("Table count: {}", table_count);
        }

        Ok(())
    }

    /// Monitor connection statistics
    pub async fn monitor_connections(&self) -> Result<()> {
        let conn_stats = sqlx::query!(
            r#"
            SELECT 
                state,
                COUNT(*) as count
            FROM pg_stat_activity 
            WHERE datname = current_database()
            GROUP BY state
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for stat in conn_stats {
            let state = stat.state.as_deref().unwrap_or("unknown");
            let count = stat.count.unwrap_or(0);
            
            info!("Connection state '{}': {} connections", state, count);
        }

        Ok(())
    }

    /// Check for database health issues
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        let mut health = DatabaseHealth::new();

        // Check basic connectivity
        let start = Instant::now();
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => {
                health.connectivity = true;
                health.response_time = start.elapsed();
            }
            Err(e) => {
                error!("Database connectivity check failed: {}", e);
                health.connectivity = false;
                health.errors.push(format!("Connectivity: {}", e));
            }
        }

        // Check for long-running transactions
        match sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM pg_stat_activity 
            WHERE state = 'active' 
            AND query_start < NOW() - INTERVAL '5 minutes'
            "#
        )
        .fetch_one(&self.pool)
        .await {
            Ok(result) => {
                let long_running = result.count.unwrap_or(0);
                if long_running > 0 {
                    health.warnings.push(format!("{} long-running transactions detected", long_running));
                }
            }
            Err(e) => {
                health.errors.push(format!("Long-running transaction check: {}", e));
            }
        }

        // Check disk space (if accessible)
        match sqlx::query!(
            r#"
            SELECT 
                pg_size_pretty(pg_database_size(current_database())) as size,
                pg_database_size(current_database()) as size_bytes
            "#
        )
        .fetch_one(&self.pool)
        .await {
            Ok(result) => {
                if let Some(size_bytes) = result.size_bytes {
                    health.database_size = size_bytes;
                    // Warn if database is larger than 1GB (configurable threshold)
                    if size_bytes > 1_000_000_000 {
                        health.warnings.push(format!("Database size is large: {}", 
                            result.size.as_deref().unwrap_or("unknown")));
                    }
                }
            }
            Err(e) => {
                health.errors.push(format!("Database size check: {}", e));
            }
        }

        Ok(health)
    }

    /// Start background monitoring task
    pub async fn start_monitoring(&self, interval_seconds: u64) {
        let pool = self.pool.clone();
        let interval = Duration::from_secs(interval_seconds);
        
        tokio::spawn(async move {
            let observability = DatabaseObservability::new(pool);
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Record pool metrics
                observability.record_pool_metrics();
                
                // Monitor various aspects
                if let Err(e) = observability.monitor_database_size().await {
                    error!("Failed to monitor database size: {}", e);
                }
                
                if let Err(e) = observability.monitor_connections().await {
                    error!("Failed to monitor connections: {}", e);
                }
                
                if let Err(e) = observability.monitor_locks().await {
                    error!("Failed to monitor locks: {}", e);
                }
                
                // Monitor slow queries less frequently
                if interval_timer.period().as_secs() % 60 == 0 {
                    if let Err(e) = observability.monitor_slow_queries(1000).await {
                        error!("Failed to monitor slow queries: {}", e);
                    }
                }
            }
        });
    }
}


#[derive(Debug)]
pub struct DatabaseHealth {
    pub connectivity: bool,
    pub response_time: Duration,
    pub database_size: i64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl DatabaseHealth {
    fn new() -> Self {
        Self {
            connectivity: false,
            response_time: Duration::from_secs(0),
            database_size: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.connectivity && self.errors.is_empty()
    }
}

/// Middleware to record database operation metrics
pub struct DatabaseMetricsMiddleware;

impl DatabaseMetricsMiddleware {
    pub fn record_query_start() -> Instant {
        Instant::now()
    }

    pub fn record_query_end(operation: &str, start: Instant, success: bool) {
        let duration = start.elapsed();
        DatabaseObservability::record_operation(operation, duration, success);
    }
}
