use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn read_names_from_file(filename: &str) -> io::Result<Vec<String>> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let names: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    Ok(names)
}

