use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

pub fn distribute_work(names_a: Vec<String>, names_b: Vec<String>) -> HashMap<&'static str, Vec<String>> {
    let work_ratios = [
        ("Parlor", 4),
        ("Toilet A", 2),
        ("Toilet B", 4),
        ("Front", 4),
        ("Backyard", 1),
        ("Tanks", 2),
        ("Bin", 1),
    ];

    let total_ratio: usize = work_ratios.iter().map(|&(_, r)| r).sum();
    let num_people = names_a.len() + names_b.len();

    let mut assignments: HashMap<&str, Vec<String>> = HashMap::new();
    for &(domain, _) in &work_ratios {
        assignments.insert(domain, Vec::new());
    }

    // Shuffle the names randomly
    let mut rng = thread_rng();
    let mut remaining_names = names_a.clone();
    remaining_names.extend(names_b.clone());
    remaining_names.shuffle(&mut rng);

    // Assign people from file A to Toilet A
    for person in names_a {
        assignments.get_mut("Toilet A").unwrap().push(person);
    }

    // Assign people from file B to Toilet B
    for person in names_b {
        assignments.get_mut("Toilet B").unwrap().push(person);
    }

    // Now assign remaining people to other domains
    let mut index = 0;

    // Assign remaining people to other domains based on the ratio
    for &(domain, ratio) in &work_ratios {
        if domain == "Toilet A" || domain == "Toilet B" {
            continue; 
        }

        let count = (ratio * num_people) / total_ratio;

        for _ in 0..count {
            if index < remaining_names.len() {
                assignments
                    .get_mut(domain)
                    .unwrap()
                    .push(remaining_names[index].clone());
                index += 1;
            }
        }
    }

    // Assign any remaining people to the least populated domain
    let mut remaining = remaining_names.into_iter().skip(index).collect::<Vec<_>>();
    while !remaining.is_empty() {
        // Find the least populated domain without holding a mutable reference
        let least_populated = assignments
            .iter()
            .min_by_key(|(_, v)| v.len())
            .map(|(k, _)| *k)
            .unwrap();

        // Modify the least populated domain
        assignments
            .get_mut(least_populated)
            .unwrap()
            .push(remaining.remove(0));
    }

    assignments
}