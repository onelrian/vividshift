use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::models::{Assignment, GenericEntity, ValidationResult, ValidationSeverity};
use crate::services::rule_engine::ValidationRule;

/// Validates that all assigned participants are available
pub struct AvailabilityCheckValidator;

#[async_trait]
impl ValidationRule for AvailabilityCheckValidator {
    async fn validate(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
        _targets: &[GenericEntity],
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult> {
        let strict_mode = config
            .get("strict")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create a map of participant availability
        let mut participant_availability: HashMap<uuid::Uuid, bool> = HashMap::new();
        for participant in participants {
            let availability: bool = participant
                .get_attribute("availability")
                .unwrap_or(true); // Default to available if not specified
            participant_availability.insert(participant.id, availability);
        }

        let mut unavailable_assignments = Vec::new();

        // Check each assignment
        for assignment in assignments {
            if let Some(&available) = participant_availability.get(&assignment.participant_id) {
                if !available {
                    // Find participant name for better error message
                    let participant_name = participants
                        .iter()
                        .find(|p| p.id == assignment.participant_id)
                        .and_then(|p| p.get_attribute::<String>("name"))
                        .unwrap_or_else(|| format!("Participant-{}", assignment.participant_id));

                    unavailable_assignments.push(participant_name);
                }
            }
        }

        let passed = unavailable_assignments.is_empty();
        let message = if unavailable_assignments.is_empty() {
            Some("All assigned participants are available".to_string())
        } else {
            Some(format!(
                "Unavailable participants assigned: {}",
                unavailable_assignments.join(", ")
            ))
        };

        Ok(ValidationResult {
            rule_name: self.name().to_string(),
            passed,
            message,
            severity: if strict_mode {
                ValidationSeverity::Error
            } else {
                ValidationSeverity::Warning
            },
        })
    }

    fn name(&self) -> &str {
        "availability_check"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Warning
    }
}
