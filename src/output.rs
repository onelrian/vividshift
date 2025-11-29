// src/output.rs

use std::collections::HashMap;

/// Prints the assignments in a clean, formatted way.
pub fn print_assignments(assignments: &HashMap<String, Vec<String>>) {
    println!("📊 Work Distribution Results");
    println!("{:=<30}", ""); // Separator line

    let mut sorted_areas: Vec<_> = assignments.keys().collect();
    sorted_areas.sort();

    for area in sorted_areas {
        let people = &assignments[area];
        // Sort people's names for consistent output
        let mut sorted_people = people.clone();
        sorted_people.sort();
        println!("➡️  {:<12}: {}", area, sorted_people.join(", "));
    }
}
