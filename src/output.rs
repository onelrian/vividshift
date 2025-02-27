use std::collections::HashMap;

pub fn print_assignments(assignments: HashMap<&str, Vec<String>>) {
    for (domain, people) in assignments {
        println!("**{}**", domain);
        for person in people {
            println!("  - {}", person);
        }
        println!();
    }
}