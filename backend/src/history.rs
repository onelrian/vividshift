// src/history.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};

const HISTORY_LENGTH: usize = 2;

// This struct defines the proper, expected format of the JSON file.
#[derive(Serialize, Deserialize, Clone)]
struct HistoryData {
    assignments: HashMap<String, Vec<String>>,
}

pub type History = HashMap<String, Vec<String>>;

/// Loads the assignment history from `assignment_history.json`.
/// This version is robust and can handle both the new and old formats.
pub fn load_history() -> io::Result<History> {
    let file = match File::open("assignment_history.json") {
        Ok(file) => file,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(History::new()),
        Err(e) => return Err(e),
    };

    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    // First, try to parse into the new struct format.
    if let Ok(data) = serde_json::from_str::<HistoryData>(&buffer) {
        return Ok(data.assignments);
    }
    
    // If that fails, try to parse it as the old format (just a direct HashMap).
    // This prevents the app from crashing if it finds an old history file.
    if let Ok(old_data) = serde_json::from_str::<History>(&buffer) {
        println!("⚠️ Found old history format. It will be converted on the next save.");
        return Ok(old_data);
    }

    // If both fail, the file is likely corrupt.
    Err(io::Error::new(io::ErrorKind::InvalidData, "Could not parse assignment_history.json"))
}

/// Updates and saves the new assignments to `assignment_history.json`.
pub fn save_history(assignments: &HashMap<String, Vec<String>>, old_history: &History) -> io::Result<()> {
    let mut new_history = old_history.clone();

    for (area, people) in assignments {
        for person in people {
            let person_history = new_history.entry(person.clone()).or_default();
            person_history.insert(0, area.clone());
            person_history.truncate(HISTORY_LENGTH);
        }
    }

    let data_to_save = HistoryData { assignments: new_history };

    let file = std::fs::File::create("assignment_history.json")?;
    
    serde_json::to_writer_pretty(file, &data_to_save)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}