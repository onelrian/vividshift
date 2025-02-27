use std::{fs::File, io::{self, BufRead}};

pub fn read_names_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let names: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    Ok(names)
}