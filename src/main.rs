mod api;
mod auth;  // Password hashing utilities
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
use tracing::{debug, error, info, warn};

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

    run_distribution_with_conn(&mut conn, &settings, false).await?;
    Ok(())
}

async fn run_distribution_with_conn(
    conn: &mut diesel::PgConnection,
    settings: &config::Settings,
    force: bool,
) -> anyhow::Result<bool> {
    match assignment_engine::perform_distribution(conn, settings, force).await {
        Ok(run) => {
            if run {
                set_github_output(true, settings.github_env_path.as_deref());
            } else {
                set_github_output(false, settings.github_env_path.as_deref());
            }
            Ok(run)
        }
        Err(e) => {
            set_github_output(false, settings.github_env_path.as_deref());
            Err(e)
        }
    }
}

async fn start_background_scheduler(pool: db::DbPool, settings: config::Settings) {
    info!("📅 Background scheduler started.");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Check every hour

    loop {
        interval.tick().await;
        debug!("⏰ Background scheduler checking for work distribution...");
        
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                error!("❌ Scheduler failed to get DB connection: {}", e);
                continue;
            }
        };

        if let Err(e) = run_distribution_with_conn(&mut conn, &settings, false).await {
            error!("❌ Background distribution failed: {}", e);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("🚀 Starting VividShift...");

    let args: Vec<String> = env::args().collect();
    let mode = env::var("VIVIDSHIFT_MODE").unwrap_or_else(|_| {
        if args.contains(&"cli".to_string()) || args.contains(&"distribution".to_string()) {
            "cli".to_string()
        } else {
            "server".to_string()
        }
    });

    if mode == "server" {
        let settings = config::Settings::new().context("Failed to load configuration")?;
        let pool = db::establish_connection(&settings.database_url)?;
        db::init_admin_user(&pool, &settings).await.context("Failed to initialize admin user")?;
        
        // Start background scheduler
        let scheduler_pool = pool.clone();
        let scheduler_settings = settings.clone();
        tokio::spawn(async move {
            start_background_scheduler(scheduler_pool, scheduler_settings).await;
        });

        api::start_server(settings, pool).await?;
    } else {
        run_distribution().await?;
    }

    info!("🎉 Done.");
    Ok(())
}
