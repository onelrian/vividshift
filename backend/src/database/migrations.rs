use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn, error};

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Starting database migrations...");
        
        match sqlx::migrate!("./migrations").run(&self.pool).await {
            Ok(_) => {
                info!("✅ Database migrations completed successfully");
                Ok(())
            }
            Err(e) => {
                error!("❌ Database migration failed: {}", e);
                Err(anyhow::anyhow!("Migration failed: {}", e))
            }
        }
    }

    /// Check migration status
    pub async fn check_migration_status(&self) -> Result<Vec<MigrationInfo>> {
        info!("Checking migration status...");
        
        let migrations = sqlx::query!(
            r#"
            SELECT version, description, installed_on, success, checksum, execution_time
            FROM _sqlx_migrations 
            ORDER BY version
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let migration_info: Vec<MigrationInfo> = migrations
            .into_iter()
            .map(|row| MigrationInfo {
                version: row.version,
                description: row.description,
                installed_on: row.installed_on,
                success: row.success,
                checksum: row.checksum,
                execution_time: row.execution_time,
            })
            .collect();

        info!("Found {} applied migrations", migration_info.len());
        Ok(migration_info)
    }

    /// Revert last migration (if supported)
    pub async fn revert_migration(&self) -> Result<()> {
        warn!("⚠️  Migration revert requested - this is a dangerous operation!");
        
        // Note: SQLx doesn't support automatic rollbacks, but we can provide guidance
        warn!("SQLx doesn't support automatic rollbacks. To revert:");
        warn!("1. Manually run the corresponding .down.sql file");
        warn!("2. Or restore from a database backup");
        warn!("3. Then run migrations again");
        
        Err(anyhow::anyhow!("Manual rollback required - see logs for instructions"))
    }

    /// Validate database schema against expected state
    pub async fn validate_schema(&self) -> Result<SchemaValidation> {
        info!("Validating database schema...");
        
        let mut validation = SchemaValidation {
            is_valid: true,
            missing_tables: Vec::new(),
            missing_indexes: Vec::new(),
            issues: Vec::new(),
        };

        // Check for required tables
        let required_tables = vec![
            "users", "user_sessions", "user_profiles",
            "domains", "entity_definitions", "entities",
            "participants", "assignment_targets", "assignments",
            "assignment_details", "assignment_history", "rule_configurations"
        ];

        for table in required_tables {
            let exists = sqlx::query!(
                "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
                table
            )
            .fetch_one(&self.pool)
            .await?;

            if !exists.exists.unwrap_or(false) {
                validation.is_valid = false;
                validation.missing_tables.push(table.to_string());
            }
        }

        // Check for required extensions
        let extensions = sqlx::query!(
            "SELECT extname FROM pg_extension WHERE extname IN ('uuid-ossp', 'btree_gin')"
        )
        .fetch_all(&self.pool)
        .await?;

        if extensions.len() < 2 {
            validation.is_valid = false;
            validation.issues.push("Missing required PostgreSQL extensions".to_string());
        }

        // Check for critical indexes
        let critical_indexes = vec![
            "idx_users_username", "idx_users_email",
            "idx_participants_name", "idx_assignment_targets_name",
            "idx_assignments_assignment_date"
        ];

        for index in critical_indexes {
            let exists = sqlx::query!(
                "SELECT EXISTS (SELECT FROM pg_indexes WHERE indexname = $1)",
                index
            )
            .fetch_one(&self.pool)
            .await?;

            if !exists.exists.unwrap_or(false) {
                validation.missing_indexes.push(index.to_string());
            }
        }

        if validation.is_valid {
            info!("✅ Database schema validation passed");
        } else {
            warn!("⚠️  Database schema validation found issues");
        }

        Ok(validation)
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        info!("Collecting database statistics...");
        
        let table_stats = sqlx::query!(
            r#"
            SELECT 
                schemaname,
                relname as tablename,
                n_tup_ins as inserts,
                n_tup_upd as updates,
                n_tup_del as deletes,
                n_live_tup as live_tuples,
                n_dead_tup as dead_tuples
            FROM pg_stat_user_tables
            ORDER BY n_live_tup DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let database_size = sqlx::query!(
            "SELECT pg_size_pretty(pg_database_size(current_database())) as size"
        )
        .fetch_one(&self.pool)
        .await?;

        let connection_count = sqlx::query!(
            "SELECT count(*) as connections FROM pg_stat_activity WHERE datname = current_database()"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            database_size: database_size.size.unwrap_or("Unknown".to_string()),
            connection_count: connection_count.connections.unwrap_or(0) as u32,
            table_count: table_stats.len() as u32,
            table_stats: table_stats.into_iter().map(|row| TableStats {
                schema_name: row.schemaname.unwrap_or_default(),
                table_name: row.tablename.unwrap_or_default(),
                inserts: row.inserts.unwrap_or(0),
                updates: row.updates.unwrap_or(0),
                deletes: row.deletes.unwrap_or(0),
                live_tuples: row.live_tuples.unwrap_or(0),
                dead_tuples: row.dead_tuples.unwrap_or(0),
            }).collect(),
        })
    }
}

#[derive(Debug)]
pub struct MigrationInfo {
    pub version: i64,
    pub description: String,
    pub installed_on: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub checksum: Vec<u8>,
    pub execution_time: i64,
}

#[derive(Debug)]
pub struct SchemaValidation {
    pub is_valid: bool,
    pub missing_tables: Vec<String>,
    pub missing_indexes: Vec<String>,
    pub issues: Vec<String>,
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub database_size: String,
    pub connection_count: u32,
    pub table_count: u32,
    pub table_stats: Vec<TableStats>,
}

#[derive(Debug)]
pub struct TableStats {
    pub schema_name: String,
    pub table_name: String,
    pub inserts: i64,
    pub updates: i64,
    pub deletes: i64,
    pub live_tuples: i64,
    pub dead_tuples: i64,
}
