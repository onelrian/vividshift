// src/group.rs

use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

/// Generates new work assignments using a hybrid rotation strategy to satisfy all constraints.
pub fn distribute_work(
    names_a: &[String],
    names_b: &[String],
    work_areas: &HashMap<String, usize>,
    history: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, Vec<String>>, String> {
    
    let all_people: HashSet<String> = names_a.iter().chain(names_b.iter()).cloned().collect();
    let names_a_set: HashSet<_> = names_a.iter().cloned().collect();
    let names_b_set: HashSet<_> = names_b.iter().cloned().collect();

    let mut assignments: HashMap<String, Vec<String>> = HashMap::new();
    for area in work_areas.keys() {
        assignments.insert(area.clone(), Vec::new());
    }

    // Step 1: Pre-calculate all possible candidates for every task
    let mut candidates: HashMap<String, HashSet<String>> = HashMap::new();
    for (area, _) in work_areas {
        let mut area_candidates = HashSet::new();
        for person in &all_people {
            let person_history = history.get(person).map_or(Vec::new(), |h| h.clone());

            // --- HYBRID ELIGIBILITY CHECK ---
            let has_worked_here_recently = if *area == "Toilet B" {
                // For the highly constrained Toilet B, only check the single most recent assignment.
                person_history.first().map_or(false, |last_area| last_area == area)
            } else {
                // For all other tasks, use the standard long-term history check.
                person_history.contains(area)
            };
            
            // Reinstate the original strict rules.
            let is_from_b_in_toilet_a = *area == "Toilet A" && names_b_set.contains(person);
            let is_from_a_in_toilet_b = *area == "Toilet B" && names_a_set.contains(person);

            // A person is eligible if they meet all conditions.
            if !has_worked_here_recently && !is_from_b_in_toilet_a && !is_from_a_in_toilet_b {
                area_candidates.insert(person.clone());
            }
        }
        candidates.insert(area.clone(), area_candidates);
    }

    // The rest of the algorithm (the constraint solver) remains the same.
    let total_spots_to_fill: usize = work_areas.values().sum();
    for _ in 0..total_spots_to_fill {
        let most_constrained_task = candidates.iter()
            .filter(|(area, _)| assignments[area.as_str()].len() < work_areas[area.as_str()])
            .min_by_key(|(_, potential_assignees)| potential_assignees.len());

        if let Some((task_name, potential_assignees)) = most_constrained_task {
            if potential_assignees.is_empty() {
                return Err(format!(
                    "could not find a valid assignment. Task '{}' needs {} more person/people, but has no eligible candidates left.",
                    task_name, work_areas[task_name] - assignments[task_name].len()
                ));
            }

            let assignees_vec: Vec<_> = potential_assignees.iter().collect();
            let person_to_assign = (*assignees_vec.choose(&mut rand::thread_rng()).unwrap()).clone();
            assignments.get_mut(task_name).unwrap().push(person_to_assign.clone());

            for an_area in candidates.values_mut() {
                an_area.remove(&person_to_assign);
            }

        } else {
            break; 
        }
    }

    Ok(assignments)
}