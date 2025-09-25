// src/main.rs

mod history;
mod group;
mod files;
mod output;

use std::collections::HashMap;

fn main() {
    println!("🚀 Starting Work Group Generator...");

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

    let names_a = files::read_names_from_file("file_a.txt").expect("❌ Error reading file_a.txt");
    let names_b = files::read_names_from_file("file_b.txt").expect("❌ Error reading file_b.txt");
    println!("✅ Loaded {} names from file_a.txt and {} names from file_b.txt.", names_a.len(), names_b.len());

    println!("🔍 Reading assignment history...");
    let history = history::load_history().expect("Could not load assignment history");
    if history.is_empty() {
        println!("ℹ️ No previous history found. This looks like a fresh run.");
    } else {
        println!("ℹ️ Found long-term history for {} people.", history.len());
    }

    println!("🔄 Generating new work distribution with long-term rotation rules...");
    
    // --- THE RETRY LOOP ---
    // This makes the application resilient to "unlucky" random shuffles that lead to dead ends.
    let mut final_assignments = None;
    const MAX_ATTEMPTS: u32 = 50;

    for attempt in 1..=MAX_ATTEMPTS {
        match group::distribute_work(&names_a, &names_b, &work_assignments, &history) {
            Ok(new_assignments) => {
                println!("✅ Successfully found a valid assignment on attempt {}!", attempt);
                final_assignments = Some(new_assignments);
                break; // Exit the loop on success
            }
            Err(_) => {
                // If it fails, we simply let the loop continue to try again with a new random shuffle.
                continue;
            }
        }
    }

    // After the loop, check if we actually found a solution.
    if let Some(assignments) = final_assignments {
        output::print_assignments(&assignments);
        if let Err(e) = history::save_history(&assignments, &history) {
            eprintln!("🔥 CRITICAL ERROR: Failed to save new assignment history: {}", e);
        } else {
            println!("\n💾 Assignment history has been updated for the next run.");
        }
    } else {
        eprintln!("🔥 CRITICAL ERROR: Could not find a valid assignment after {} attempts. The constraints and history are likely too restrictive.", MAX_ATTEMPTS);
    }
}