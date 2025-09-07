use std::collections::HashMap;

pub fn print_assignments(assignments: &HashMap<String, Vec<String>>) {
    println!("\n**📊 Work Distribution Results**\n");
    
    // Create a sorted list of tasks for consistent output order.
    let mut sorted_tasks: Vec<_> = assignments.keys().collect();
    sorted_tasks.sort();

    for task in sorted_tasks {
        if let Some(people) = assignments.get(task) {
            if !people.is_empty() {
                println!("**{}**: {}", task, people.join(", "));
            }
        }
    }
}