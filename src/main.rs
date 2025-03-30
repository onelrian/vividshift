mod group;
mod files;
mod output;

use group::distribute_work;
use files::read_names_from_file;
use output::print_assignments;
use std::collections::HashMap;
use std::collections::HashSet;

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

    let mut prev_assignments: HashMap<&str, HashSet<String>> = HashMap::new();
    let assignments = distribute_work(names_a, names_b, &mut prev_assignments);

    print_assignments(assignments);
}
