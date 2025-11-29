mod db;
mod group;
mod models;
mod output;
mod schema;

use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

fn set_github_output(should_notify: bool) {
    if let Ok(github_env) = env::var("GITHUB_ENV") {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(github_env)
            .expect("Failed to open GITHUB_ENV file");

        writeln!(file, "SHOULD_NOTIFY={}", should_notify).expect("Failed to write to GITHUB_ENV");
    } else {
        println!("⚠️ GITHUB_ENV not set, skipping environment variable update.");
    }
}

fn main() {
    println!("🚀 Starting Work Group Generator...");

    // 1. Connect to DB
    let pool = db::establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");

    // 2. Check Schedule (14 day rule)
    match db::should_run(&mut conn) {
        Ok(true) => println!("✅ It has been 14+ days (or first run). Proceeding."),
        Ok(false) => {
            println!("⏳ It has NOT been 14 days since the last run. Skipping.");
            set_github_output(false);
            return;
        }
        Err(e) => {
            eprintln!("🔥 Error checking schedule: {}", e);
            set_github_output(false);
            return;
        }
    }

    let work_assignments: HashMap<String, usize> = [
        ("Parlor".to_string(), 5),
        ("Frontyard".to_string(), 3),
        ("Backyard".to_string(), 1),
        ("Tank".to_string(), 2),
        ("Toilet B".to_string(), 4),
        ("Toilet A".to_string(), 2),
        ("Bin".to_string(), 1),
    ]
    .into_iter()
    .collect();

    // 3. Load Data from DB
    let (names_a, names_b, name_to_id) =
        db::fetch_people(&mut conn).expect("Failed to fetch people");
    println!(
        "✅ Loaded {} names from Group A and {} names from Group B.",
        names_a.len(),
        names_b.len()
    );

    println!("🔍 Reading assignment history from DB...");
    let history =
        db::fetch_history(&mut conn, &name_to_id).expect("Could not load assignment history");

    // 4. Generate Assignments
    println!("🔄 Generating new work distribution...");
    let mut final_assignments = None;
    const MAX_ATTEMPTS: u32 = 50;

    for attempt in 1..=MAX_ATTEMPTS {
        match group::distribute_work(&names_a, &names_b, &work_assignments, &history) {
            Ok(new_assignments) => {
                println!(
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
        if let Err(e) = db::save_assignments(&mut conn, &assignments, &name_to_id) {
            eprintln!(
                "🔥 CRITICAL ERROR: Failed to save new assignments to DB: {}",
                e
            );
            set_github_output(false); // Don't notify if save failed
        } else {
            println!("\n💾 Assignment history has been saved to the database.");
            set_github_output(true);
        }
    } else {
        eprintln!(
            "🔥 CRITICAL ERROR: Could not find a valid assignment after {} attempts.",
            MAX_ATTEMPTS
        );
        set_github_output(false);
        std::process::exit(1);
    }
}
