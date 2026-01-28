mod config;
mod db;
mod group;
mod models;
mod output;
mod people_config;
mod schema;

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

fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!("🚀 Starting Work Group Generator...");

    // 2. Load Configuration
    let settings = config::Settings::new().context("Failed to load configuration")?;
    info!("✅ Configuration loaded.");

    // 3. Connect to DB
    let pool = db::establish_connection(&settings.database_url);
    let mut conn = pool.get().context("Failed to get DB connection")?;

    // 4. Check Schedule (configurable interval)
    let interval = settings.assignment_interval_days();
    info!("⏱️  Assignment interval configured: {} days", interval);
    
    match db::should_run(&mut conn, interval) {
        Ok(true) => info!("✅ It has been {}+ days (or first run). Proceeding.", interval),
        Ok(false) => {
            info!("⏳ It has NOT been {} days since the last run. Skipping.", interval);
            set_github_output(false, settings.github_env_path.as_deref());
            return Ok(());
        }
        Err(e) => {
            error!("🔥 Error checking schedule: {}", e);
            set_github_output(false, settings.github_env_path.as_deref());
            return Err(anyhow::anyhow!("Error checking schedule: {}", e));
        }
    }

    let work_areas = &settings.work_assignments;
    info!("📋 Work assignments loaded: {:?}", work_areas.keys());

    // 5. Fetch People
    let (names_a, names_b, name_to_id) =
        db::fetch_people(&mut conn).context("Failed to fetch people")?;
    info!(
        "👥 Fetched {} active people (Group A: {}, Group B: {})",
        names_a.len() + names_b.len(),
        names_a.len(),
        names_b.len()
    );

    // 6. Fetch History
    info!("🔍 Reading assignment history from DB...");
    let history = db::fetch_history(&mut conn, &name_to_id).context("Failed to fetch history")?;

    // 7. Generate Assignments (Start Retry Loop)
    info!("🔄 Generating new work distribution...");
    let mut final_assignments = None;
    const MAX_ATTEMPTS: u32 = 500;

    for attempt in 1..=MAX_ATTEMPTS {
        match group::distribute_work(&names_a, &names_b, work_areas, &history) {
            Ok(new_assignments) => {
                info!(
                    "✅ Successfully found a valid assignment on attempt {}!",
                    attempt
                );
                final_assignments = Some(new_assignments);
                break;
            }
            Err(_) => continue,
        }
    }

    // 8. Save and Output
    if let Some(assignments) = final_assignments {
        output::print_assignments(&assignments);
        if let Err(e) = db::save_assignments(&mut conn, &assignments, &name_to_id) {
            error!(
                "🔥 CRITICAL ERROR: Failed to save new assignments to DB: {}",
                e
            );
            set_github_output(false, settings.github_env_path.as_deref());
            return Err(anyhow::anyhow!("Failed to save assignments: {}", e));
        } else {
            info!("💾 Assignment history has been saved to the database.");
            set_github_output(true, settings.github_env_path.as_deref());
        }
    } else {
        error!(
            "🔥 CRITICAL ERROR: Could not find a valid assignment after {} attempts.",
            MAX_ATTEMPTS
        );
        set_github_output(false, settings.github_env_path.as_deref());
        anyhow::bail!(
            "Could not find a valid assignment after {} attempts.",
            MAX_ATTEMPTS
        );
    }

    info!("🎉 Done.");
    Ok(())
}
