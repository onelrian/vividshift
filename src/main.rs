// Declare the modules to match your file names.
mod files;
mod group;
mod output;

use std::process;

fn main() {
    println!("--- Starting Work Distribution ---");

    // 1. Read the lists of names.
    let mut names_a = files::read_names_from_file("file_a.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_a.txt: {}", err);
        process::exit(1);
    });

    let mut names_b = files::read_names_from_file("file_b.txt").unwrap_or_else(|err| {
        eprintln!("Error reading file_b.txt: {}", err);
        process::exit(1);
    });
    
    // Combine all names into a single list.
    let mut all_names = Vec::new();
    all_names.append(&mut names_a);
    all_names.append(&mut names_b);

    println!("✅ Successfully read {} names.", all_names.len());

    // 2. Read the assignment history.
    let history = files::read_assignment_history("assignment_history.txt").unwrap_or_else(|err| {
        eprintln!("Warning: Could not read assignment_history.txt: {}. Assuming no history.", err);
        // Return an empty HashMap if there's an error, so the program can continue.
        Default::default()
    });

    println!("✅ Read {} previous assignments from history.", history.len());

    // 3. Generate the new, non-consecutive work assignments.
    // Call the function from the `group` module.
    let new_assignments = group::distribute_work(&mut all_names, &history);
    println!("✅ Generated new work assignments.");

    // 4. Print the results to the console.
    output::print_assignments(&new_assignments);

    // 5. Write the new assignments to the history file for the next run.
    if let Err(e) = files::write_assignment_history("assignment_history.txt", &new_assignments) {
        eprintln!("Error writing to assignment_history.txt: {}", e);
        process::exit(1);
    }
    
    println!("\n✅ Successfully updated assignment_history.txt for the next cycle.");
    println!("--- Work Distribution Complete ---");
}