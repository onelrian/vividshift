mod group;
mod files;
mod output;

use group::{distribute_work, AssignmentHistory};
use files::read_names_from_file;
use output::print_assignments;

const HISTORY_FILE: &str = "assignment_history.txt";

fn main() {
    let names_a = match read_names_from_file("file_a.txt") {
        Ok(names) => names,
        Err(e) => {
            eprintln!("Error reading file_a.txt: {}", e);
            return;
        }
    };

    let names_b = match read_names_from_file("file_b.txt") {
        Ok(names) => names,
        Err(e) => {
            eprintln!("Error reading file_b.txt: {}", e);
            return;
        }
    };

    // Load assignment history
    let mut history = match AssignmentHistory::load_from_file(HISTORY_FILE) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Warning: Could not load assignment history: {}", e);
            AssignmentHistory::new()
        }
    };

    // Distribute work based on history
    let assignments = distribute_work(names_a, names_b, &history);

    // Verify no duplicates
    let mut all_assigned = Vec::new();
    for (_, people) in &assignments {
        for person in people {
            all_assigned.push(person.clone());
        }
    }
    
    let unique_assigned: Vec<_> = all_assigned.iter().cloned().collect();
    if unique_assigned.len() != all_assigned.len() {
        eprintln!("Warning: Some people were assigned to multiple groups!");
    }

    // Update history with new assignments
    history.update(&assignments);

    // Save updated history
    if let Err(e) = history.save_to_file(HISTORY_FILE) {
        eprintln!("Warning: Could not save assignment history: {}", e);
    }

    // Print the assignments
    print_assignments(assignments);
}

