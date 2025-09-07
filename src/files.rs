use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

/// Reads a list of names from a given file, one name per line.
pub fn read_names_from_file(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut names = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            names.push(line.trim().to_string());
        }
    }
    Ok(names)
}

/// Reads the assignment history file ("Name:Task").
/// Returns a HashMap of { name -> last_task }.
/// If the file doesn't exist, it returns an empty HashMap, which is normal for a first run.
pub fn read_assignment_history(path: &str) -> io::Result<HashMap<String, String>> {
    let mut history = HashMap::new();
    
    // Attempt to open the file, but if it doesn't exist, just return the empty history.
    let file = match File::open(path) {
        Ok(file) => file,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(history); 
        }
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Some((name, task)) = line.split_once(':') {
            history.insert(name.trim().to_string(), task.trim().to_string());
        }
    }

    Ok(history)
}

/// Writes the newly generated assignments to the history file, overwriting the old one.
pub fn write_assignment_history(
    path: &str,
    assignments: &HashMap<String, Vec<String>>,
) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true) // Overwrite the file if it exists
        .open(path)?;

    let mut history_entries = Vec::new();
    
    // Flatten the assignments map into a list of "Name:Task" strings.
    for (task, people) in assignments {
        for person in people {
            history_entries.push(format!("{}:{}\n", person, task));
        }
    }
    
    // Sort for consistent output, which is nice for tracking changes in git.
    history_entries.sort();

    for entry in history_entries {
        file.write_all(entry.as_bytes())?;
    }

    Ok(())
}