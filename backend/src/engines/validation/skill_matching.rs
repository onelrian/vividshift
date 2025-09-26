use async_trait::async_trait;
use std::collections::HashMap;

use crate::models::{Assignment, GenericEntity, ValidationResult, ValidationSeverity};
use crate::services::rule_engine::ValidationRule;

pub struct SkillMatchingValidator;

#[async_trait]
impl ValidationRule for SkillMatchingValidator {
    async fn validate(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult, anyhow::Error> {
        let mut issues = Vec::new();
        let strict_mode = config.get("strict")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        for assignment in assignments {
            // Find the participant and target
            let participant = participants.iter()
                .find(|p| p.id == assignment.participant_id);
            let target = targets.iter()
                .find(|t| t.id == assignment.target_id);

            if let (Some(participant), Some(target)) = (participant, target) {
                let participant_skills: Vec<String> = participant.get_attribute("skills").unwrap_or_default();
                let required_skills: Vec<String> = target.get_attribute("required_skills").unwrap_or_default();

                if !required_skills.is_empty() {
                    let matching_skills: Vec<_> = required_skills.iter()
                        .filter(|skill| participant_skills.contains(skill))
                        .collect();

                    let match_percentage = matching_skills.len() as f64 / required_skills.len() as f64;

                    if match_percentage < 0.5 {
                        let _severity = if strict_mode {
                            ValidationSeverity::Error
                        } else {
                            ValidationSeverity::Warning
                        };

                        issues.push(format!(
                            "Participant {} has low skill match ({:.1}%) for target {}. Required: {:?}, Has: {:?}",
                            participant.get_attribute::<String>("name").unwrap_or_default(),
                            match_percentage * 100.0,
                            target.get_attribute::<String>("name").unwrap_or_default(),
                            required_skills,
                            participant_skills
                        ));
                    }
                }
            }
        }

        let severity = if strict_mode && !issues.is_empty() {
            ValidationSeverity::Error
        } else if !issues.is_empty() {
            ValidationSeverity::Warning
        } else {
            ValidationSeverity::Info
        };

        Ok(ValidationResult {
            rule_name: "skill_matching".to_string(),
            passed: issues.is_empty() || !strict_mode,
            message: Some(if issues.is_empty() {
                "All assignments have adequate skill matching".to_string()
            } else {
                format!("Found {} skill matching issues: {}", issues.len(), issues.join("; "))
            }),
            severity,
        })
    }

    fn name(&self) -> &str {
        "skill_matching"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Warning
    }
}
