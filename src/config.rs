use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub work_assignments: HashMap<String, usize>,
    pub github_env_path: Option<String>,
    /// Configurable interval in days between assignment shuffles
    /// Defaults to 14 if not specified
    pub assignment_interval_days: Option<i64>,
    pub discord_webhook_url: Option<String>,
    pub discord_enabled: Option<bool>,
    /// Admin email for initial admin account
    pub admin_email: String,
    /// Admin password for initial admin account
    pub admin_password: String,
    /// JWT secret for signing custom JWTs
    pub jwt_secret: String,
    /// Server host (e.g. localhost)
    pub host: String,
    /// Server port (e.g. 8080)
    pub port: u16,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default configurations
            .add_source(File::with_name("config/default").required(false))
            // Add environment specific config (e.g. config/production.toml)
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add environment overrides
            // e.g. APP_DATABASE_URL=postgres://...
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .set_override_option("database_url", std::env::var("DATABASE_URL").ok())?
            .set_override_option("github_env_path", std::env::var("GITHUB_ENV").ok())?
            .set_override_option("discord_webhook_url", std::env::var("DISCORD_WEBHOOK_URL").ok())?
            .set_override_option("discord_enabled", std::env::var("DISCORD_ENABLED").ok().map(|v| v == "true" || v == "1"))?
            .set_override_option("admin_email", std::env::var("ADMIN_EMAIL").ok())?
            .set_override_option("admin_password", std::env::var("ADMIN_PASSWORD").ok())?
            .set_override_option("jwt_secret", std::env::var("JWT_SECRET").ok())?
            .set_override("host", std::env::var("HOST").ok())?
            .set_override("port", std::env::var("PORT").ok())?
            .build()?;

        s.try_deserialize()
    }

    /// Returns the configured assignment interval in days with validation
    /// - Defaults to 14 days if not specified
    /// - Validates range: 1-365 days
    /// - Invalid values are clamped to valid range
    pub fn assignment_interval_days(&self) -> i64 {
        match self.assignment_interval_days {
            Some(interval) => {
                // Validate and clamp to reasonable range
                if interval < 1 {
                    tracing::warn!(
                        "Invalid assignment_interval_days: {}. Defaulting to 14.",
                        interval
                    );
                    14
                } else if interval > 365 {
                    tracing::warn!(
                        "Assignment interval {} exceeds maximum (365 days). Using 365.",
                        interval
                    );
                    365
                } else {
                    interval
                }
            }
            None => 14, // Default to 14 days
        }
    }
}
