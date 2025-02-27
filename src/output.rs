use std::collections::HashMap;

pub fn print_assignments(assignments: HashMap<&str, Vec<String>>) {
    println!("**📊 Work Distribution Results**\n");
    for (domain, names) in assignments {
        println!("**{}**: {}", domain, names.join(", "));
    }
}
