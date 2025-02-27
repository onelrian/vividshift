mod files;
mod group;

use std::io;

fn main() -> io::Result<()> {
    // Read names from a file
    let filename = "names.txt"; // Replace with your file path
    let names = files::read_names_from_file(filename)?;

    // Distribute work
    let assignments = group::distribute_work(names);

    // Print the assignments
    for (domain, people) in assignments {
        println!("{}: {:?}", domain, people);
    }

    Ok(())
}