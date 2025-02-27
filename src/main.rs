mod group;
mod files;
mod output;

use group::distribute_work;
use files::read_names_from_file;
use output::print_assignments;

fn main() {
    // Read names from file A and file B
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

    // Distribute the work among the people
    let assignments = distribute_work(names_a, names_b);

    // Print the assignments in the desired format
    print_assignments(assignments);
}
