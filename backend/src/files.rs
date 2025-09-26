// src/files.rs

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Reads names from a text file, one per line.
/// Skips empty lines and trims whitespace.
pub fn read_names_from_file<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut names = Vec::new();
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if !line.is_empty() {
            names.push(line);
        }
    }
    Ok(names)
}