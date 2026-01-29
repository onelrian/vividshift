mod api;
mod config;
mod db;
mod discord;
mod group;
mod models;
mod output;
mod people_config;
mod schema;
mod assignment_engine;

use anyhow::Context;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{error, info, warn};

fn set_github_output(should_notify: bool, env_path: Option<&str>) {
    let path = match env_path {
        Some(p) => p.to_string(),
        None => env::var("GITHUB_ENV").unwrap_or_default(),
    };

    if path.is_empty() {
        warn!("⚠️ GITHUB_ENV not set, skipping environment variable update.");
        return;
    }

    let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open GITHUB_ENV file at {}: {}", path, e);
            return;
        }
    };

    if let Err(e) = writeln!(file, "SHOULD_NOTIFY={}", should_notify) {
        error!("Failed to write to GITHUB_ENV: {}", e);
    }
}

async fn run_distribution() -> anyhow::Result<()> {
    info!("🔄 Running scheduled work distribution...");

    // 1. Load Configuration
    let settings = config::Settings::new().context("Failed to load configuration")?;
    info!("✅ Configuration loaded.");

    // 2. Connect to DB
    let pool = db::establish_connection(&settings.database_url)?;
    let mut conn = pool.get().context("Failed to get DB connection")?;

    match assignment_engine::perform_distribution(&mut conn, &settings, false).await {
        Ok(run) => {
            if run {
                set_github_output(true, settings.github_env_path.as_deref());
            } else {
                set_github_output(false, settings.github_env_path.as_deref());
            }
        }
        Err(e) => {
            set_github_output(false, settings.github_env_path.as_deref());
            return Err(e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!("🚀 Starting VividShift...");

    let mode = env::var("VIVIDSHIFT_MODE").unwrap_or_else(|_| "cli".to_string());

    if mode == "server" {
        let settings = config::Settings::new().context("Failed to load configuration")?;
        api::start_server(settings).await?;
    } else {
        run_distribution().await?;
    }

    info!("🎉 Done.");
    Ok(())
}
