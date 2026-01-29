use anyhow::{Context, Result};
use tracing::{error, info};
use diesel::prelude::*;

use crate::config::Settings;
use crate::db;
use crate::group;
use crate::discord;
use crate::output;

pub async fn perform_distribution(
    conn: &mut PgConnection,
    settings: &Settings,
    force: bool,
) -> Result<bool> {
    info!("🔄 Starting work distribution (force: {})...", force);

    // Fetch settings from DB (database is now source of truth for dynamic settings)
    let db_settings = db::fetch_db_settings(conn).context("Failed to fetch settings from DB")?;
    
    let interval = db_settings.get("assignment_interval_days")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| settings.assignment_interval_days());

    let discord_enabled = db_settings.get("discord_enabled")
        .map(|v| v == "true" || v == "1")
        .or(settings.discord_enabled)
        .unwrap_or(false);
        
    let discord_webhook_url = db_settings.get("discord_webhook_url")
        .cloned()
        .or_else(|| settings.discord_webhook_url.clone());

    // 1. Check Schedule (configurable interval)
    if !force {
        info!("⏱️  Assignment interval configured: {} days", interval);
        
        match db::should_run(conn, interval) {
            Ok(true) => info!("✅ It has been {}+ days (or first run). Proceeding.", interval),
            Ok(false) => {
                info!("⏳ It has NOT been {} days since the last run. Skipping.", interval);
                return Ok(false);
            }
            Err(e) => {
                error!("🔥 Error checking schedule: {}", e);
                return Err(anyhow::anyhow!("Error checking schedule: {}", e));
            }
        }
    }

    let work_areas = &settings.work_assignments;
    info!("📋 Work assignments loaded: {:?}", work_areas.keys());

    // 2. Fetch People
    let (names_a, names_b, name_to_id) =
        db::fetch_people(conn).context("Failed to fetch people")?;
    
    // 3. Fetch History
    info!("🔍 Reading assignment history from DB...");
    let history = db::fetch_history(conn, &name_to_id).context("Failed to fetch history")?;

    // 4. Generate Assignments (Start Retry Loop)
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

    // 5. Save and Output
    if let Some(assignments) = final_assignments {
        output::print_assignments(&assignments);
        if let Err(e) = db::save_assignments(conn, &assignments, &name_to_id) {
            error!(
                "🔥 CRITICAL ERROR: Failed to save new assignments to DB: {}",
                e
            );
            return Err(anyhow::anyhow!("Failed to save assignments: {}", e));
        } else {
            info!("💾 Assignment history has been saved to the database.");

            // 6. Send Discord Notification (if enabled)
            if discord_enabled {
                if let Some(webhook_url) = &discord_webhook_url {
                    info!("🔔 Discord integration enabled. Sending notification...");
                    if let Err(e) = discord::send_notification(webhook_url, &assignments) {
                        error!("⚠️ Failed to send Discord notification: {}", e);
                    }
                }
            }
            return Ok(true);
        }
    } else {
        error!(
            "🔥 CRITICAL ERROR: Could not find a valid assignment after {} attempts.",
            MAX_ATTEMPTS
        );
        anyhow::bail!(
            "Could not find a valid assignment after {} attempts.",
            MAX_ATTEMPTS
        );
    }
}
