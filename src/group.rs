use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

/// Defines the work assignments with the task name and the number of people required.
fn get_work_assignments() -> Vec<(&'static str, usize)> {
    vec![
        ("Toilet A", 2),
        ("Toilet B", 4),
        ("Parlor", 5),
        ("Frontyard", 3),
        ("Backyard", 1),
        ("Tank", 2),
        ("Bin", 1),
    ]
}

/// Distributes people into work groups, ensuring no one is assigned the same task
/// they had in the previous cycle (based on the provided history).
pub fn distribute_work(
    people: &mut Vec<String>,
    history: &HashMap<String, String>,
) -> HashMap<String, Vec<String>> {
    let mut assignments: HashMap<String, Vec<String>> = HashMap::new();
    let work_definitions = get_work_assignments();

    // Shuffle the list of people for randomness.
    people.shuffle(&mut thread_rng());

    let mut unassigned_people = people.clone();

    // --- FIX #1: Iterate by reference (&) so `work_definitions` is not moved ---
    for &(task, num_required) in &work_definitions {
        let mut assigned_to_task: Vec<String> = Vec::new();
        let mut remaining_people: Vec<String> = Vec::new();
        
        // Iterate through the available people to find suitable candidates for the current task.
        for person in unassigned_people.into_iter() {
            // Check if we have enough people for this task already.
            if assigned_to_task.len() >= num_required {
                remaining_people.push(person);
                continue;
            }

            // Check the person's history.
            let last_task = history.get(&person);
            
            // If the person's last task is the same as the current one, they are not eligible.
            if last_task.is_some() && last_task.unwrap() == task {
                remaining_people.push(person); // Put them back in the pool for the next tasks.
            } else {
                assigned_to_task.push(person); // Assign them!
            }
        }
        
        // Update the pool of unassigned people for the next iteration.
        unassigned_people = remaining_people;
        assignments.insert(task.to_string(), assigned_to_task);
    }
    
    // Assign any remaining people (if any) to tasks that are still under-staffed
    if !unassigned_people.is_empty() {
        // --- FIX #2: Also iterate by reference here ---
        for &(task, num_required) in &work_definitions {
            if let Some(assigned) = assignments.get_mut(task) {
                while assigned.len() < num_required && !unassigned_people.is_empty() {
                     assigned.push(unassigned_people.pop().unwrap());
                }
            }
        }
    }

    assignments
}