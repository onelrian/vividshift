use async_trait::async_trait;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use anyhow::Result;

use crate::models::{Assignment, GenericEntity};
use crate::services::rule_engine::{AssignmentStrategy, StrategyConfig, ExecutionContext};

pub struct RandomAssignmentStrategy;

impl RandomAssignmentStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AssignmentStrategy for RandomAssignmentStrategy {
    async fn execute(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        _config: &StrategyConfig,
    ) -> Result<Vec<Assignment>> {
        let mut assignments = Vec::new();
        let mut rng = rand::thread_rng();
        
        // Shuffle participants for randomness
        let mut shuffled_participants = participants.to_vec();
        shuffled_participants.shuffle(&mut rng);
        
        // Simple random assignment
        for target in targets {
            let required_count: u32 = target.get_attribute("required_count").unwrap_or(1);
            
            for _ in 0..required_count {
                if let Some(participant) = shuffled_participants.first() {
                    assignments.push(Assignment {
                        participant_id: participant.id,
                        target_id: target.id,
                        confidence: 0.5, // Random assignment has medium confidence
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("strategy".to_string(), serde_json::json!("random"));
                            meta.insert("target_name".to_string(), 
                                       serde_json::json!(target.get_attribute::<String>("name").unwrap_or_default()));
                            meta
                        },
                    });
                }
            }
        }
        
        Ok(assignments)
    }

    fn name(&self) -> &str {
        "random_assignment"
    }

    fn description(&self) -> &str {
        "Randomly assigns participants to targets"
    }

    fn validate_config(&self, _config: &StrategyConfig) -> Result<()> {
        // Random assignment doesn't need specific configuration validation
        Ok(())
    }
}
