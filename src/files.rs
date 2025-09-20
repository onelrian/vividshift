use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter};

// Define the structure for our long-term history data.
pub type LongTermHistory = HashMap<String, Vec<String>>;

/// Reads and parses the long-term history from the JSON file.
pub fn read_long_term_history(path: &str) -> io::Result<LongTermHistory> {
    // --- THIS BLOCK IS NOW SYNTACTICALLY CORRECT ---
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(HashMap::new());
        }
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);
    let history = serde_json::from_reader(reader)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    Ok(history)
}

/// Writes the updated long-term history to the JSON file.
pub fn write_long_term_history(path: &str, history: &LongTermHistory) -> io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, history)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

/// Reads a list of names from a given file.
pub fn read_names_from_file(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let mut names = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if !line.trim().is_empty() {
            names.push(line.trim().to_string());
        }
    }
    Ok(names)
}