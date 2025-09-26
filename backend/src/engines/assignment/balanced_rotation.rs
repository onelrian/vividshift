use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Assignment, GenericEntity};
use crate::services::rule_engine::{AssignmentStrategy, StrategyConfig};

/// Balanced rotation assignment strategy
/// Distributes participants across targets while considering assignment history
pub struct BalancedRotationStrategy {
    history: HashMap<Uuid, Vec<String>>, // participant_id -> list of previous assignments
}

impl BalancedRotationStrategy {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn with_history(history: HashMap<Uuid, Vec<String>>) -> Self {
        Self { history }
    }

    fn get_rotation_weight(&self, config: &StrategyConfig) -> f64 {
        config
            .parameters
            .get("rotation_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7)
    }

    fn get_balance_weight(&self, config: &StrategyConfig) -> f64 {
        config
            .parameters
            .get("balance_weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3)
    }

    fn get_max_attempts(&self, config: &StrategyConfig) -> u32 {
        config
            .parameters
            .get("max_attempts")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as u32
    }

    fn calculate_assignment_score(
        &self,
        participant_id: Uuid,
        target_name: &str,
        rotation_weight: f64,
        balance_weight: f64,
    ) -> f64 {
        // Calculate rotation score (higher if participant hasn't done this recently)
        let rotation_score = if let Some(history) = self.history.get(&participant_id) {
            let recent_assignments = history.len().min(10); // Consider last 10 assignments
            let target_count = history.iter().filter(|&t| t == target_name).count();
            
            if recent_assignments == 0 {
                1.0 // New participant gets full score
            } else {
                1.0 - (target_count as f64 / recent_assignments as f64)
            }
        } else {
            1.0 // No history means full rotation score
        };

        // Balance score (could be enhanced with workload balancing)
        let balance_score = 0.5; // Placeholder for now

        rotation_weight * rotation_score + balance_weight * balance_score
    }

    fn attempt_assignment(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &StrategyConfig,
    ) -> Result<Vec<Assignment>> {
        let rotation_weight = self.get_rotation_weight(config);
        let balance_weight = self.get_balance_weight(config);
        
        let mut assignments = Vec::new();
        let mut available_participants: Vec<_> = participants.iter().collect();
        let mut rng = thread_rng();

        // Shuffle participants for randomness
        available_participants.shuffle(&mut rng);

        for target in targets {
            let target_name: String = target
                .get_attribute("name")
                .ok_or_else(|| anyhow!("Target missing name attribute"))?;
            
            let required_count: u32 = target
                .get_attribute("required_count")
                .ok_or_else(|| anyhow!("Target missing required_count attribute"))?;

            // Calculate scores for all available participants for this target
            let mut participant_scores: Vec<_> = available_participants
                .iter()
                .map(|p| {
                    let score = self.calculate_assignment_score(
                        p.id,
                        &target_name,
                        rotation_weight,
                        balance_weight,
                    );
                    (*p, score)
                })
                .collect();

            // Sort by score (highest first)
            participant_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // Assign top-scoring participants
            let assignments_needed = required_count.min(participant_scores.len() as u32);
            
            for i in 0..assignments_needed as usize {
                let (participant, score) = participant_scores[i];
                
                assignments.push(Assignment {
                    participant_id: participant.id,
                    target_id: target.id,
                    confidence: score,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("strategy".to_string(), Value::String("balanced_rotation".to_string()));
                        meta.insert("score".to_string(), Value::Number(serde_json::Number::from_f64(score).unwrap()));
                        meta.insert("target_name".to_string(), Value::String(target_name.clone()));
                        meta
                    },
                });

                // Remove assigned participant from available pool
                available_participants.retain(|p| p.id != participant.id);
            }

            // Check if we couldn't fulfill the requirement
            if assignments_needed < required_count {
                return Err(anyhow!(
                    "Not enough participants available for target '{}'. Required: {}, Available: {}",
                    target_name,
                    required_count,
                    assignments_needed
                ));
            }
        }

        Ok(assignments)
    }
}

#[async_trait]
impl AssignmentStrategy for BalancedRotationStrategy {
    async fn execute(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &StrategyConfig,
    ) -> Result<Vec<Assignment>> {
        let max_attempts = self.get_max_attempts(config);

        for attempt in 1..=max_attempts {
            match self.attempt_assignment(participants, targets, config) {
                Ok(assignments) => {
                    tracing::info!(
                        "Balanced rotation assignment succeeded on attempt {}",
                        attempt
                    );
                    return Ok(assignments);
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(anyhow!(
                            "Failed to generate assignment after {} attempts. Last error: {}",
                            max_attempts,
                            e
                        ));
                    }
                    tracing::debug!("Assignment attempt {} failed: {}", attempt, e);
                }
            }
        }

        Err(anyhow!("Assignment generation failed"))
    }

    fn name(&self) -> &str {
        "balanced_rotation"
    }

    fn description(&self) -> &str {
        "Balanced assignment strategy that considers rotation history and workload balance"
    }

    fn validate_config(&self, config: &StrategyConfig) -> Result<()> {
        // Validate rotation_weight
        if let Some(weight) = config.parameters.get("rotation_weight") {
            if let Some(w) = weight.as_f64() {
                if !(0.0..=1.0).contains(&w) {
                    return Err(anyhow!("rotation_weight must be between 0.0 and 1.0"));
                }
            } else {
                return Err(anyhow!("rotation_weight must be a number"));
            }
        }

        // Validate balance_weight
        if let Some(weight) = config.parameters.get("balance_weight") {
            if let Some(w) = weight.as_f64() {
                if !(0.0..=1.0).contains(&w) {
                    return Err(anyhow!("balance_weight must be between 0.0 and 1.0"));
                }
            } else {
                return Err(anyhow!("balance_weight must be a number"));
            }
        }

        // Validate max_attempts
        if let Some(attempts) = config.parameters.get("max_attempts") {
            if let Some(a) = attempts.as_u64() {
                if a == 0 || a > 1000 {
                    return Err(anyhow!("max_attempts must be between 1 and 1000"));
                }
            } else {
                return Err(anyhow!("max_attempts must be a positive integer"));
            }
        }

        Ok(())
    }
}

impl Default for BalancedRotationStrategy {
    fn default() -> Self {
        Self::new()
    }
}
