use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn distribute_work(names: Vec<String>) -> HashMap<&'static str, Vec<String>> {
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
    let num_people = names.len();

    let mut assignments: HashMap<&str, Vec<String>> = HashMap::new();
    for &(domain, _) in &work_ratios {
        assignments.insert(domain, Vec::new());
    }

    // Shuffle the names randomly
    let mut rng = thread_rng();
    let mut shuffled_names = names;
    shuffled_names.shuffle(&mut rng);

    let mut index = 0;
    
    // Assign people based on ratio
    for &(domain, ratio) in &work_ratios {
        let count = (ratio * num_people) / total_ratio;

        for _ in 0..count {
            if index < num_people {
                assignments.get_mut(domain).unwrap().push(shuffled_names[index].clone());
                index += 1;
            }
        }
    }

    // Assign any remaining people
    let mut remaining = shuffled_names.into_iter().skip(index).collect::<Vec<_>>();
    while !remaining.is_empty() {
        // Find the least populated domain without holding a mutable reference
        let least_populated = assignments
            .iter()
            .min_by_key(|(_, v)| v.len())
            .map(|(k, _)| *k)
            .unwrap();

        // Modify the least populated domain
        assignments.get_mut(least_populated).unwrap().push(remaining.remove(0));
    }

    assignments
}
