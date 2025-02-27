use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

pub fn distribute_work(names_a: Vec<String>, names_b: Vec<String>) -> HashMap<&'static str, Vec<String>> {
    let work_assignments = [
        ("Parlor", 4),
        ("Frontyard", 4),
        ("Backyard", 1),
        ("Tank", 2),
        ("Toilet B", 4),  // Only from file B
        ("Toilet A", 2),  // Only from file A
        ("Bin", 1),
    ];

    let mut assignments: HashMap<&str, Vec<String>> = HashMap::new();
    for &(domain, _) in &work_assignments {
        assignments.insert(domain, Vec::new());
    }

    let mut rng = thread_rng();

    // Shuffle names randomly
    let mut shuffled_a = names_a.clone();
    let mut shuffled_b = names_b.clone();
    shuffled_a.shuffle(&mut rng);
    shuffled_b.shuffle(&mut rng);

    // Assign people from file A strictly to Toilet A
    let toilet_a_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet A").unwrap().1;
    assignments.insert("Toilet A", shuffled_a.drain(..toilet_a_count.min(shuffled_a.len())).collect());

    // Assign people from file B strictly to Toilet B
    let toilet_b_count = work_assignments.iter().find(|&&(d, _)| d == "Toilet B").unwrap().1;
    assignments.insert("Toilet B", shuffled_b.drain(..toilet_b_count.min(shuffled_b.len())).collect());

    // Combine the remaining people from both groups
    let mut remaining_people = Vec::new();
    remaining_people.extend(shuffled_a);
    remaining_people.extend(shuffled_b);
    remaining_people.shuffle(&mut rng);

    let mut index = 0;
    for &(domain, count) in &work_assignments {
        if domain == "Toilet A" || domain == "Toilet B" {
            continue; // These are already assigned
        }

        for _ in 0..count {
            if index < remaining_people.len() {
                assignments.get_mut(domain).unwrap().push(remaining_people[index].clone());
                index += 1;
            }
        }
    }

    assignments
}
