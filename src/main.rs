use output::print_assignments;

mod group;
mod files;
mod output;


fn main() {
    // Read names from file A and file B
    let names_a = match files::read_names_from_file("file_a.txt") {
        Ok(names) => names,
        Err(e) => {
            eprintln!("Error reading file_a.txt: {}", e);
            return;
        }
    };
    
    let names_b = match files::read_names_from_file("file_b.txt") {
        Ok(names) => names,
        Err(e) => {
            eprintln!("Error reading file_b.txt: {}", e);
            return;
        }
    };

    // Distribute the work among the people
    let assignments = group::distribute_work(names_a, names_b);

    // Print the assignments
    print_assignments(assignments);
}

