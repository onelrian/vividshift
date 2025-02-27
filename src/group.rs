use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

pub fn distribute_work(names_a: Vec<String>, names_b: Vec<String>) -> HashMap<&'static str, Vec<String>> {
    let work_ratios = [
        ("Parlor", 4),
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
    assignments.insert("Toilet A", Vec::new());
    assignments.insert("Toilet B", Vec::new());

    // Shuffle the remaining names randomly
    let mut rng = thread_rng();
    let mut shuffled_a = names_a.clone();
    let mut shuffled_b = names_b.clone();
    shuffled_a.shuffle(&mut rng);
    shuffled_b.shuffle(&mut rng);

    // Assign people from file A strictly to Toilet A
    for person in &names_a {
        assignments.get_mut("Toilet A").unwrap().push(person.clone());
    }

    // Assign people from file B strictly to Toilet B
    for person in &names_b {
        assignments.get_mut("Toilet B").unwrap().push(person.clone());
    }

    // Remove assigned people from the remaining pool
    let remaining_people: Vec<String> = shuffled_a
        .into_iter()
        .chain(shuffled_b.into_iter())
        .filter(|p| !assignments["Toilet A"].contains(p) && !assignments["Toilet B"].contains(p))
        .collect();

    let mut index = 0;

    // Assign remaining people to other domains based on the ratio
    for &(domain, ratio) in &work_ratios {
        let count = (ratio * num_people) / total_ratio;
        for _ in 0..count {
            if index < remaining_people.len() {
                assignments
                    .get_mut(domain)
                    .unwrap()
                    .push(remaining_people[index].clone());
                index += 1;
            }
        }
    }

    // Assign any remaining people to the least populated domain
    let mut leftover = remaining_people.into_iter().skip(index).collect::<Vec<_>>();
    while !leftover.is_empty() {
        let least_populated = assignments
            .iter()
            .filter(|(k, _)| *k != &"Toilet A" && *k != &"Toilet B")
            .min_by_key(|(_, v)| v.len())
            .map(|(k, _)| *k)
            .unwrap();

        assignments
            .get_mut(least_populated)
            .unwrap()
            .push(leftover.remove(0));
    }

    assignments
}
