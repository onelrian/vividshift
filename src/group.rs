use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

// Structure to track previous assignments
#[derive(Debug)]
pub struct AssignmentHistory {
    pub history: HashMap<String, String>, // Person -> Last assignment
}

impl AssignmentHistory {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    // Load previous assignment history from file
    pub fn load_from_file(file_path: &str) -> io::Result<Self> {
        let path = Path::new(file_path);
        let mut history = HashMap::new();

        // If the file exists, read it
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let person = parts[0].trim().to_string();
                    let assignment = parts[1].trim().to_string();
                    history.insert(person, assignment);
                }
            }
        }

        Ok(Self { history })
    }

    // Save current assignment history to file
    pub fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;

        for (person, assignment) in &self.history {
            writeln!(file, "{}:{}", person, assignment)?;
        }

        Ok(())
    }

    // Update history with new assignments
    pub fn update(&mut self, assignments: &HashMap<&str, Vec<String>>) {
        for (domain, people) in assignments {
            for person in people {
                self.history.insert(person.clone(), domain.to_string());
            }
        }
    }

    // Check if a person was previously assigned to a domain
    pub fn was_previously_assigned(&self, person: &str, domain: &str) -> bool {
        if let Some(prev_domain) = self.history.get(person) {
            return prev_domain == domain;
        }
        false
    }
}

pub fn distribute_work(
    names_a: Vec<String>,
    names_b: Vec<String>,
    history: &AssignmentHistory,
) -> HashMap<&'static str, Vec<String>> {
    let work_assignments = [
        ("Parlor", 5),    
        ("Frontyard", 3), 
        ("Backyard", 1),
        ("Tank", 2),
        ("Toilet B", 4),
        ("Toilet A", 2),
        ("Bin", 1),
    ];

    let mut assignments: HashMap<&str, Vec<String>> = HashMap::new();
    for &(domain, _) in &work_assignments {
        assignments.insert(domain, Vec::new());
    }

    let mut rng = thread_rng();
    
    // Create a copy of names to work with
    let mut available_a = names_a.clone();
    let mut available_b = names_b.clone();
    
    // Shuffle the names
    available_a.shuffle(&mut rng);
    available_b.shuffle(&mut rng);
    
    // First, handle Toilet A (only from file_a.txt)
    let toilet_a_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet A").unwrap().1;
    
    // Sort available_a to prioritize people who weren't in Toilet A last time
    available_a.sort_by(|a, b| {
        let a_was_in_toilet_a = history.was_previously_assigned(a, "Toilet A");
        let b_was_in_toilet_a = history.was_previously_assigned(b, "Toilet A");
        a_was_in_toilet_a.cmp(&b_was_in_toilet_a)
    });
    
    // Take people for Toilet A
    let toilet_a_people: Vec<String> = available_a.drain(..toilet_a_count.min(available_a.len())).collect();
    assignments.insert("Toilet A", toilet_a_people);
    
    // Next, handle Toilet B (only from file_b.txt)
    let toilet_b_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet B").unwrap().1;
    
    // Sort available_b to prioritize people who weren't in Toilet B last time
    available_b.sort_by(|a, b| {
        let a_was_in_toilet_b = history.was_previously_assigned(a, "Toilet B");
        let b_was_in_toilet_b = history.was_previously_assigned(b, "Toilet B");
        a_was_in_toilet_b.cmp(&b_was_in_toilet_b)
    });
    
    // Take people for Toilet B
    let toilet_b_people: Vec<String> = available_b.drain(..toilet_b_count.min(available_b.len())).collect();
    assignments.insert("Toilet B", toilet_b_people);
    
    // Combine remaining people from both files
    let mut remaining_people = Vec::new();
    remaining_people.extend(available_a);
    remaining_people.extend(available_b);
    
    // For each remaining work assignment
    for &(domain, count) in &work_assignments {
        if domain == "Toilet A" || domain == "Toilet B" {
            continue; // Already handled
        }
        
        // Sort remaining people to prioritize those who weren't in this domain last time
        remaining_people.sort_by(|a, b| {
            let a_was_in_domain = history.was_previously_assigned(a, domain);
            let b_was_in_domain = history.was_previously_assigned(b, domain);
            a_was_in_domain.cmp(&b_was_in_domain)
        });
        
        // Take required number of people for this domain
        let mut domain_people = Vec::new();
        for _ in 0..count {
            if remaining_people.is_empty() {
                break;
            }
            domain_people.push(remaining_people.remove(0));
        }
        
        assignments.insert(domain, domain_people);
    }
    
    // Check if any domain has fewer people than required
    // If so, try to redistribute from domains that have more than required
    let mut domains_with_excess = Vec::new();
    let mut domains_with_shortage = Vec::new();
    
    for &(domain, required) in &work_assignments {
        let current = assignments.get(domain).unwrap().len();
        if current < required {
            domains_with_shortage.push((domain, required - current));
        } else if current > required {
            domains_with_excess.push((domain, current - required));
        }
    }
    
    // If we have both excess and shortage, try to balance
    if !domains_with_excess.is_empty() && !domains_with_shortage.is_empty() {
        for (shortage_domain, shortage) in domains_with_shortage {
            for (excess_domain, excess) in &domains_with_excess {
                if *excess == 0 {
                    continue;
                }
                
                let to_move = shortage.min(*excess);
                if to_move > 0 {
                    let mut excess_people = assignments.get_mut(excess_domain).unwrap();
                    let mut people_to_move = Vec::new();
                    
                    // Prioritize moving people who were previously in the shortage domain
                    excess_people.sort_by(|a, b| {
                        let a_was_in_shortage = history.was_previously_assigned(a, shortage_domain);
                        let b_was_in_shortage = history.was_previously_assigned(b, shortage_domain);
                        b_was_in_shortage.cmp(&a_was_in_shortage) // Reverse order to prioritize those NOT in shortage domain
                    });
                    
                    for _ in 0..to_move {
                        if !excess_people.is_empty() {
                            people_to_move.push(excess_people.remove(0));
                        }
                    }
                    
                    assignments.get_mut(shortage_domain).unwrap().extend(people_to_move);
                }
            }
        }
    }
    
    assignments
}

