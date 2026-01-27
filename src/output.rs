// src/output.rs

use std::collections::HashMap;
use tracing::info;

/// Prints the assignments in a clean, formatted way.
pub fn print_assignments(assignments: &HashMap<String, Vec<String>>) {
    info!("📊 Work Distribution Results");

    let mut sorted_areas: Vec<_> = assignments.keys().collect();
    sorted_areas.sort();

    for area in sorted_areas {
        let people = &assignments[area];
        // Sort people's names for consistent output
        let mut sorted_people = people.clone();
        sorted_people.sort();
        info!("➡️  {:<12}: {}", area, sorted_people.join(", "));
    }
}
