// Declare the modules.
mod files;
mod group;
mod output;

use std::process;

fn main() {
    // --- Use eprintln! for all log messages ---
    eprintln!("--- Starting Work Distribution ---");

    // 1. Read the lists of names.
    let mut names_a = files::read_names_from_file("file_a.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_a.txt: {}", err);
        process::exit(1);
    });

    let mut names_b = files::read_names_from_file("file_b.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_b.txt: {}", err);
        process::exit(1);
    });
    
    let mut all_names = Vec::new();
    all_names.append(&mut names_a);
    all_names.append(&mut names_b);

    eprintln!("✅ Successfully read {} names.", all_names.len());

    // 2. Read the assignment history.
    let history = files::read_assignment_history("assignment_history.txt").unwrap_or_else(|err| {
        eprintln!("Warning: Could not read assignment_history.txt: {}. Assuming no history.", err);
        Default::default()
    });

    eprintln!("✅ Read {} previous assignments from history.", history.len());

    // 3. Generate the new, non-consecutive work assignments.
    let new_assignments = group::distribute_work(&mut all_names, &history);
    eprintln!("✅ Generated new work assignments.");

    // 4. Format the results into a single, clean string.
    let final_output = output::format_assignments(&new_assignments);
    
    // 5. Print the final, clean output to stdout. This is what the workflow will capture.
    print!("{}", final_output);

    // 6. Write the new assignments to the history file for the next run.
    if let Err(e) = files::write_assignment_history("assignment_history.txt", &new_assignments) {
        eprintln!("Error writing to assignment_history.txt: {}", e);
        process::exit(1);
    }
    
    eprintln!("\n✅ Successfully updated assignment_history.txt for the next cycle.");
    eprintln!("--- Work Distribution Complete ---");
}

  

