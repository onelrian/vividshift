use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};

pub fn distribute_work(
    names_a: Vec<String>,
    names_b: Vec<String>,
    prev_assignments: &mut HashMap<&'static str, HashSet<String>>,
) -> HashMap<&'static str, Vec<String>> {
    let work_assignments = [
        ("Parlor", 5),    // Increased by 1
        ("Frontyard", 3), // Reduced to 3
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
    let mut shuffled_a = names_a.clone();
    let mut shuffled_b = names_b.clone();
    shuffled_a.shuffle(&mut rng);
    shuffled_b.shuffle(&mut rng);

    let toilet_a_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet A").unwrap().1;
    assignments.insert("Toilet A", shuffled_a.drain(..toilet_a_count.min(shuffled_a.len())).collect());

    let toilet_b_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet B").unwrap().1;
    assignments.insert("Toilet B", shuffled_b.drain(..toilet_b_count.min(shuffled_b.len())).collect());

    let mut remaining_people = Vec::new();
    remaining_people.extend(shuffled_a);
    remaining_people.extend(shuffled_b);
    remaining_people.shuffle(&mut rng);

    let mut index = 0;
    for &(domain, count) in &work_assignments {
        if domain == "Toilet A" || domain == "Toilet B" {
            continue;
        }

        for _ in 0..count {
            if index < remaining_people.len() {
                let person = remaining_people[index].clone();
                
                if let Some(prev_group) = prev_assignments.get(domain) {
                    if prev_group.contains(&person) {
                        continue;
                    }
                }
                
                assignments.get_mut(domain).unwrap().push(person.clone());
                prev_assignments.entry(domain).or_insert(HashSet::new()).insert(person);
                index += 1;
            }
        }
    }

    assignments
}
