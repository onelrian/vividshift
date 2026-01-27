use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub work_assignments: HashMap<String, usize>,
    pub github_env_path: Option<String>,
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
            .build()?;

        s.try_deserialize()
    }
}
