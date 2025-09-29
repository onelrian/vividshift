use anyhow::{Result, anyhow};
use sqlx::PgPool;
use tracing::{info, warn, error};
use std::time::Duration;

/// Database security configuration and validation
pub struct DatabaseSecurity {
    pool: PgPool,
}

impl DatabaseSecurity {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Validate database connection security settings
    pub async fn validate_connection_security(&self) -> Result<SecurityReport> {
        let mut report = SecurityReport::new();

        // Check SSL/TLS configuration
        match self.check_ssl_configuration().await {
            Ok(ssl_enabled) => {
                if ssl_enabled {
                    report.ssl_enabled = true;
                    info!("Database SSL/TLS is enabled");
                } else {
                    report.warnings.push("Database SSL/TLS is not enabled".to_string());
                    warn!("Database connection is not using SSL/TLS");
                }
            }
            Err(e) => {
                report.errors.push(format!("Failed to check SSL configuration: {}", e));
            }
        }

        // Check user privileges
        match self.check_user_privileges().await {
            Ok(privileges) => {
                if privileges.is_superuser {
                    report.warnings.push("Database user has superuser privileges".to_string());
                    warn!("Database user should not have superuser privileges in production");
                }
                
                if privileges.can_create_db {
                    report.warnings.push("Database user can create databases".to_string());
                }
                
                report.user_privileges = Some(privileges);
            }
            Err(e) => {
                report.errors.push(format!("Failed to check user privileges: {}", e));
            }
        }

        // Check for default passwords or weak authentication
        match self.check_authentication_strength().await {
            Ok(auth_info) => {
                report.authentication_info = Some(auth_info);
            }
            Err(e) => {
                report.errors.push(format!("Failed to check authentication: {}", e));
            }
        }

        // Check connection limits
        match self.check_connection_limits().await {
            Ok(limits) => {
                if limits.max_connections > 200 {
                    report.warnings.push("High maximum connection limit detected".to_string());
                }
                report.connection_limits = Some(limits);
            }
            Err(e) => {
                report.errors.push(format!("Failed to check connection limits: {}", e));
            }
        }

        Ok(report)
    }

