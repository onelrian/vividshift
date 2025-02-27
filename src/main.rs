mod files;
mod group;

use std::io;

fn main() -> io::Result<()> {
    let filename = "names.txt";
    let names = files::read_names_from_file(filename)?;

    // Distribute work
    let assignments = group::distribute_work(names);

    for (domain, people) in assignments {
        println!("{}: {:?}", domain, people);
    }

    Ok(())
}
