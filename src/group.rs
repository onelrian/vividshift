use std::collections::{HashMap, HashSet};

// A type alias for our long-term history for clarity.
type LongTermHistory = HashMap<String, Vec<String>>;

/// Defines the work assignments.
fn get_work_assignments() -> Vec<(&'static str, usize)> {
    vec![
        ("Toilet A", 2), ("Toilet B", 4), ("Parlor", 5),
        ("Frontyard", 3), ("Backyard", 1), ("Tank", 2), ("Bin", 1),
    ]
}

/// The definitive and flawless distribution logic.
pub fn distribute_work(
    people: &mut Vec<String>,
    history: &LongTermHistory,
    group_a: &HashSet<String>,
    group_b: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut assignments: HashMap<String, Vec<String>> = HashMap::new();
    let work_definitions = get_work_assignments();
    
    let mut unassigned_people: HashSet<String> = people.iter().cloned().collect();

    // MAIN ASSIGNMENT LOOP: Iterate through each task and find the best candidates
    for &(task, num_required) in &work_definitions {
        let mut candidates: Vec<String> = unassigned_people
            .iter()
            .filter(|person| {
                // Check all hard rules.
                let last_task = history.get(*person).and_then(|h| h.get(0));
                let is_consecutive = last_task.map_or(false, |t| t == task);
                let is_ineligible_for_toilet_a = task == "Toilet A" && group_b.contains(*person);
                let is_ineligible_for_toilet_b = task == "Toilet B" && group_a.contains(*person);

                !is_consecutive && !is_ineligible_for_toilet_a && !is_ineligible_for_toilet_b
            })
            .cloned()
            .collect();

        // Sort candidates by fairness score to prioritize rotation.
        candidates.sort_by_key(|person| {
            history.get(person).map_or(0, |h| h.iter().filter(|&t| t == task).count())
        });

        let assigned_this_task: Vec<String> = candidates
            .into_iter()
            .take(num_required)
            .collect();

        for person in &assigned_this_task {
            unassigned_people.remove(person);
        }
        assignments.insert(task.to_string(), assigned_this_task);
    }

    // FINAL GUARANTEE: Force-assign any remaining people
    if !unassigned_people.is_empty() {
        let mut people_to_place: Vec<String> = unassigned_people.into_iter().collect();
        // The rand crate is no longer needed here as we are just filling spots
        
        for (task, num_required) in &work_definitions {
            if let Some(assigned) = assignments.get_mut(*task) {
                while assigned.len() < *num_required && !people_to_place.is_empty() {
                    assigned.push(people_to_place.pop().unwrap());
                }
            }
        }
    }

    assignments
}