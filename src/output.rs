use std::collections::HashMap;

pub fn format_assignments(assignments: &HashMap<String, Vec<String>>) -> String {
    let mut result = String::new();
    
    result.push_str("\n**📊 Work Distribution Results**\n\n");
    
    // Create a sorted list of tasks for consistent output order.
    let mut sorted_tasks: Vec<_> = assignments.keys().collect();
    sorted_tasks.sort();

    for task in sorted_tasks {
        if let Some(people) = assignments.get(task) {
            if !people.is_empty() {
                // Append each formatted line to our result string
                result.push_str(&format!("**{}**: {}\n", task, people.join(", ")));
            }
        }
    }
    
    result // Return the final, formatted string
}