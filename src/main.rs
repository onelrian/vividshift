mod files;
mod group;
mod output;

use std::collections::{HashMap, HashSet};
use std::process;

fn main() {
    eprintln!("--- Starting Work Distribution ---");

    let names_a = files::read_names_from_file("file_a.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_a.txt: {}", err);
        process::exit(1);
    });
    let mut names_b = files::read_names_from_file("file_b.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_b.txt: {}", err);
        process::exit(1);
    });

    let group_a_set: HashSet<String> = names_a.iter().cloned().collect();
    let group_b_set: HashSet<String> = names_b.iter().cloned().collect();
    eprintln!(
        "✅ Loaded Group A ({} people) and Group B ({} people).",
        group_a_set.len(),
        group_b_set.len()
    );

    let mut all_names = names_a.clone();
    all_names.append(&mut names_b);
    eprintln!("✅ Successfully read {} total names.", all_names.len());

    // --- Read from the new JSON history file ---
    let mut history = files::read_long_term_history("long_term_history.json").unwrap_or_else(|err| {
        eprintln!("Warning: Could not read history file: {}. Starting fresh.", err);
        HashMap::new()
    });
    eprintln!("✅ Read long-term history for {} people.", history.len());

    let new_assignments =
        group::distribute_work(&mut all_names, &history, &group_a_set, &group_b_set);
    eprintln!("✅ Generated new work assignments with rotation logic.");

    let final_output = output::format_assignments(&new_assignments);

    // 🚨 FIX: use eprintln! so Drone Discord Plugin picks it up
    eprintln!("{}", final_output);

    // --- Update the history for the next run ---
    for (task, people) in &new_assignments {
        for person in people {
            let person_history = history.entry(person.clone()).or_default();
            person_history.insert(0, task.clone()); // Add the new task to the front.
            person_history.truncate(5); // Keep only the last 5 assignments.
        }
    }

    // --- Write to the new JSON history file ---
    if let Err(e) = files::write_long_term_history("long_term_history.json", &history) {
        eprintln!("Error writing long-term history: {}", e);
        process::exit(1);
    }

    eprintln!("\n✅ Successfully updated long_term_history.json for the next cycle.");
    eprintln!("--- Work Distribution Complete ---");
}