    /// Check if SSL/TLS is enabled
    async fn check_ssl_configuration(&self) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT setting FROM pg_settings WHERE name = 'ssl'"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.setting.as_deref() == Some("on")).unwrap_or(false))
    }

    /// Check current user privileges
    async fn check_user_privileges(&self) -> Result<UserPrivileges> {
        let result = sqlx::query!(
            r#"
            SELECT 
                usesuper as is_superuser,
                usecreatedb as can_create_db,
                userepl as can_replicate,
                usebypassrls as can_bypass_rls
            FROM pg_user 
            WHERE usename = current_user
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserPrivileges {
            is_superuser: result.is_superuser.unwrap_or(false),
            can_create_db: result.can_create_db.unwrap_or(false),
            can_replicate: result.can_replicate.unwrap_or(false),
            can_bypass_rls: result.can_bypass_rls.unwrap_or(false),
        })
    }

    /// Check authentication method and strength
    async fn check_authentication_strength(&self) -> Result<AuthenticationInfo> {
        // This is limited by what we can check from within the database
        let result = sqlx::query!(
            r#"
            SELECT 
                current_user as username,
                inet_client_addr() as client_addr,
                inet_server_addr() as server_addr
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AuthenticationInfo {
            username: result.username.unwrap_or_default(),
            client_addr: result.client_addr.map(|addr| addr.to_string()),
            server_addr: result.server_addr.map(|addr| addr.to_string()),
        })
    }

    /// Check connection limits and current usage
    async fn check_connection_limits(&self) -> Result<ConnectionLimits> {
        let limits = sqlx::query!(
            r#"
            SELECT 
                setting::int as max_connections
            FROM pg_settings 
            WHERE name = 'max_connections'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let current = sqlx::query!(
            r#"
            SELECT COUNT(*) as current_connections
            FROM pg_stat_activity
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ConnectionLimits {
            max_connections: limits.max_connections.unwrap_or(0),
            current_connections: current.current_connections.unwrap_or(0) as i32,
        })
    }

    /// Set up connection timeouts and limits
    pub async fn configure_connection_security(&self, config: &SecurityConfig) -> Result<()> {
        // Set statement timeout to prevent long-running queries
        if let Some(timeout) = config.statement_timeout {
            let timeout_ms = timeout.as_millis() as i32;
            sqlx::query(&format!("SET statement_timeout = {}", timeout_ms))
                .execute(&self.pool)
                .await?;
            info!("Set statement timeout to {}ms", timeout_ms);
        }

        // Set lock timeout
        if let Some(timeout) = config.lock_timeout {
            let timeout_ms = timeout.as_millis() as i32;
            sqlx::query(&format!("SET lock_timeout = {}", timeout_ms))
                .execute(&self.pool)
                .await?;
            info!("Set lock timeout to {}ms", timeout_ms);
        }

        // Set idle in transaction timeout
        if let Some(timeout) = config.idle_in_transaction_timeout {
            let timeout_ms = timeout.as_millis() as i32;
            sqlx::query(&format!("SET idle_in_transaction_session_timeout = {}", timeout_ms))
                .execute(&self.pool)
                .await?;
            info!("Set idle in transaction timeout to {}ms", timeout_ms);
        }

        Ok(())
    }

    /// Check for security vulnerabilities
    pub async fn security_audit(&self) -> Result<Vec<SecurityIssue>> {
        let mut issues = Vec::new();

        // Check for tables without proper constraints
        let unconstrained_tables = sqlx::query!(
            r#"
            SELECT tablename
            FROM pg_tables 
            WHERE schemaname = 'public'
            AND tablename NOT IN (
                SELECT DISTINCT table_name 
                FROM information_schema.table_constraints 
                WHERE constraint_type = 'PRIMARY KEY'
            )
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for table in unconstrained_tables {
            if let Some(tablename) = table.tablename {
                issues.push(SecurityIssue {
                    severity: SecuritySeverity::Medium,
                    category: "Schema".to_string(),
                    description: format!("Table '{}' has no primary key constraint", tablename),
                    recommendation: "Add a primary key constraint to ensure data integrity".to_string(),
                });
            }
        }

        // Check for unencrypted sensitive columns (basic heuristic)
        let sensitive_columns = sqlx::query!(
            r#"
            SELECT table_name, column_name
            FROM information_schema.columns
            WHERE table_schema = 'public'
            AND (
                column_name ILIKE '%password%' OR
                column_name ILIKE '%secret%' OR
                column_name ILIKE '%token%' OR
                column_name ILIKE '%key%'
            )
            AND data_type = 'text'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for column in sensitive_columns {
            if let (Some(table_name), Some(column_name)) = (column.table_name, column.column_name) {
                // Skip known encrypted fields
                if column_name.contains("hash") || column_name.contains("encrypted") {
                    continue;
                }
                
                issues.push(SecurityIssue {
                    severity: SecuritySeverity::High,
                    category: "Data Protection".to_string(),
                    description: format!("Potentially sensitive column '{}.{}' may not be encrypted", table_name, column_name),
                    recommendation: "Ensure sensitive data is properly encrypted or hashed".to_string(),
                });
            }
        }

        // Check for overly permissive grants
        let public_grants = sqlx::query!(
            r#"
            SELECT grantee, table_name, privilege_type
            FROM information_schema.role_table_grants
            WHERE grantee = 'PUBLIC'
            AND table_schema = 'public'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        if !public_grants.is_empty() {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::High,
                category: "Access Control".to_string(),
                description: format!("Found {} grants to PUBLIC role", public_grants.len()),
                recommendation: "Review and restrict database permissions to specific roles".to_string(),
            });
        }

        Ok(issues)
    }
}

#[derive(Debug)]
pub struct SecurityReport {
    pub ssl_enabled: bool,
    pub user_privileges: Option<UserPrivileges>,
    pub authentication_info: Option<AuthenticationInfo>,
    pub connection_limits: Option<ConnectionLimits>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl SecurityReport {
    fn new() -> Self {
        Self {
            ssl_enabled: false,
            user_privileges: None,
            authentication_info: None,
            connection_limits: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn is_secure(&self) -> bool {
        self.ssl_enabled && self.errors.is_empty() && self.warnings.len() < 3
    }
}

#[derive(Debug)]
pub struct UserPrivileges {
    pub is_superuser: bool,
    pub can_create_db: bool,
    pub can_replicate: bool,
    pub can_bypass_rls: bool,
}

#[derive(Debug)]
pub struct AuthenticationInfo {
    pub username: String,
    pub client_addr: Option<String>,
    pub server_addr: Option<String>,
}

#[derive(Debug)]
pub struct ConnectionLimits {
    pub max_connections: i32,
    pub current_connections: i32,
}

#[derive(Debug)]
pub struct SecurityConfig {
    pub statement_timeout: Option<Duration>,
    pub lock_timeout: Option<Duration>,
    pub idle_in_transaction_timeout: Option<Duration>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            statement_timeout: Some(Duration::from_secs(30)),
            lock_timeout: Some(Duration::from_secs(10)),
            idle_in_transaction_timeout: Some(Duration::from_secs(60)),
        }
    }
}

#[derive(Debug)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub category: String,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}
