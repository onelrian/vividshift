use async_trait::async_trait;
use std::collections::HashMap;
use anyhow::Result;

use crate::models::{Assignment, GenericEntity};
use crate::services::rule_engine::{AssignmentStrategy, StrategyConfig, ExecutionContext};

pub struct SkillBasedStrategy;

impl SkillBasedStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AssignmentStrategy for SkillBasedStrategy {
    async fn execute(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        _config: &StrategyConfig,
    ) -> Result<Vec<Assignment>> {
        let mut assignments = Vec::new();
        
        // Skill-based assignment logic
        for target in targets {
            let required_count: u32 = target.get_attribute("required_count").unwrap_or(1);
            let required_skills: Vec<String> = target.get_attribute("required_skills").unwrap_or_default();
            
            // Find participants with matching skills
            let mut suitable_participants: Vec<_> = participants
                .iter()
                .filter(|p| {
                    let participant_skills: Vec<String> = p.get_attribute("skills").unwrap_or_default();
                    required_skills.iter().any(|skill| participant_skills.contains(skill))
                })
                .collect();
            
            // Sort by skill match score (simple implementation)
            suitable_participants.sort_by(|a, b| {
                let a_skills: Vec<String> = a.get_attribute("skills").unwrap_or_default();
                let b_skills: Vec<String> = b.get_attribute("skills").unwrap_or_default();
                
                let a_score = required_skills.iter()
                    .filter(|skill| a_skills.contains(skill))
                    .count();
                let b_score = required_skills.iter()
                    .filter(|skill| b_skills.contains(skill))
                    .count();
                
                b_score.cmp(&a_score) // Descending order
            });
            
            // Assign top matching participants
            for participant in suitable_participants.iter().take(required_count as usize) {
                let participant_skills: Vec<String> = participant.get_attribute("skills").unwrap_or_default();
                let skill_match_count = required_skills.iter()
                    .filter(|skill| participant_skills.contains(skill))
                    .count();
                
                let confidence = if required_skills.is_empty() {
                    0.5
                } else {
                    skill_match_count as f64 / required_skills.len() as f64
                };
                
                assignments.push(Assignment {
                    participant_id: participant.id,
                    target_id: target.id,
                    confidence,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("strategy".to_string(), serde_json::json!("skill_based"));
                        meta.insert("skill_match_score".to_string(), serde_json::json!(skill_match_count));
                        meta.insert("target_name".to_string(), 
                                   serde_json::json!(target.get_attribute::<String>("name").unwrap_or_default()));
                        meta
                    },
                });
            }
        }
        
        Ok(assignments)
    }

    fn name(&self) -> &str {
        "skill_based"
    }

    fn description(&self) -> &str {
        "Assigns participants based on skill matching"
    }

    fn validate_config(&self, _config: &StrategyConfig) -> Result<()> {
        // Skill-based assignment doesn't need specific configuration validation
        Ok(())
    }
}
