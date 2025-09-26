use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::models::{Assignment, GenericEntity, ValidationResult, ValidationSeverity};
use crate::services::rule_engine::ValidationRule;

/// Validates that assignment targets don't exceed their capacity
pub struct CapacityCheckValidator;

#[async_trait]
impl ValidationRule for CapacityCheckValidator {
    async fn validate(
        &self,
        assignments: &[Assignment],
        _participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult, anyhow::Error> {
        let strict_mode = config
            .get("strict")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut target_counts: HashMap<uuid::Uuid, usize> = HashMap::new();
        
        // Count assignments per target
        for assignment in assignments {
            *target_counts.entry(assignment.target_id).or_insert(0) += 1;
        }

        let mut violations = Vec::new();

        // Check each target's capacity
        for target in targets {
            let required_count: u32 = target
                .get_attribute("required_count")
                .unwrap_or(1);
            
            let assigned_count = target_counts.get(&target.id).copied().unwrap_or(0);
            let target_name: String = target
                .get_attribute("name")
                .unwrap_or_else(|| format!("Target-{}", target.id));

            if assigned_count > required_count as usize {
                violations.push(format!(
                    "Target '{}' has {} assignments but only needs {}",
                    target_name, assigned_count, required_count
                ));
            } else if assigned_count < required_count as usize {
                let message = format!(
                    "Target '{}' has {} assignments but needs {}",
                    target_name, assigned_count, required_count
                );
                
                if strict_mode {
                    violations.push(message);
                } else {
                    tracing::warn!("Capacity warning: {}", message);
                }
            }
        }

        let passed = violations.is_empty();
        let message = if violations.is_empty() {
            Some("All targets have appropriate capacity".to_string())
        } else {
            Some(violations.join("; "))
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
        "capacity_check"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Error
    }
}
