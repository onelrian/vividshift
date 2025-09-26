use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Assignment, GenericEntity};

pub struct WorkloadBalancer;

impl WorkloadBalancer {
    pub fn new() -> Self {
        Self
    }

    /// Balance workload across participants by redistributing assignments
    pub fn balance_workload(
        &self,
        assignments: Vec<Assignment>,
        _participants: &[GenericEntity],
    ) -> Vec<Assignment> {
        // Count assignments per participant
        let mut assignment_counts: HashMap<Uuid, usize> = HashMap::new();
        for assignment in &assignments {
            *assignment_counts.entry(assignment.participant_id).or_insert(0) += 1;
        }

        // For now, return assignments as-is
        // In a full implementation, this would redistribute assignments
        // to balance workload more evenly
        assignments
    }

    /// Calculate workload distribution statistics
    pub fn calculate_distribution_stats(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
    ) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if participants.is_empty() {
            return stats;
        }

        // Count assignments per participant
        let mut assignment_counts: HashMap<Uuid, usize> = HashMap::new();
        for assignment in assignments {
            *assignment_counts.entry(assignment.participant_id).or_insert(0) += 1;
        }

        let counts: Vec<usize> = participants.iter()
            .map(|p| assignment_counts.get(&p.id).copied().unwrap_or(0))
            .collect();

        let total_assignments = counts.iter().sum::<usize>() as f64;
        let mean = total_assignments / participants.len() as f64;
        
        let variance = counts.iter()
            .map(|&count| {
                let diff = count as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / participants.len() as f64;

        stats.insert("mean_assignments".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_deviation".to_string(), variance.sqrt());
        stats.insert("total_assignments".to_string(), total_assignments);

        stats
    }
}
