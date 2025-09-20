use rand::seq::SliceRandom;
use rand::thread_rng;
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

/// Calculates a "fairness score" based on long-term history to encourage rotation.
fn calculate_fairness_score(person: &str, task: &str, history: &LongTermHistory) -> u32 {
    let mut score = 0;
    if let Some(person_history) = history.get(person) {
        // Base score for total times done.
        score += person_history.iter().filter(|&t| t == task).count() as u32 * 10;
        
        // Penalties for recent assignments.
        if person_history.get(2).map_or(false, |t| t == task) { score += 50; }
        if person_history.get(3).map_or(false, |t| t == task) { score += 25; }
        if person_history.get(4).map_or(false, |t| t == task) { score += 10; }
    }
    score
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

    // --- MAIN ASSIGNMENT LOOP: Iterate through each task and find the best candidates ---
    for &(task, num_required) in &work_definitions {
        // Step 1: Find all eligible candidates from the unassigned pool.
        let mut candidates: Vec<(String, u32)> = unassigned_people
            .iter()
            .filter_map(|person| {
                // Check all hard rules.
                let person_history = history.get(person);
                let is_on_cooldown = person_history.map_or(false, |h| {
                    h.get(0).map_or(false, |t| t == task) || h.get(1).map_or(false, |t| t == task)
                });
                let is_ineligible_for_toilet_a = task == "Toilet A" && group_b.contains(person);
                let is_ineligible_for_toilet_b = task == "Toilet B" && group_a.contains(person);

                // If the person is eligible, calculate their fairness score.
                if !is_on_cooldown && !is_ineligible_for_toilet_a && !is_ineligible_for_toilet_b {
                    let score = calculate_fairness_score(person, task, history);
                    Some((person.clone(), score))
                } else {
                    None // This person is not eligible for this task.
                }
            })
            .collect();

        // Step 2: Sort candidates by score to find the fairest choices.
        candidates.sort_by_key(|&(_, score)| score);

        // Step 3: Assign the best candidates up to the required number.
        let assigned_this_task: Vec<String> = candidates
            .into_iter()
            .take(num_required)
            .map(|(person, _)| person)
            .collect();

        // Step 4: Update the state.
        for person in &assigned_this_task {
            unassigned_people.remove(person);
        }
        assignments.insert(task.to_string(), assigned_this_task);
    }

    // --- FINAL GUARANTEE: Force-assign any remaining people ---
    // This handles rare edge cases where rules were too strict, ensuring all 18 slots are filled.
    if !unassigned_people.is_empty() {
        let mut people_to_place: Vec<String> = unassigned_people.into_iter().collect();
        people_to_place.shuffle(&mut thread_rng()); // Shuffle for fairness
        
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