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

/// Calculates a "fairness score" based on how many times a person has done a task.
fn calculate_fairness_score(person: &str, task: &str, history: &LongTermHistory) -> usize {
    history
        .get(person)
        .map_or(0, |h| h.iter().filter(|&t| t == task).count())
}

/// The final, robust distribution logic using a prioritized, multi-phase assignment system.
pub fn distribute_work(
    people: &mut Vec<String>,
    history: &LongTermHistory,
    group_a: &HashSet<String>,
    group_b: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut assignments: HashMap<String, Vec<String>> = HashMap::new();
    let work_definitions = get_work_assignments();
    
    let mut unassigned_people: HashSet<String> = people.iter().cloned().collect();

    // --- PHASE 1: Assign the most restrictive tasks first ---
    for &(task, num_required) in &[("Toilet A", 2), ("Toilet B", 4)] {
        let relevant_pool = if task == "Toilet A" { group_a } else { group_b };
        
        let mut candidates: Vec<(String, usize)> = unassigned_people
            .iter()
            .filter(|&p| relevant_pool.contains(p))
            .filter_map(|p| {
                let last_task = history.get(p).and_then(|h| h.get(0));
                let is_consecutive = last_task.map_or(false, |t| t == task);
                if !is_consecutive {
                    let score = calculate_fairness_score(p, task, history);
                    Some((p.clone(), score))
                } else {
                    None
                }
            })
            .collect();
        
        candidates.sort_by_key(|&(_, score)| score);

        let assigned_this_task: Vec<String> = candidates
            .into_iter()
            .take(num_required)
            .map(|(person, _)| person)
            .collect();

        for person in &assigned_this_task {
            unassigned_people.remove(person);
        }
        assignments.insert(task.to_string(), assigned_this_task);
    }

    // --- PHASE 2: Assign all remaining general tasks ---
    for &(task, num_required) in &work_definitions {
        if task == "Toilet A" || task == "Toilet B" { continue; }

        let mut candidates: Vec<(String, usize)> = unassigned_people
            .iter()
            .filter_map(|p| {
                let last_task = history.get(p).and_then(|h| h.get(0));
                let is_consecutive = last_task.map_or(false, |t| t == task);
                if !is_consecutive {
                    let score = calculate_fairness_score(p, task, history);
                    Some((p.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|&(_, score)| score);
        
        let assigned_this_task: Vec<String> = candidates
            .into_iter()
            .take(num_required)
            .map(|(person, _)| person)
            .collect();

        for person in &assigned_this_task {
            unassigned_people.remove(person);
        }
        assignments.insert(task.to_string(), assigned_this_task);
    }

    // --- FINAL GUARANTEE ---
    if !unassigned_people.is_empty() {
        let mut people_to_place: Vec<String> = unassigned_people.into_iter().collect();
        people_to_place.shuffle(&mut thread_rng()); 
        
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